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
/// Video source format, separate from the optional ONVIF control service.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum StreamKind {
    #[default]
    Auto,
    Rtsp,
    Http,
    Mjpeg,
    Jpeg,
}
fn default_snapshot_interval() -> u32 {
    1000
}
fn default_true() -> bool {
    true
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
    #[serde(default)]
    pub folder: Option<String>,
    pub id: String,
    pub name: String,
    pub address: String,
    pub port: u16,
    pub protocol: Protocol,
    pub stream_url: String,
    #[serde(default)]
    pub stream_kind: StreamKind,
    #[serde(default = "default_snapshot_interval")]
    pub snapshot_interval_ms: u32,
    #[serde(default = "default_true")]
    pub mjpeg_multipart: bool,
    pub substream_url: Option<String>,
    pub secret_ref: Option<String>,
    #[serde(default)]
    pub no_credentials: bool,
    #[serde(default = "default_true")]
    pub tls_verify: bool,
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
            folder: None,
            name: String::new(),
            address: String::new(),
            port: 554,
            protocol: Protocol::Rtsp,
            stream_url: String::new(),
            stream_kind: StreamKind::Auto,
            snapshot_interval_ms: default_snapshot_interval(),
            mjpeg_multipart: true,
            substream_url: None,
            secret_ref: None,
            no_credentials: false,
            tls_verify: true,
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
    /// The stream URL is the only editable source of video host/port/protocol.
    pub fn set_stream_url(&mut self, value: &str) -> Result<()> {
        let value = value.trim();
        validate_stream(value)?;
        let url = url::Url::parse(value)?;
        self.address = url.host_str().unwrap_or_default().into();
        self.port = url.port().unwrap_or(match url.scheme() {
            "https" => 443,
            "http" => 80,
            _ => 554,
        });
        self.protocol = if !self.capabilities.device_url.is_empty() {
            Protocol::Onvif
        } else {
            match url.scheme() {
                "https" => Protocol::Https,
                "http" => Protocol::Http,
                _ => Protocol::Rtsp,
            }
        };
        self.stream_url = value.into();
        Ok(())
    }
    /// Changing a device endpoint must never retain authorization/capabilities of the old device.
    pub fn set_onvif_endpoint(&mut self, value: &str) -> Result<()> {
        let value = value.trim();
        if !value.is_empty() {
            let u =
                url::Url::parse(value).map_err(|_| anyhow::anyhow!("Endereço ONVIF inválido."))?;
            ensure!(
                value.len() <= 4096
                    && matches!(u.scheme(), "http" | "https")
                    && u.host_str().is_some()
                    && u.username().is_empty()
                    && u.password().is_none(),
                "Use um endereço ONVIF HTTP/HTTPS sem credenciais na URL."
            );
        }
        if self.capabilities.device_url != value {
            self.capabilities = Capabilities {
                device_url: value.into(),
                ..Default::default()
            };
            self.profiles.clear();
            self.profile_token = None;
            self.ptz_allowed = false;
            self.manufacturer.clear();
            self.model.clear();
        }
        Ok(())
    }
    pub fn validate_source(&self, value: &str) -> Result<()> {
        validate_stream(value)?;
        let scheme = url::Url::parse(value)?.scheme().to_string();
        match self.stream_kind {
            StreamKind::Rtsp => ensure!(scheme == "rtsp", "Para RTSP, informe uma URL rtsp://."),
            StreamKind::Http | StreamKind::Mjpeg | StreamKind::Jpeg => ensure!(
                matches!(scheme.as_str(), "http" | "https"),
                "Para HTTP(S), MJPEG ou JPEG, informe uma URL http:// ou https://."
            ),
            StreamKind::Auto => {}
        }
        Ok(())
    }
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
        self.validate_source(&self.stream_url)?;
        if self.stream_kind == StreamKind::Jpeg {
            ensure!(
                (200..=60000).contains(&self.snapshot_interval_ms),
                "Atualização JPEG deve ficar entre 200 e 60000 ms."
            );
        }
        if let Some(s) = &self.substream_url {
            self.validate_source(s)?;
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
    /// Reorganize existing panels transactionally, preserving identities, cameras and aspect.
    /// Reject smaller layouts that cannot hold all panels instead of silently dropping work.
    pub fn reflow(&mut self, columns: u8, rows: u8, main: bool) -> Result<()> {
        ensure!(
            (1..=16).contains(&columns) && (1..=16).contains(&rows),
            "Grade inválida."
        );
        let mut next = self.clone();
        next.columns = columns;
        next.rows = rows;
        next.panels.clear();
        for (index, old) in self.panels.iter().enumerate() {
            let mut p = old.clone();
            if main && index == 0 {
                p.width = 2;
                p.height = 2;
            }
            let mut found = None;
            for y in 0..rows {
                for x in 0..columns {
                    p.x = x;
                    p.y = y;
                    next.panels.push(p.clone());
                    let valid = next.validate().is_ok();
                    next.panels.pop();
                    if valid {
                        found = Some(p.clone());
                        break;
                    }
                }
                if found.is_some() {
                    break;
                }
            }
            let p=found.ok_or_else(||anyhow::anyhow!("Este layout não comporta todos os painéis. Aumente a grade ou ajuste os tamanhos; nenhuma câmera foi removida."))?;
            next.panels.push(p);
        }
        next.validate()?;
        *self = next;
        Ok(())
    }
    pub fn scale_grid(&mut self, columns: u8, rows: u8) -> Result<()> {
        self.validate()?;
        ensure!(
            (1..=16).contains(&columns) && (1..=16).contains(&rows),
            "Grade inválida."
        );
        let mut next = self.clone();
        for p in &mut next.panels {
            let edge = |n: u8, old: u8, new: u8| {
                ((u16::from(n) * u16::from(new) + u16::from(old) / 2) / u16::from(old)) as u8
            };
            let right = edge(p.x + p.width, self.columns, columns);
            let bottom = edge(p.y + p.height, self.rows, rows);
            p.x = edge(p.x, self.columns, columns);
            p.y = edge(p.y, self.rows, rows);
            p.width = right.saturating_sub(p.x).max(1);
            p.height = bottom.saturating_sub(p.y).max(1);
        }
        next.columns = columns;
        next.rows = rows;
        next.validate()?;
        *self = next;
        Ok(())
    }
    pub fn dashboard_layout(&mut self, columns: u8, rows: u8, main: bool) -> Result<()> {
        ensure!(
            [1, 2, 3, 4].contains(&columns) && [1, 2, 3, 4].contains(&rows),
            "Layout inválido."
        );
        let slots: Vec<_> = if main {
            vec![
                (0, 0, 8, 8),
                (8, 0, 4, 4),
                (8, 4, 4, 4),
                (0, 8, 4, 4),
                (4, 8, 4, 4),
                (8, 8, 4, 4),
            ]
        } else {
            (0..rows)
                .flat_map(|y| {
                    (0..columns).map(move |x| {
                        (x * (12 / columns), y * (12 / rows), 12 / columns, 12 / rows)
                    })
                })
                .collect()
        };
        ensure!(
            self.panels.len() <= slots.len(),
            "O layout não comporta todos os painéis. Nenhuma câmera foi removida."
        );
        let mut next = self.clone();
        next.columns = 12;
        next.rows = 12;
        for (p, &(x, y, w, h)) in next.panels.iter_mut().zip(&slots) {
            p.x = x;
            p.y = y;
            p.width = w;
            p.height = h;
        }
        next.validate()?;
        *self = next;
        Ok(())
    }
    pub fn add_camera(&mut self, camera_id: &str, position: Option<(u8, u8)>) -> Result<()> {
        self.add_camera_sized(camera_id, position, (1, 1))
    }
    pub fn add_camera_sized(
        &mut self,
        camera_id: &str,
        position: Option<(u8, u8)>,
        size: (u8, u8),
    ) -> Result<()> {
        ensure!(!camera_id.is_empty(), "Selecione uma câmera.");
        let cells: Vec<_> = if let Some(p) = position {
            vec![p]
        } else {
            (0..self.rows)
                .flat_map(|y| (0..self.columns).map(move |x| (x, y)))
                .collect()
        };
        for (x, y) in cells {
            let p = Panel {
                id: id(),
                camera_id: camera_id.into(),
                x,
                y,
                width: size.0,
                height: size.1,
                aspect: Aspect::Fit,
            };
            let mut next = self.clone();
            next.panels.push(p);
            if next.validate().is_ok() {
                *self = next;
                return Ok(());
            }
        }
        bail!("Não há espaço livre nesta posição. Reduza o tamanho, escolha outra posição ou remova um painel.")
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

#[cfg(test)]
mod ux_regressions {
    use super::*;
    #[test]
    fn endpoint_and_stream_are_consistent() -> Result<()> {
        let mut c = Camera {
            name: "Entrada".into(),
            ..Default::default()
        };
        c.set_onvif_endpoint("https://camera.local/onvif/device_service")?;
        c.set_stream_url(" rtsp://camera.local:8554/live ")?;
        assert_eq!(c.address, "camera.local");
        assert_eq!(c.port, 8554);
        assert_eq!(c.protocol, Protocol::Onvif);
        c.capabilities.ptz_url = Some("https://camera.local/ptz".into());
        c.ptz_allowed = true;
        c.profile_token = Some("old".into());
        c.set_onvif_endpoint("https://new.local/onvif")?;
        assert!(c.capabilities.ptz_url.is_none());
        assert!(!c.ptz_allowed);
        assert!(c.profile_token.is_none());
        assert!(c.set_onvif_endpoint("http://u:p@host/onvif").is_err());
        c.set_onvif_endpoint("")?;
        c.set_stream_url("https://camera.local/image")?;
        assert_eq!(c.port, 443);
        assert_eq!(c.protocol, Protocol::Https);
        Ok(())
    }
    #[test]
    fn reflow_preserves_panels_and_rejects_data_loss() -> Result<()> {
        let mut v = View::default();
        v.add_camera("B", Some((0, 0)))?;
        v.add_camera("A", Some((1, 0)))?;
        v.panels[0].aspect = Aspect::Fill;
        let before = v.panels.clone();
        v.reflow(3, 3, false)?;
        for (old, new) in before.iter().zip(&v.panels) {
            assert_eq!(old.id, new.id);
            assert_eq!(old.camera_id, new.camera_id);
            assert_eq!(old.aspect, new.aspect);
        }
        let saved = v.clone();
        assert!(v.reflow(1, 1, false).is_err());
        assert_eq!(v, saved);
        assert!(v.add_camera("C", Some((0, 0))).is_err());
        assert_eq!(v, saved);
        Ok(())
    }
}

#[cfg(test)]
mod connection_kind_regressions {
    use super::*;
    #[test]
    fn source_kind_and_snapshot_validation() -> Result<()> {
        let mut c = Camera {
            name: "Typed source".into(),
            stream_url: "rtsp://camera.test/live".into(),
            stream_kind: StreamKind::Rtsp,
            ..Default::default()
        };
        c.validate()?;
        c.stream_kind = StreamKind::Http;
        assert!(c.validate().is_err());
        c.set_stream_url("https://camera.test/snapshot.jpg")?;
        c.stream_kind = StreamKind::Jpeg;
        c.validate()?;
        c.snapshot_interval_ms = 0;
        assert!(c.validate().is_err());
        c.snapshot_interval_ms = 1000;
        Ok(())
    }
}

#[cfg(test)]
mod dashboard_tests {
    use super::*;
    #[test]
    fn rescale_and_dashboard_preserve_identity_and_reject_loss() -> Result<()> {
        let mut view = View::default();
        view.add_camera("A", Some((0, 0)))?;
        view.add_camera("B", Some((1, 0)))?;
        view.panels[0].aspect = Aspect::Fill;
        let ids: Vec<_> = view.panels.iter().map(|p| p.id.clone()).collect();
        view.scale_grid(12, 12)?;
        assert_eq!((view.panels[0].width, view.panels[0].height), (6, 6));
        view.dashboard_layout(3, 3, true)?;
        assert_eq!((view.panels[0].width, view.panels[0].height), (8, 8));
        assert_eq!(view.panels[0].aspect, Aspect::Fill);
        assert_eq!(
            view.panels.iter().map(|p| p.id.clone()).collect::<Vec<_>>(),
            ids
        );
        let old = view.clone();
        assert!(view.dashboard_layout(1, 1, false).is_err());
        assert_eq!(view, old);
        assert!(view.add_camera_sized("C", Some((0, 0)), (3, 3)).is_err());
        assert_eq!(view, old);
        view.add_camera_sized("C", Some((0, 8)), (4, 4))?;
        view.validate()?;
        Ok(())
    }
}
