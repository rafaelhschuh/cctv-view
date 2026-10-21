use super::*;
use cctv_core::{StreamKind, Transport};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub const MAX_ARCHIVE_BYTES: u64 = 8 * 1024 * 1024;

fn folder_name(name: &str) -> Result<String> {
    let name = name.trim();
    ensure!(
        !name.is_empty() && name.len() <= 160 && !name.chars().any(char::is_control),
        "Use um nome de pasta entre 1 e 160 bytes, sem caracteres de controle."
    );
    Ok(name.into())
}

fn read_folders(db: &Connection) -> Result<Vec<String>> {
    let data: Option<String> = db
        .query_row(
            "SELECT value FROM settings WHERE key='camera_folders'",
            [],
            |r| r.get(0),
        )
        .optional()?;
    Ok(data
        .map(|data| serde_json::from_str(&data))
        .transpose()?
        .unwrap_or_default())
}

impl Store {
    pub fn folders(&self) -> Result<Vec<String>> {
        read_folders(&self.db)
    }
    pub fn create_folder(&mut self, name: &str) -> Result<()> {
        let name = folder_name(name)?;
        let tx = self
            .db
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let mut folders = read_folders(&tx)?;
        ensure!(
            !folders
                .iter()
                .any(|v| v.to_lowercase() == name.to_lowercase()),
            "Já existe uma pasta com esse nome."
        );
        folders.push(name);
        folders.sort_by_key(|v| v.to_lowercase());
        tx.execute("INSERT INTO settings VALUES('camera_folders',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value", [serde_json::to_string(&folders)?])?;
        tx.commit()?;
        Ok(())
    }
    /// Removing a folder never removes cameras, credentials or View panels.
    pub fn change_folder(&mut self, old: &str, next: Option<&str>) -> Result<()> {
        let tx = self
            .db
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let mut folders = read_folders(&tx)?;
        ensure!(folders.iter().any(|f| f == old), "A pasta não existe mais.");
        let next = next.map(folder_name).transpose()?;
        if let Some(name) = &next {
            ensure!(
                !folders
                    .iter()
                    .any(|f| f != old && f.to_lowercase() == name.to_lowercase()),
                "Já existe uma pasta com esse nome."
            );
        }
        folders.retain(|f| f != old);
        if let Some(name) = &next {
            folders.push(name.clone());
        }
        folders.sort_by_key(|v| v.to_lowercase());
        tx.execute("UPDATE cameras SET data=json_set(data,'$.folder',?2) WHERE json_extract(data,'$.folder')=?1", params![old,next])?;
        tx.execute("INSERT INTO settings VALUES('camera_folders',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value", [serde_json::to_string(&folders)?])?;
        tx.commit()?;
        Ok(())
    }
    pub fn move_camera(&mut self, id: &str, folder: Option<&str>) -> Result<()> {
        let tx = self
            .db
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let folders = read_folders(&tx)?;
        ensure!(
            folder.is_none_or(|f| folders.iter().any(|v| v == f)),
            "A pasta não existe mais."
        );
        let changed = tx.execute(
            "UPDATE cameras SET data=json_set(data,'$.folder',?2) WHERE id=?1",
            params![id, folder],
        )?;
        ensure!(changed == 1, "A câmera não existe mais.");
        tx.commit()?;
        Ok(())
    }
    pub fn prepare_camera_clone(&self, id: &str) -> Result<Camera> {
        let source = self
            .cameras()?
            .into_iter()
            .find(|c| c.id == id)
            .context("A câmera não existe mais.")?;
        let mut camera = PortableCamera::from_camera(&source)?.into_camera()?;
        // Keep the suffix within Camera's 160-byte limit, including multibyte names.
        let mut name = source.name;
        while name.len() > 147 {
            name.pop();
        }
        camera.name = format!("{name} (cópia)");
        Ok(camera)
    }
    pub fn export_cameras(&self) -> Result<String> {
        let archive = CameraArchive {
            format: "cctv-view-cameras".into(),
            version: 1,
            folders: self.folders()?,
            cameras: self
                .cameras()?
                .iter()
                .map(PortableCamera::from_camera)
                .collect::<Result<_>>()?,
        };
        archive.sanitized_json()
    }
    /// Validate the complete archive before starting an atomic additive import.
    pub fn import_cameras(&mut self, archive: CameraArchive) -> Result<usize> {
        archive.validate()?;
        let mut cameras = archive
            .cameras
            .into_iter()
            .map(PortableCamera::into_camera)
            .collect::<Result<Vec<_>>>()?;
        let tx = self
            .db
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let mut folders = read_folders(&tx)?;
        for folder in archive.folders {
            if let Some(existing) = folders
                .iter()
                .find(|f| f.to_lowercase() == folder.to_lowercase())
            {
                for camera in &mut cameras {
                    if camera.folder.as_deref() == Some(&folder) {
                        camera.folder = Some(existing.clone());
                    }
                }
            } else {
                folders.push(folder);
            }
        }
        folders.sort_by_key(|v| v.to_lowercase());
        tx.execute("INSERT INTO settings VALUES('camera_folders',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value", [serde_json::to_string(&folders)?])?;
        for camera in &cameras {
            tx.execute(
                "INSERT INTO cameras VALUES(?1,?2)",
                params![camera.id, serde_json::to_string(camera)?],
            )?;
        }
        tx.commit()?;
        Ok(cameras.len())
    }
}

/// Explicit allowlist: never serialize Camera, Vault references, profiles or app settings.
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CameraArchive {
    pub format: String,
    pub version: u32,
    pub folders: Vec<String>,
    pub cameras: Vec<PortableCamera>,
}
impl CameraArchive {
    pub fn parse(data: &[u8]) -> Result<Self> {
        ensure!(
            data.len() as u64 <= MAX_ARCHIVE_BYTES,
            "O arquivo excede o limite de 8 MiB."
        );
        let archive: Self = serde_json::from_slice(data).map_err(|_| {
            anyhow::anyhow!("Arquivo de câmeras inválido. Use um JSON exportado pelo CCTV View.")
        })?;
        archive.validate()?;
        Ok(archive)
    }
    pub fn sanitized_json(mut self) -> Result<String> {
        self.validate()?;
        for camera in &mut self.cameras {
            camera.stream_url = portable_url(&camera.stream_url)?;
            camera.substream_url = camera
                .substream_url
                .as_deref()
                .map(portable_url)
                .transpose()?;
            camera.onvif_url = portable_url(&camera.onvif_url)?;
        }
        let data = serde_json::to_string_pretty(&self)?;
        ensure!(
            data.len() as u64 <= MAX_ARCHIVE_BYTES,
            "A exportação excede o limite de 8 MiB."
        );
        Ok(data)
    }
    fn validate(&self) -> Result<()> {
        ensure!(
            self.format == "cctv-view-cameras" && self.version == 1,
            "Formato ou versão de arquivo incompatível."
        );
        ensure!(
            self.cameras.len() <= 2000 && self.folders.len() <= 500,
            "O arquivo excede o limite de 2000 câmeras ou 500 pastas."
        );
        let mut names = HashSet::new();
        for name in &self.folders {
            ensure!(
                folder_name(name)? == *name && names.insert(name.to_lowercase()),
                "Pasta inválida ou repetida no arquivo."
            );
        }
        for camera in &self.cameras {
            ensure!(
                camera
                    .folder
                    .as_ref()
                    .is_none_or(|f| self.folders.contains(f)),
                "Câmera aponta para uma pasta ausente do arquivo."
            );
            camera.clone().into_camera()?;
        }
        Ok(())
    }
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortableCamera {
    pub name: String,
    pub folder: Option<String>,
    pub stream_url: String,
    pub substream_url: Option<String>,
    pub onvif_url: String,
    pub stream_kind: StreamKind,
    pub snapshot_interval_ms: u32,
    pub mjpeg_multipart: bool,
    pub no_credentials: bool,
    pub tls_verify: bool,
    pub transport: Transport,
    pub timeout_ms: u32,
    pub audio_enabled: bool,
    pub hardware_decode: bool,
}
// Query strings and fragments may hold passwords/tokens under arbitrary vendor keys.
// Omit all of them rather than pretending a blacklist can identify every secret.
fn portable_url(value: &str) -> Result<String> {
    if value.is_empty() {
        return Ok(String::new());
    }
    let mut url =
        url::Url::parse(value).map_err(|_| anyhow::anyhow!("Uma câmera contém URL inválida."))?;
    ensure!(
        url.username().is_empty() && url.password().is_none(),
        "URLs com credenciais não podem ser transferidas."
    );
    url.set_query(None);
    url.set_fragment(None);
    Ok(url.into())
}
impl PortableCamera {
    fn from_camera(c: &Camera) -> Result<Self> {
        Ok(Self {
            name: c.name.clone(),
            folder: c.folder.clone(),
            stream_url: portable_url(&c.stream_url)?,
            substream_url: c.substream_url.as_deref().map(portable_url).transpose()?,
            onvif_url: portable_url(&c.capabilities.device_url)?,
            stream_kind: c.stream_kind,
            snapshot_interval_ms: c.snapshot_interval_ms,
            mjpeg_multipart: c.mjpeg_multipart,
            no_credentials: c.no_credentials,
            tls_verify: c.tls_verify,
            transport: c.transport.clone(),
            timeout_ms: c.timeout_ms,
            audio_enabled: c.audio_enabled,
            hardware_decode: c.hardware_decode,
        })
    }
    pub fn into_camera(self) -> Result<Camera> {
        let mut c = Camera {
            name: self.name,
            folder: self.folder,
            stream_kind: self.stream_kind,
            snapshot_interval_ms: self.snapshot_interval_ms,
            mjpeg_multipart: self.mjpeg_multipart,
            no_credentials: self.no_credentials,
            tls_verify: self.tls_verify,
            transport: self.transport,
            timeout_ms: self.timeout_ms,
            audio_enabled: self.audio_enabled,
            hardware_decode: self.hardware_decode,
            substream_url: self
                .substream_url
                .as_deref()
                .map(portable_url)
                .transpose()?,
            ..Default::default()
        };
        c.set_onvif_endpoint(&portable_url(&self.onvif_url)?)?;
        c.set_stream_url(&portable_url(&self.stream_url)?)?;
        c.validate()?;
        Ok(c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn camera() -> Camera {
        Camera {
            name: "Portaria".into(),
            stream_url: "rtsp://camera.test/live?password=secret#token".into(),
            secret_ref: Some("vault-private-reference".into()),
            ..Default::default()
        }
    }
    #[test]
    fn folders_preserve_views_and_clone_omits_secrets() -> Result<()> {
        let mut s = Store::memory()?;
        s.create_folder(" Entrada ")?;
        assert!(s.create_folder("entrada").is_err());
        let c = camera();
        s.save_camera(&c)?;
        s.move_camera(&c.id, Some("Entrada"))?;
        let mut v = View::default();
        v.preset(1, 1, std::slice::from_ref(&c.id), false);
        s.save_view(&v)?;
        let copy = s.prepare_camera_clone(&c.id)?;
        s.save_camera(&copy)?;
        assert_ne!(copy.id, c.id);
        assert!(copy.secret_ref.is_none());
        assert!(!copy.ptz_allowed);
        assert_eq!(copy.folder.as_deref(), Some("Entrada"));
        assert_eq!(copy.stream_url, "rtsp://camera.test/live");
        s.change_folder("Entrada", Some("Exterior"))?;
        assert!(s
            .cameras()?
            .iter()
            .all(|c| c.folder.as_deref() == Some("Exterior")));
        s.change_folder("Exterior", None)?;
        assert!(s.cameras()?.iter().all(|c| c.folder.is_none()));
        assert_eq!(s.views()?[0], v);
        assert_eq!(
            s.cameras()?
                .iter()
                .find(|v| v.id == c.id)
                .unwrap()
                .secret_ref,
            c.secret_ref
        );
        assert!(s.move_camera(&c.id, Some("missing")).is_err());
        let mut invalid = c.clone();
        invalid.folder = Some("missing".into());
        assert!(s.save_camera(&invalid).is_err());
        assert!(s
            .cameras()?
            .iter()
            .find(|camera| camera.id == c.id)
            .unwrap()
            .folder
            .is_none());
        Ok(())
    }
    #[test]
    fn import_merges_folder_case_and_rejects_incomplete_batches() -> Result<()> {
        let mut source = Store::memory()?;
        source.create_folder("Exterior")?;
        let mut c = camera();
        c.folder = Some("Exterior".into());
        source.save_camera(&c)?;
        let json = source.export_cameras()?;
        let mut target = Store::memory()?;
        target.create_folder("EXTERIOR")?;
        target.import_cameras(CameraArchive::parse(json.as_bytes())?)?;
        assert_eq!(target.folders()?, vec!["EXTERIOR"]);
        assert_eq!(target.cameras()?[0].folder.as_deref(), Some("EXTERIOR"));
        let mut archive = CameraArchive::parse(json.as_bytes())?;
        let mut bad = archive.cameras[0].clone();
        bad.timeout_ms = 0;
        archive.cameras.push(bad);
        assert!(target.import_cameras(archive).is_err());
        assert_eq!(target.cameras()?.len(), 1);
        let mut data: serde_json::Value = serde_json::from_str(&json)?;
        data["cameras"][0]["stream_url"] = "rtsp://user:password@camera.test/live".into();
        assert!(CameraArchive::parse(serde_json::to_string(&data)?.as_bytes()).is_err());
        assert!(CameraArchive::parse(&vec![b' '; MAX_ARCHIVE_BYTES as usize + 1]).is_err());
        Ok(())
    }
    #[test]
    fn archive_is_additive_atomic_and_contains_no_secret_metadata() -> Result<()> {
        let mut s = Store::memory()?;
        s.create_folder("Vazia")?;
        let c = camera();
        s.save_camera(&c)?;
        let json = s.export_cameras()?;
        for forbidden in [
            "secret_ref",
            "vault-private-reference",
            "password=secret",
            "#token",
            "profiles",
            "app_lock",
        ] {
            assert!(!json.contains(forbidden));
        }
        let a = CameraArchive::parse(json.as_bytes())?;
        assert_eq!(s.import_cameras(a)?, 1);
        assert_eq!(s.cameras()?.len(), 2);
        assert_ne!(s.cameras()?[0].id, s.cameras()?[1].id);
        let mut a = CameraArchive::parse(json.as_bytes())?;
        a.cameras[0].stream_url = "file:///etc/passwd".into();
        assert!(s.import_cameras(a).is_err());
        assert_eq!(s.cameras()?.len(), 2);
        let mut data: serde_json::Value = serde_json::from_str(&json)?;
        data["cameras"][0]["password"] = "bad".into();
        assert!(CameraArchive::parse(serde_json::to_string(&data)?.as_bytes()).is_err());
        data["version"] = 99.into();
        assert!(CameraArchive::parse(serde_json::to_string(&data)?.as_bytes()).is_err());
        assert_eq!(s.folders()?, vec!["Vazia"]);
        Ok(())
    }
}
