mod xml;
use anyhow::{bail, ensure, Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use cctv_core::{Capabilities, Credentials, MediaProfile};
use sha1::{Digest, Sha1};
use std::{collections::HashSet, time::Duration};
use tokio::net::UdpSocket;
use xml::{escape, parse, Node};
const DEVICE: &str = "http://www.onvif.org/ver10/device/wsdl";
const MEDIA: &str = "http://www.onvif.org/ver10/media/wsdl";
const PTZ: &str = "http://www.onvif.org/ver20/ptz/wsdl";
#[derive(Clone)]
pub struct Client {
    http: reqwest::Client,
    credentials: Credentials,
    device: String,
}
#[derive(Clone, Debug)]
pub struct DeviceInfo {
    pub manufacturer: String,
    pub model: String,
    pub capabilities: Capabilities,
    pub profiles: Vec<MediaProfile>,
}
#[derive(Clone, Debug)]
pub struct Discovered {
    pub endpoint: String,
    pub scopes: String,
}
#[derive(Clone, Debug)]
pub struct Preset {
    pub token: String,
    pub name: String,
}
fn endpoint(url: &str) -> Result<url::Url> {
    let u = url::Url::parse(url).context("Endereço ONVIF inválido")?;
    ensure!(
        matches!(u.scheme(), "http" | "https")
            && u.host_str().is_some()
            && u.username().is_empty()
            && u.password().is_none(),
        "Endpoint ONVIF inválido."
    );
    Ok(u)
}
impl Client {
    pub fn new(device: &str, credentials: Credentials) -> Result<Self> {
        endpoint(device)?;
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(4))
            .connect_timeout(Duration::from_secs(2))
            .redirect(reqwest::redirect::Policy::none())
            .build()?;
        Ok(Self {
            http,
            credentials,
            device: device.into(),
        })
    }
    fn security(&self) -> String {
        if self.credentials.username.is_empty() {
            return String::new();
        }
        let nonce = uuid::Uuid::new_v4();
        let created = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let digest = password_digest(nonce.as_bytes(), &created, &self.credentials.password);
        format!("<wsse:Security s:mustUnderstand=\"1\"><wsse:UsernameToken><wsse:Username>{}</wsse:Username><wsse:Password Type=\"http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-username-token-profile-1.0#PasswordDigest\">{digest}</wsse:Password><wsse:Nonce EncodingType=\"http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-soap-message-security-1.0#Base64Binary\">{}</wsse:Nonce><wsu:Created>{created}</wsu:Created></wsse:UsernameToken></wsse:Security>",escape(&self.credentials.username),STANDARD.encode(nonce.as_bytes()))
    }
    async fn soap(
        &self,
        target: &str,
        namespace: &str,
        operation: &str,
        body: &str,
    ) -> Result<Node> {
        let target_url = endpoint(target)?;
        let device_url = endpoint(&self.device)?;
        ensure!(
            target_url.host_str() == device_url.host_str(),
            "Serviço ONVIF em host diferente; cadastre esse host explicitamente."
        );
        let envelope=format!("<s:Envelope xmlns:s=\"http://www.w3.org/2003/05/soap-envelope\" xmlns:wsse=\"http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-secext-1.0.xsd\" xmlns:wsu=\"http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-utility-1.0.xsd\" xmlns:tt=\"http://www.onvif.org/ver10/schema\"><s:Header>{}</s:Header><s:Body><op:{operation} xmlns:op=\"{namespace}\">{body}</op:{operation}></s:Body></s:Envelope>",self.security());
        let mut response = self
            .http
            .post(target_url)
            .header(
                "Content-Type",
                format!("application/soap+xml; charset=utf-8; action=\"{namespace}/{operation}\""),
            )
            .body(envelope)
            .send()
            .await
            .map_err(|_| {
                anyhow::anyhow!(
                    "Não foi possível conectar ao serviço ONVIF (rede, TLS ou timeout)."
                )
            })?;
        ensure!(
            response.status().is_success(),
            "ONVIF retornou HTTP {}. Verifique autenticação e compatibilidade.",
            response.status().as_u16()
        );
        let mut bytes = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|_| anyhow::anyhow!("Resposta ONVIF interrompida."))?
        {
            ensure!(
                bytes.len() + chunk.len() <= 2 * 1024 * 1024,
                "Resposta ONVIF muito grande."
            );
            bytes.extend_from_slice(&chunk);
        }
        let node = parse(std::str::from_utf8(&bytes)?)?;
        if node.find("Fault").is_some() {
            bail!("O dispositivo recusou {operation}. Recurso não suportado, autenticação ou relógio incorreto.");
        }
        Ok(node)
    }
    pub async fn inspect(&self) -> Result<DeviceInfo> {
        let info = self
            .soap(&self.device, DEVICE, "GetDeviceInformation", "")
            .await?;
        let caps = self
            .soap(
                &self.device,
                DEVICE,
                "GetCapabilities",
                "<op:Category>All</op:Category>",
            )
            .await?;
        let media = caps
            .find("Media")
            .context("Serviço Media1 não anunciado pelo dispositivo.")?
            .value("XAddr");
        endpoint(&media)?;
        let ptz = caps
            .find("PTZ")
            .map(|n| n.value("XAddr"))
            .filter(|s| !s.is_empty());
        let capabilities = Capabilities {
            device_url: self.device.clone(),
            media_url: media.clone(),
            ptz_url: ptz,
        };
        let response = self.soap(&media, MEDIA, "GetProfiles", "").await?;
        let mut nodes = vec![];
        response.all("Profiles", &mut nodes);
        let mut profiles = Vec::new();
        for node in nodes {
            let token = node.attr("token");
            if token.is_empty() {
                continue;
            }
            let body=format!("<op:StreamSetup><tt:Stream>RTP-Unicast</tt:Stream><tt:Transport><tt:Protocol>RTSP</tt:Protocol></tt:Transport></op:StreamSetup><op:ProfileToken>{}</op:ProfileToken>",escape(&token));
            let uri = self
                .soap(&media, MEDIA, "GetStreamUri", &body)
                .await?
                .value("Uri");
            cctv_core::validate_stream(&uri).context(
                "O dispositivo retornou URL de stream inválida ou com credenciais embutidas.",
            )?;
            profiles.push(MediaProfile {
                token,
                name: node
                    .child("Name")
                    .map(|n| n.text.clone())
                    .unwrap_or_default(),
                uri,
                width: node.value("Width").parse().unwrap_or(0),
                height: node.value("Height").parse().unwrap_or(0),
                ptz: node.find("PTZConfiguration").is_some(),
            });
        }
        ensure!(
            !profiles.is_empty(),
            "Nenhum perfil de mídia utilizável encontrado."
        );
        Ok(DeviceInfo {
            manufacturer: info.value("Manufacturer"),
            model: info.value("Model"),
            capabilities,
            profiles,
        })
    }
    pub async fn continuous_move(
        &self,
        target: &str,
        profile: &str,
        x: f64,
        y: f64,
        zoom: f64,
    ) -> Result<()> {
        ensure!(
            [x, y, zoom]
                .iter()
                .all(|n| n.is_finite() && (-1.0..=1.0).contains(n)),
            "Velocidade PTZ inválida."
        );
        let body=format!("<op:ProfileToken>{}</op:ProfileToken><op:Velocity><tt:PanTilt x=\"{x}\" y=\"{y}\"/><tt:Zoom x=\"{zoom}\"/></op:Velocity><op:Timeout>PT0.5S</op:Timeout>",escape(profile));
        let result = self
            .soap(target, PTZ, "ContinuousMove", &body)
            .await
            .map(|_| ());
        if result.is_err() {
            let _ = self.stop(target, profile).await;
        }
        result
    }
    pub async fn stop(&self, target: &str, profile: &str) -> Result<()> {
        self.soap(target,PTZ,"Stop",&format!("<op:ProfileToken>{}</op:ProfileToken><op:PanTilt>true</op:PanTilt><op:Zoom>true</op:Zoom>",escape(profile))).await?;
        Ok(())
    }
    pub async fn presets(&self, target: &str, profile: &str) -> Result<Vec<Preset>> {
        let n = self
            .soap(
                target,
                PTZ,
                "GetPresets",
                &format!("<op:ProfileToken>{}</op:ProfileToken>", escape(profile)),
            )
            .await?;
        let mut presets = vec![];
        n.all("Preset", &mut presets);
        Ok(presets
            .into_iter()
            .map(|n| Preset {
                token: n.attr("token"),
                name: n.value("Name"),
            })
            .collect())
    }
    pub async fn goto_preset(&self, target: &str, profile: &str, token: &str) -> Result<()> {
        self.soap(
            target,
            PTZ,
            "GotoPreset",
            &format!(
                "<op:ProfileToken>{}</op:ProfileToken><op:PresetToken>{}</op:PresetToken>",
                escape(profile),
                escape(token)
            ),
        )
        .await?;
        Ok(())
    }
    pub async fn set_preset(&self, target: &str, profile: &str, name: &str) -> Result<String> {
        ensure!(
            !name.trim().is_empty() && name.len() <= 160,
            "Nome de preset inválido."
        );
        Ok(self
            .soap(
                target,
                PTZ,
                "SetPreset",
                &format!(
                    "<op:ProfileToken>{}</op:ProfileToken><op:PresetName>{}</op:PresetName>",
                    escape(profile),
                    escape(name)
                ),
            )
            .await?
            .value("PresetToken"))
    }
}
fn password_digest(nonce: &[u8], created: &str, password: &str) -> String {
    let mut h = Sha1::new();
    h.update(nonce);
    h.update(created.as_bytes());
    h.update(password.as_bytes());
    STANDARD.encode(h.finalize())
}
/// Discovery is opt-in. Only sends an outbound multicast probe; no listening service is left running.
pub async fn discover(duration: Duration) -> Result<Vec<Discovered>> {
    discover_at("239.255.255.250:3702", duration).await
}
async fn discover_at(destination: &str, duration: Duration) -> Result<Vec<Discovered>> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.set_multicast_ttl_v4(1)?;
    let message_id = format!("urn:uuid:{}", uuid::Uuid::new_v4());
    let probe=format!("<s:Envelope xmlns:s=\"http://www.w3.org/2003/05/soap-envelope\" xmlns:a=\"http://schemas.xmlsoap.org/ws/2004/08/addressing\" xmlns:d=\"http://schemas.xmlsoap.org/ws/2005/04/discovery\" xmlns:dn=\"http://www.onvif.org/ver10/network/wsdl\"><s:Header><a:MessageID>{}</a:MessageID><a:To>urn:schemas-xmlsoap-org:ws:2005:04:discovery</a:To><a:Action>http://schemas.xmlsoap.org/ws/2005/04/discovery/Probe</a:Action></s:Header><s:Body><d:Probe><d:Types>dn:NetworkVideoTransmitter</d:Types></d:Probe></s:Body></s:Envelope>",message_id);
    for _ in 0..2 {
        socket.send_to(probe.as_bytes(), destination).await?;
    }
    let deadline = tokio::time::Instant::now() + duration.min(Duration::from_secs(15));
    let mut buf = vec![0; 65536];
    let mut devices = Vec::new();
    let mut seen = HashSet::new();
    while let Ok(Ok((n, _))) = tokio::time::timeout_at(deadline, socket.recv_from(&mut buf)).await {
        if let Ok(text) = std::str::from_utf8(&buf[..n]) {
            if let Ok(node) = parse(text) {
                if node.value("RelatesTo") != message_id {
                    continue;
                }
                let mut matches = vec![];
                node.all("ProbeMatch", &mut matches);
                for m in matches {
                    for address in m.value("XAddrs").split_whitespace() {
                        if endpoint(address).is_ok() && seen.insert(address.to_string()) {
                            devices.push(Discovered {
                                endpoint: address.into(),
                                scopes: m.value("Scopes"),
                            });
                        }
                    }
                }
            }
        }
        if devices.len() >= 256 {
            break;
        }
    }
    Ok(devices)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn digest_and_input_validation() {
        assert_eq!(
            password_digest(b"nonce", "2026-01-01T00:00:00Z", "secret"),
            STANDARD.encode(Sha1::digest(b"nonce2026-01-01T00:00:00Zsecret"))
        );
        assert!(Client::new("file:///etc/passwd", Credentials::default()).is_err());
    }
    #[tokio::test]
    async fn soap_mock_checks_auth_move_and_stop() -> Result<()> {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
        let address = format!("http://{}/onvif/device_service", listener.local_addr()?);
        let worker = std::thread::spawn(move || {
            let mut requests = vec![];
            for _ in 0..2 {
                let (mut s, _) = listener.accept().unwrap();
                s.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
                let mut request = vec![];
                let mut chunk = [0; 4096];
                loop {
                    let n = s.read(&mut chunk).unwrap();
                    request.extend_from_slice(&chunk[..n]);
                    let text = String::from_utf8_lossy(&request);
                    if let Some((header, body)) = text.split_once("\r\n\r\n") {
                        let len = header
                            .lines()
                            .find_map(|l| {
                                l.to_ascii_lowercase()
                                    .strip_prefix("content-length: ")
                                    .and_then(|s| s.parse::<usize>().ok())
                            })
                            .unwrap_or(0);
                        if body.len() >= len {
                            break;
                        }
                    }
                }
                requests.push(String::from_utf8(request).unwrap());
                let body="<s:Envelope xmlns:s=\"http://www.w3.org/2003/05/soap-envelope\"><s:Body/></s:Envelope>";
                write!(
                    s,
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                )
                .unwrap();
            }
            requests
        });
        let client = Client::new(
            &address,
            Credentials {
                username: "operator".into(),
                password: "never-in-xml".into(),
            },
        )?;
        client
            .continuous_move(&address, "profile&1", 0.2, 0., 0.)
            .await?;
        client.stop(&address, "profile&1").await?;
        let requests = worker.join().unwrap();
        assert!(requests[0].contains("PasswordDigest"));
        assert!(!requests[0].contains("never-in-xml"));
        assert!(requests[0].contains("PT0.5S"));
        assert!(requests[0].contains("profile&amp;1"));
        assert!(requests[1].contains("<op:Stop"));
        Ok(())
    }
}

#[cfg(test)]
mod discovery_tests {
    use super::*;
    #[tokio::test]
    async fn discovery_correlates_and_deduplicates_udp_responses() -> Result<()> {
        let server = UdpSocket::bind("127.0.0.1:0").await?;
        let destination = server.local_addr()?.to_string();
        let task = tokio::spawn(async move {
            let mut buf = [0; 8192];
            let (n, peer) = server.recv_from(&mut buf).await?;
            let request = parse(std::str::from_utf8(&buf[..n])?)?;
            let id = request.value("MessageID");
            let reply=format!("<Envelope><Header><RelatesTo>{}</RelatesTo></Header><Body><ProbeMatches><ProbeMatch><XAddrs>http://127.0.0.1/onvif http://127.0.0.1/onvif file:///invalid</XAddrs><Scopes>onvif://name/fixture</Scopes></ProbeMatch></ProbeMatches></Body></Envelope>",escape(&id));
            server.send_to(reply.as_bytes(), peer).await?;
            server.send_to(reply.as_bytes(), peer).await?;
            let wrong = reply
                .replace(&escape(&id), "unrelated")
                .replace("127.0.0.1/onvif", "127.0.0.2/onvif");
            server.send_to(wrong.as_bytes(), peer).await?;
            Ok::<_, anyhow::Error>(())
        });
        let devices = discover_at(&destination, Duration::from_millis(200)).await?;
        task.await??;
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].endpoint, "http://127.0.0.1/onvif");
        Ok(())
    }
}
