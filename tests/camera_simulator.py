#!/usr/bin/env python3
"""Loopback-only fixture: HTTP JPEG/MJPEG/TS/HLS, RTSP TCP H264, SOAP/PTZ, optional HTTPS.
No production code uses this server or any ffmpeg subprocess.
"""
import argparse
import base64
import hashlib
import http.server
import json
import re
import socket
import socketserver
import ssl
import struct
import threading
import time
import xml.etree.ElementTree as ET
from pathlib import Path

ARGS = None
EVENTS = []
EVENT_LOCK = threading.Lock()
NS = 'http://www.onvif.org/ver10/schema'


def event(name, data=''):
    with EVENT_LOCK:
        EVENTS.append({'operation': name, 'data': data, 'time': time.monotonic()})


class HTTP(http.server.BaseHTTPRequestHandler):
    def log_message(self, *_):
        pass

    def do_GET(self):
        if self.path == '/events':
            with EVENT_LOCK:
                body = json.dumps(EVENTS).encode()
            self.send_response(200)
            self.send_header('Content-Length', str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path == '/mjpeg':
            self.send_response(200)
            self.send_header('Content-Type', 'multipart/x-mixed-replace; boundary=frame')
            self.end_headers()
            jpeg = (ARGS.fixtures / 'sample.jpg').read_bytes()
            try:
                while True:
                    self.wfile.write(b'--frame\r\nContent-Type: image/jpeg\r\nContent-Length: ' + str(len(jpeg)).encode() + b'\r\n\r\n' + jpeg + b'\r\n')
                    self.wfile.flush()
                    time.sleep(.1)
            except (BrokenPipeError, ConnectionResetError):
                return
        paths = {'/snapshot.jpg': ('sample.jpg', 'image/jpeg'), '/live.ts': ('sample.ts', 'video/mp2t'),
                 '/live.m3u8': ('live.m3u8', 'application/vnd.apple.mpegurl'), '/segment.ts': ('sample.ts', 'video/mp2t'),
                 '/hevc.ts': ('hevc.ts', 'video/mp2t')}
        if self.path not in paths:
            self.send_error(404)
            return
        filename, content_type = paths[self.path]
        body = (ARGS.fixtures / filename).read_bytes()
        self.send_response(200)
        self.send_header('Content-Type', content_type)
        self.send_header('Content-Length', str(len(body)))
        self.end_headers()
        try:
            self.wfile.write(body)
        except (BrokenPipeError, ConnectionResetError):
            pass

    def do_POST(self):
        length = int(self.headers.get('Content-Length', '0'))
        if length > 65536:
            self.send_error(413)
            return
        root = ET.fromstring(self.rfile.read(length))
        operation = next(iter(root.find('{http://www.w3.org/2003/05/soap-envelope}Body')))
        name = operation.tag.split('}')[-1]
        if ARGS.username:
            username = next((n.text for n in root.iter() if n.tag.endswith('}Username')), '')
            nonce = next((n.text for n in root.iter() if n.tag.endswith('}Nonce')), '')
            created = next((n.text for n in root.iter() if n.tag.endswith('}Created')), '')
            password = next((n.text for n in root.iter() if n.tag.endswith('}Password')), '')
            expected = base64.b64encode(hashlib.sha1(base64.b64decode(nonce) + created.encode() + ARGS.password.encode()).digest()).decode()
            if username != ARGS.username or password != expected:
                self.send_error(401)
                return
        event(name, ET.tostring(operation, encoding='unicode') if name in ('ContinuousMove', 'Stop', 'GotoPreset', 'SetPreset') else '')
        target = f'http://127.0.0.1:{ARGS.http_port}/onvif/device_service'
        bodies = {
            'GetDeviceInformation': '<Manufacturer>CCTV Fixture</Manufacturer><Model>Local simulator</Model>',
            'GetCapabilities': f'<Capabilities><tt:Media><tt:XAddr>{target}</tt:XAddr></tt:Media><tt:PTZ><tt:XAddr>{target}</tt:XAddr></tt:PTZ></Capabilities>',
            'GetProfiles': '<Profiles token="main"><tt:Name>Principal</tt:Name><tt:VideoEncoderConfiguration><tt:Resolution><tt:Width>320</tt:Width><tt:Height>180</tt:Height></tt:Resolution></tt:VideoEncoderConfiguration><tt:PTZConfiguration token="ptz"/></Profiles>',
            'GetStreamUri': f'<MediaUri><tt:Uri>rtsp://127.0.0.1:{ARGS.rtsp_port}/live</tt:Uri></MediaUri>',
            'GetPresets': '<Preset token="home"><tt:Name>Entrada</tt:Name></Preset>',
            'SetPreset': '<PresetToken>saved</PresetToken>',
            'ContinuousMove': '', 'Stop': '', 'GotoPreset': '',
        }
        if name not in bodies:
            self.send_error(400)
            return
        body = f'<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope" xmlns:tt="{NS}"><s:Body><{name}Response>{bodies[name]}</{name}Response></s:Body></s:Envelope>'.encode()
        self.send_response(200)
        self.send_header('Content-Type', 'application/soap+xml')
        self.send_header('Content-Length', str(len(body)))
        self.end_headers()
        self.wfile.write(body)


class RTSP(socketserver.BaseRequestHandler):
    def handle(self):
        conn = self.request
        conn.settimeout(15)
        lock = threading.Lock()
        stopped = threading.Event()
        data = b''
        sender = None
        nals = [n for n in re.split(b'\x00\x00\x00?\x01', (ARGS.fixtures / 'video.h264').read_bytes()) if n]
        sps = next(n for n in nals if n[0] & 31 == 7)
        pps = next(n for n in nals if n[0] & 31 == 8)
        frames = []
        for nal in nals:
            if nal[0] & 31 == 9 or not frames:
                frames.append([])
            frames[-1].append(nal)

        def send_frames():
            seq, timestamp = 0, 0
            try:
                while not stopped.is_set():
                    for frame in frames:
                        for ni, nal in enumerate(frame):
                            payloads = [nal]
                            if len(nal) > 1200:
                                chunks = [nal[i:i+1198] for i in range(1, len(nal), 1198)]
                                payloads = [bytes([(nal[0] & 224) | 28, (nal[0] & 31) | (128 if i == 0 else 0) | (64 if i == len(chunks)-1 else 0)]) + part for i, part in enumerate(chunks)]
                            for pi, payload in enumerate(payloads):
                                marker = ni == len(frame)-1 and pi == len(payloads)-1
                                packet = struct.pack('!BBHII', 128, 96 | (128 if marker else 0), seq & 65535, timestamp & 0xffffffff, 42) + payload
                                with lock:
                                    conn.sendall(b'$\x00' + struct.pack('!H', len(packet)) + packet)
                                seq += 1
                        timestamp += 9000
                        if stopped.wait(.1):
                            return
            except (OSError, BrokenPipeError):
                stopped.set()

        try:
            while not stopped.is_set():
                chunk = conn.recv(65536)
                if not chunk:
                    break
                data += chunk
                while data:
                    if data[0:1] == b'$':
                        if len(data) < 4:
                            break
                        length = struct.unpack('!H', data[2:4])[0]
                        if len(data) < length+4:
                            break
                        data = data[length+4:]
                        continue
                    if b'\r\n\r\n' not in data:
                        break
                    request, data = data.split(b'\r\n\r\n', 1)
                    lines = request.decode().split('\r\n')
                    method, uri, _ = lines[0].split(' ')
                    headers = dict(line.split(': ', 1) for line in lines[1:] if ': ' in line)
                    cseq = headers.get('CSeq', '1')
                    status, extra, body = '200 OK', '', b''
                    auth = base64.b64encode(f'{ARGS.username}:{ARGS.password}'.encode()).decode()
                    if ARGS.username and method != 'OPTIONS' and headers.get('Authorization') != 'Basic ' + auth:
                        status, extra = '401 Unauthorized', 'WWW-Authenticate: Basic realm="CCTV Fixture"\r\n'
                    elif method == 'OPTIONS':
                        extra = 'Public: OPTIONS, DESCRIBE, SETUP, PLAY, GET_PARAMETER, TEARDOWN\r\n'
                    elif method == 'DESCRIBE':
                        body = (f'v=0\r\no=- 0 0 IN IP4 127.0.0.1\r\ns=CCTV Fixture\r\nc=IN IP4 127.0.0.1\r\nt=0 0\r\na=control:*\r\nm=video 0 RTP/AVP 96\r\na=rtpmap:96 H264/90000\r\na=fmtp:96 packetization-mode=1;sprop-parameter-sets={base64.b64encode(sps).decode()},{base64.b64encode(pps).decode()}\r\na=control:trackID=0\r\n').encode()
                        extra = f'Content-Type: application/sdp\r\nContent-Base: {uri}/\r\n'
                    elif method == 'SETUP':
                        if 'RTP/AVP/TCP' not in headers.get('Transport', ''):
                            status = '461 Unsupported Transport'
                        extra = 'Transport: RTP/AVP/TCP;unicast;interleaved=0-1\r\nSession: fixture\r\n'
                    elif method in ('PLAY', 'GET_PARAMETER', 'TEARDOWN'):
                        extra = 'Session: fixture\r\n'
                    response = f'RTSP/1.0 {status}\r\nCSeq: {cseq}\r\n{extra}Content-Length: {len(body)}\r\n\r\n'.encode() + body
                    with lock:
                        conn.sendall(response)
                    if method == 'PLAY' and status == '200 OK' and sender is None:
                        sender = threading.Thread(target=send_frames, daemon=True)
                        sender.start()
                    if method == 'TEARDOWN':
                        stopped.set()
        except (OSError, ValueError):
            pass
        finally:
            stopped.set()


class TCP(socketserver.ThreadingTCPServer):
    allow_reuse_address = True
    daemon_threads = True


def main():
    global ARGS
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--fixtures', type=Path, required=True)
    parser.add_argument('--http-port', type=int, default=18080)
    parser.add_argument('--rtsp-port', type=int, default=18554)
    parser.add_argument('--https-port', type=int, default=18443)
    parser.add_argument('--tls-cert')
    parser.add_argument('--tls-key')
    parser.add_argument('--username', default='')
    parser.add_argument('--password', default='')
    ARGS = parser.parse_args()
    servers = [http.server.ThreadingHTTPServer(('127.0.0.1', ARGS.http_port), HTTP), TCP(('127.0.0.1', ARGS.rtsp_port), RTSP)]
    if ARGS.tls_cert and ARGS.tls_key:
        server = http.server.ThreadingHTTPServer(('127.0.0.1', ARGS.https_port), HTTP)
        context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
        context.load_cert_chain(ARGS.tls_cert, ARGS.tls_key)
        server.socket = context.wrap_socket(server.socket, server_side=True)
        servers.append(server)
    for server in servers:
        threading.Thread(target=server.serve_forever, daemon=True).start()
    print('Loopback camera fixtures ready', flush=True)
    try:
        threading.Event().wait()
    except KeyboardInterrupt:
        for server in servers:
            server.shutdown()


if __name__ == '__main__':
    main()
