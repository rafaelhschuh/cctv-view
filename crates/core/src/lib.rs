use anyhow::{bail, ensure, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const APP_ID: &str = "br.dev.schuh.cctv-view";
pub fn id() -> String {
    Uuid::new_v4().to_string()
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Transport {
    Tcp,
    Udp,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Protocol {
    Rtsp,
    Http,
    Https,
    Onvif,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MediaProfile {
    pub token: String,
    pub name: String,
    pub uri: String,
    pub width: u32,
    pub height: u32,
    pub ptz: bool,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Capabilities {
    pub device_url: String,
    pub media_url: String,
    pub ptz_url: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Camera {
    pub id: String,
    pub name: String,
    pub address: String,
    pub port: u16,
    pub protocol: Protocol,
    pub stream_url: String,
    pub substream_url: Option<String>,
    pub secret_ref: Option<String>,
    pub manufacturer: String,
    pub model: String,
    pub profiles: Vec<MediaProfile>,
    pub profile_token: Option<String>,
    pub transport: Transport,
    pub timeout_ms: u32,
    pub audio_enabled: bool,
    pub ptz_allowed: bool,
    pub capabilities: Capabilities,
    pub last_status: String,
    pub hardware_decode: bool,
}
impl Default for Camera {
    fn default() -> Self {
        Self {
            id: id(),
            name: String::new(),
            address: String::new(),
            port: 554,
            protocol: Protocol::Rtsp,
            stream_url: String::new(),
            substream_url: None,
            secret_ref: None,
            manufacturer: String::new(),
            model: String::new(),
            profiles: vec![],
            profile_token: None,
            transport: Transport::Tcp,
            timeout_ms: 5000,
            audio_enabled: true,
            ptz_allowed: false,
            capabilities: Capabilities::default(),
            last_status: "Nunca conectada".into(),
            hardware_decode: true,
        }
    }
}
impl Camera {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            !self.name.trim().is_empty() && self.name.len() <= 160,
            "Informe um nome com até 160 caracteres."
        );
        ensure!(
            (500..=60000).contains(&self.timeout_ms),
            "Timeout deve ficar entre 500 e 60000 ms."
        );
        ensure!(self.port > 0, "Porta inválida.");
        validate_stream(&self.stream_url)?;
        if let Some(s) = &self.substream_url {
            validate_stream(s)?;
        }
        if self.ptz_allowed {
            ensure!(
                self.capabilities.ptz_url.is_some() && self.profile_token.is_some(),
                "PTZ exige serviço e perfil ONVIF."
            );
        }
        Ok(())
    }
}
pub fn validate_stream(value: &str) -> Result<()> {
    let url = url::Url::parse(value).map_err(|_| anyhow::anyhow!("URL inválida."))?;
    ensure!(
        matches!(url.scheme(), "rtsp" | "http" | "https"),
        "Use RTSP, HTTP ou HTTPS."
    );
    ensure!(
        url.host_str().is_some(),
        "Informe o endereço do dispositivo."
    );
    ensure!(
        url.username().is_empty() && url.password().is_none(),
        "Informe credenciais nos campos protegidos, fora da URL."
    );
    ensure!(
        !value.contains('\0') && value.len() <= 4096,
        "URL inválida."
    );
    // Query strings can contain vendor tokens; do not log URLs, including validated ones.
    Ok(())
}
// Intentionally no Debug/Serialize: credentials must never be logged or persisted with cameras.
#[derive(Clone, Default)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Aspect {
    Fit,
    Fill,
    Stretch,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Panel {
    pub id: String,
    pub camera_id: String,
    pub x: u8,
    pub y: u8,
    pub width: u8,
    pub height: u8,
    pub aspect: Aspect,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct View {
    pub id: String,
    pub name: String,
    pub columns: u8,
    pub rows: u8,
    pub panels: Vec<Panel>,
    pub favorite: bool,
    pub order: i32,
}
impl Default for View {
    fn default() -> Self {
        Self {
            id: id(),
            name: "Monitoramento".into(),
            columns: 2,
            rows: 2,
            panels: vec![],
            favorite: false,
            order: 0,
        }
    }
}
impl View {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            !self.name.trim().is_empty() && self.name.len() <= 160,
            "Informe um nome para a View."
        );
        ensure!(
            (1..=16).contains(&self.columns) && (1..=16).contains(&self.rows),
            "Grade deve ter entre 1 e 16 células por eixo."
        );
        let mut occupied = std::collections::HashSet::new();
        let mut ids = std::collections::HashSet::new();
        for p in &self.panels {
            ensure!(ids.insert(&p.id), "Identificador de painel duplicado.");
            ensure!(
                !p.camera_id.is_empty() && p.width > 0 && p.height > 0,
                "Painel inválido."
            );
            ensure!(
                u16::from(p.x) + u16::from(p.width) <= u16::from(self.columns)
                    && u16::from(p.y) + u16::from(p.height) <= u16::from(self.rows),
                "Painel fora da grade."
            );
            for x in p.x..p.x + p.width {
                for y in p.y..p.y + p.height {
                    if !occupied.insert((x, y)) {
                        bail!("Painéis sobrepostos.");
                    }
                }
            }
        }
        Ok(())
    }
    pub fn preset(&mut self, columns: u8, rows: u8, cameras: &[String], main: bool) {
        self.columns = columns;
        self.rows = rows;
        self.panels.clear();
        let mut index = 0;
        for y in 0..rows {
            for x in 0..columns {
                if main && x < 2 && y < 2 && (x != 0 || y != 0) {
                    continue;
                }
                let Some(camera) = cameras.get(index) else {
                    return;
                };
                self.panels.push(Panel {
                    id: id(),
                    camera_id: camera.clone(),
                    x,
                    y,
                    width: if main && index == 0 { 2 } else { 1 },
                    height: if main && index == 0 { 2 } else { 1 },
                    aspect: Aspect::Fit,
                });
                index += 1;
            }
        }
    }
    pub fn duplicate(&self) -> Self {
        let mut v = self.clone();
        v.id = id();
        v.name = format!("{} — cópia", self.name);
        for p in &mut v.panels {
            p.id = id();
        }
        v
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Preferences {
    pub last_view: Option<String>,
    pub initial_view: Option<String>,
    pub width: i32,
    pub height: i32,
    pub maximized: bool,
    pub sidebar: bool,
    pub volume: f64,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            last_view: None,
            initial_view: None,
            width: 1280,
            height: 800,
            maximized: false,
            sidebar: true,
            volume: 0.7,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_credentials_and_unsafe_protocols() {
        for s in [
            "rtsp://user:pass@host/live",
            "file:///etc/passwd",
            "http:///",
            "ftp://host/video",
        ] {
            assert!(validate_stream(s).is_err(), "{s}");
        }
        assert!(validate_stream("https://camera.local/live.m3u8").is_ok());
    }
    #[test]
    fn layout_bounds_overlap_and_presets() {
        let ids: Vec<_> = (0..16).map(|_| id()).collect();
        let mut v = View::default();
        v.preset(4, 4, &ids, false);
        assert_eq!(v.panels.len(), 16);
        assert!(v.validate().is_ok());
        v.panels[1].x = 0;
        assert!(v.validate().is_err());
        v.preset(3, 3, &ids, true);
        assert_eq!(v.panels.len(), 6);
        assert!(v.validate().is_ok());
        v.panels[0].width = 255;
        assert!(v.validate().is_err());
        let copy = v.duplicate();
        assert_ne!(copy.id, v.id);
        assert_ne!(copy.panels[0].id, v.panels[0].id);
    }
}
