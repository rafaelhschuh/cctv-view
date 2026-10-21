use super::*;

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
    pub fn clone_camera(&mut self, id: &str) -> Result<()> {
        self.clone_with(id, Vault::get, Vault::store, Vault::delete)
    }
    fn clone_with(
        &mut self,
        id: &str,
        mut load: impl FnMut(&str) -> Result<Credentials>,
        save: impl FnMut(&str, &Credentials) -> Result<()>,
        delete: impl FnMut(&str) -> Result<()>,
    ) -> Result<()> {
        // Read the metadata exactly once, before a keyring lookup may block/unlock.
        let mut camera = self
            .cameras()?
            .into_iter()
            .find(|c| c.id == id)
            .context("A câmera não existe mais.")?;
        let credentials = camera.secret_ref.as_deref().map(&mut load).transpose()?;
        camera.id = cctv_core::id();
        camera.secret_ref = None;
        while camera.name.len() > 147 {
            camera.name.pop();
        }
        camera.name.push_str(" (cópia)");
        let archive = CameraArchive::from_records(
            self.folders()?,
            vec![ArchiveCamera::new(camera, credentials)],
        );
        self.import_with(archive, save, delete)?;
        Ok(())
    }
    pub fn export_encrypted(&self, password: &str) -> Result<String> {
        self.archive_with(Vault::get)?.encrypt(password)
    }
    fn archive_with(
        &self,
        mut load: impl FnMut(&str) -> Result<Credentials>,
    ) -> Result<CameraArchive> {
        let mut cameras = vec![];
        for mut camera in self.cameras()? {
            let credentials = camera.secret_ref.as_deref().map(&mut load).transpose()?;
            camera.secret_ref = None;
            cameras.push(ArchiveCamera::new(camera, credentials));
        }
        Ok(CameraArchive::from_records(self.folders()?, cameras))
    }
    pub fn import_cameras(&mut self, archive: CameraArchive) -> Result<usize> {
        self.import_with(archive, Vault::store, Vault::delete)
    }
    fn import_with(
        &mut self,
        archive: CameraArchive,
        mut save: impl FnMut(&str, &Credentials) -> Result<()>,
        mut delete: impl FnMut(&str) -> Result<()>,
    ) -> Result<usize> {
        archive.validate()?;
        let mut records = archive.cameras;
        let tx = self
            .db
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let mut folders = read_folders(&tx)?;
        for folder in archive.folders {
            if let Some(existing) = folders
                .iter()
                .find(|f| f.to_lowercase() == folder.to_lowercase())
            {
                for record in &mut records {
                    if record.camera.folder.as_deref() == Some(&folder) {
                        record.camera.folder = Some(existing.clone());
                    }
                }
            } else {
                folders.push(folder);
            }
        }
        folders.sort_by_key(|v| v.to_lowercase());
        let mut created = vec![];
        let result = (|| -> Result<usize> {
            tx.execute("INSERT INTO settings VALUES('camera_folders',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value",[serde_json::to_string(&folders)?])?;
            for record in &mut records {
                record.camera.id = cctv_core::id();
                record.camera.secret_ref = None;
                if let Some(credentials) = record.credentials.take() {
                    let reference = format!("camera:{}", record.camera.id);
                    created.push(reference.clone());
                    save(&reference, &credentials.into())?;
                    record.camera.secret_ref = Some(reference);
                }
                tx.execute(
                    "INSERT INTO cameras VALUES(?1,?2)",
                    params![record.camera.id, serde_json::to_string(&record.camera)?],
                )?;
            }
            tx.commit()?;
            Ok(records.len())
        })();
        if result.is_err() {
            for reference in created {
                let _ = delete(&reference);
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn camera() -> Camera {
        Camera {
            name: "Portaria".into(),
            stream_url: "rtsp://camera.test/live?channel=1&password=fixture#token".into(),
            secret_ref: Some("original-reference".into()),
            ..Default::default()
        }
    }
    #[test]
    fn folders_and_clone_preserve_complete_camera() -> Result<()> {
        let mut store = Store::memory()?;
        store.create_folder("Entrada")?;
        assert!(store.create_folder("entrada").is_err());
        let c = camera();
        store.save_camera(&c)?;
        store.move_camera(&c.id, Some("Entrada"))?;
        let mut v = View::default();
        v.preset(1, 1, std::slice::from_ref(&c.id), false);
        store.save_view(&v)?;
        store.clone_with(
            &c.id,
            |_| {
                Ok(Credentials {
                    username: "user".into(),
                    password: "fixture-password".into(),
                })
            },
            |_, _| Ok(()),
            |_| Ok(()),
        )?;
        let cloned = store.cameras()?.into_iter().find(|v| v.id != c.id).unwrap();
        assert_ne!(cloned.id, c.id);
        assert_ne!(cloned.secret_ref, c.secret_ref);
        assert_eq!(cloned.stream_url, c.stream_url);
        store.change_folder("Entrada", Some("Externo"))?;
        store.change_folder("Externo", None)?;
        assert!(store.cameras()?.iter().all(|c| c.folder.is_none()));
        assert_eq!(store.views()?[0], v);
        assert_eq!(
            store
                .cameras()?
                .iter()
                .find(|v| v.id == c.id)
                .unwrap()
                .secret_ref,
            c.secret_ref
        );
        assert!(store.move_camera(&c.id, Some("missing")).is_err());
        let mut invalid = c.clone();
        invalid.folder = Some("missing".into());
        assert!(store.save_camera(&invalid).is_err());
        Ok(())
    }
    #[test]
    fn cloning_uses_one_metadata_snapshot_when_keyring_blocks() -> Result<()> {
        let dir = std::env::temp_dir().join(cctv_core::id());
        std::fs::create_dir_all(&dir)?;
        let mut store = Store::open(dir.join("test.db"))?;
        let source = camera();
        store.save_camera(&source)?;
        let other = Store::open(dir.join("test.db"))?;
        let mut changed = source.clone();
        changed.set_stream_url("rtsp://different.test/live")?;
        store.clone_with(
            &source.id,
            |_| {
                other.save_camera(&changed)?;
                Ok(Credentials {
                    username: "original-user".into(),
                    password: "original-password".into(),
                })
            },
            |_, c| {
                assert_eq!(c.password, "original-password");
                Ok(())
            },
            |_| Ok(()),
        )?;
        let copy = store
            .cameras()?
            .into_iter()
            .find(|c| c.id != source.id)
            .unwrap();
        assert_eq!(copy.stream_url, source.stream_url);
        drop(other);
        drop(store);
        std::fs::remove_dir_all(dir)?;
        Ok(())
    }
    #[test]
    fn encrypted_import_restores_credentials_and_rolls_back_failures() -> Result<()> {
        let mut source = Store::memory()?;
        source.create_folder("Exterior")?;
        let mut camera = camera();
        camera.folder = Some("Exterior".into());
        source.save_camera(&camera)?;
        let archive = source.archive_with(|_| {
            Ok(Credentials {
                username: "operator".into(),
                password: "camera-password".into(),
            })
        })?;
        let encrypted = archive.encrypt("file-password")?;
        assert!(!encrypted.contains("camera-password"));
        assert!(!encrypted.contains("Portaria"));
        let mut target = Store::memory()?;
        target.create_folder("EXTERIOR")?;
        let mut secrets = vec![];
        target.import_with(
            CameraArchive::decrypt(encrypted.as_bytes(), "file-password")?,
            |id, c| {
                secrets.push((id.to_string(), c.username.clone(), c.password.clone()));
                Ok(())
            },
            |_| Ok(()),
        )?;
        assert_eq!(target.cameras()?[0].stream_url, camera.stream_url);
        assert_eq!(target.cameras()?[0].folder.as_deref(), Some("EXTERIOR"));
        assert_eq!(secrets[0].2, "camera-password");
        assert_ne!(secrets[0].0, "original-reference");
        let persisted: String =
            target
                .db
                .query_row("SELECT data FROM cameras LIMIT 1", [], |r| r.get(0))?;
        assert!(!persisted.contains("camera-password"));
        assert!(!persisted.contains("operator"));
        let mut broken = CameraArchive::decrypt(encrypted.as_bytes(), "file-password")?;
        broken.cameras.push(broken.cameras[0].clone());
        let mut attempts = 0;
        let mut cleaned = vec![];
        assert!(target
            .import_with(
                broken,
                |_, _| {
                    attempts += 1;
                    ensure!(attempts < 2, "fixture failure");
                    Ok(())
                },
                |id| {
                    cleaned.push(id.to_string());
                    Ok(())
                }
            )
            .is_err());
        assert_eq!(cleaned.len(), 2);
        assert_eq!(target.cameras()?.len(), 1);
        Ok(())
    }
}
