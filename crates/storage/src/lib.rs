use anyhow::{ensure, Context, Result};
use cctv_core::{Camera, Credentials, Preferences, View, APP_ID};
use rusqlite::{params, Connection, OptionalExtension};
use std::{
    ffi::{c_char, c_int, CStr, CString},
    path::{Path, PathBuf},
};

pub struct Store {
    db: Connection,
}
impl Store {
    pub fn default_path() -> Result<PathBuf> {
        let dirs = directories::BaseDirs::new().context("Répertoire XDG indisponível")?;
        let dir = dirs.data_dir().join(APP_ID);
        std::fs::create_dir_all(&dir)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))?;
        }
        Ok(dir.join("cctv.db"))
    }
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        Self::from_connection(Connection::open(path)?)
    }
    pub fn memory() -> Result<Self> {
        Self::from_connection(Connection::open_in_memory()?)
    }
    fn from_connection(db: Connection) -> Result<Self> {
        db.busy_timeout(std::time::Duration::from_secs(3))?;
        db.execute_batch("PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL;")?;
        let version: i32 = db.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        ensure!(version <= 1, "Banco criado por uma versão mais recente.");
        if version == 0 {
            db.execute_batch(
                "BEGIN IMMEDIATE;
            CREATE TABLE cameras(id TEXT PRIMARY KEY, data TEXT NOT NULL);
            CREATE TABLE views(id TEXT PRIMARY KEY, data TEXT NOT NULL);
            CREATE TABLE settings(key TEXT PRIMARY KEY, value TEXT NOT NULL);
            PRAGMA user_version=1; COMMIT;",
            )?;
        }
        Ok(Self { db })
    }
    pub fn cameras(&self) -> Result<Vec<Camera>> {
        self.list("SELECT data FROM cameras ORDER BY json_extract(data,'$.name') COLLATE NOCASE")
    }
    pub fn views(&self) -> Result<Vec<View>> {
        self.list("SELECT data FROM views ORDER BY json_extract(data,'$.order'), json_extract(data,'$.name')")
    }
    fn list<T: serde::de::DeserializeOwned>(&self, sql: &str) -> Result<Vec<T>> {
        let mut statement = self.db.prepare(sql)?;
        let rows = statement.query_map([], |r| r.get::<_, String>(0))?;
        rows.map(|r| Ok(serde_json::from_str(&r?)?)).collect()
    }
    pub fn save_camera(&self, c: &Camera) -> Result<()> {
        c.validate()?;
        self.db.execute(
            "INSERT INTO cameras VALUES(?1,?2) ON CONFLICT(id) DO UPDATE SET data=excluded.data",
            params![c.id, serde_json::to_string(c)?],
        )?;
        Ok(())
    }
    pub fn delete_camera(&mut self, id: &str) -> Result<()> {
        let mut views = self.views()?;
        let tx = self.db.transaction()?;
        for view in &mut views {
            view.panels.retain(|p| p.camera_id != id);
            tx.execute(
                "UPDATE views SET data=?2 WHERE id=?1",
                params![view.id, serde_json::to_string(view)?],
            )?;
        }
        tx.execute("DELETE FROM cameras WHERE id=?1", [id])?;
        tx.commit()?;
        Ok(())
    }
    pub fn save_view(&self, v: &View) -> Result<()> {
        v.validate()?;
        let ids: std::collections::HashSet<_> = self.cameras()?.into_iter().map(|c| c.id).collect();
        ensure!(
            v.panels.iter().all(|p| ids.contains(&p.camera_id)),
            "A View contém câmera inexistente."
        );
        self.db.execute(
            "INSERT INTO views VALUES(?1,?2) ON CONFLICT(id) DO UPDATE SET data=excluded.data",
            params![v.id, serde_json::to_string(v)?],
        )?;
        Ok(())
    }
    pub fn delete_view(&self, id: &str) -> Result<()> {
        self.db.execute("DELETE FROM views WHERE id=?1", [id])?;
        Ok(())
    }
    pub fn preferences(&self) -> Result<Preferences> {
        let v: Option<String> = self
            .db
            .query_row(
                "SELECT value FROM settings WHERE key='preferences'",
                [],
                |r| r.get(0),
            )
            .optional()?;
        Ok(match v {
            Some(s) => serde_json::from_str(&s)?,
            None => Preferences::default(),
        })
    }
    pub fn save_preferences(&self, p: &Preferences) -> Result<()> {
        self.db.execute("INSERT INTO settings VALUES('preferences',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value",[serde_json::to_string(p)?])?;
        Ok(())
    }
}
unsafe extern "C" {
    fn cctv_secret_store(id: *const c_char, value: *const c_char) -> c_int;
    fn cctv_secret_get(id: *const c_char, failed: *mut c_int) -> *mut c_char;
    fn cctv_secret_delete(id: *const c_char) -> c_int;
    fn cctv_secret_free(value: *mut c_char);
}
pub struct Vault;
impl Vault {
    pub fn store(id: &str, c: &Credentials) -> Result<()> {
        let id = CString::new(id)?;
        let data = CString::new(serde_json::to_string(&(
            c.username.as_str(),
            c.password.as_str(),
        ))?)?;
        // SAFETY: NUL-terminated strings live throughout the synchronous call; C retains no pointers.
        ensure!(
            unsafe { cctv_secret_store(id.as_ptr(), data.as_ptr()) } != 0,
            "Não foi possível salvar no chaveiro. Desbloqueie o Secret Service."
        );
        Ok(())
    }
    pub fn get(id: &str) -> Result<Credentials> {
        let id = CString::new(id)?;
        let mut failed = 0;
        // SAFETY: valid output pointer and string; returned allocation belongs to libsecret.
        let ptr = unsafe { cctv_secret_get(id.as_ptr(), &mut failed) };
        ensure!(failed == 0, "Chaveiro indisponível ou bloqueado.");
        ensure!(!ptr.is_null(), "Credenciais não encontradas no chaveiro.");
        // SAFETY: successful libsecret lookup returns a NUL-terminated allocated string.
        let result =
            unsafe { serde_json::from_slice::<(String, String)>(CStr::from_ptr(ptr).to_bytes()) };
        // SAFETY: pointer returned by libsecret; freed exactly once with its matching allocator.
        unsafe { cctv_secret_free(ptr) };
        let (username, password) = result?;
        Ok(Credentials { username, password })
    }
    pub fn delete(id: &str) -> Result<()> {
        let id = CString::new(id)?;
        // SAFETY: temporary valid C string; no pointer retention.
        ensure!(
            unsafe { cctv_secret_delete(id.as_ptr()) } != 0,
            "Falha ao remover credenciais do chaveiro."
        );
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn migrations_crud_and_cascade_layouts() -> Result<()> {
        let mut s = Store::memory()?;
        let mut c = Camera {
            name: "Portaria".into(),
            stream_url: "rtsp://localhost/live".into(),
            ..Default::default()
        };
        s.save_camera(&c)?;
        c.name = "Entrada".into();
        s.save_camera(&c)?;
        assert_eq!(s.cameras()?[0].name, "Entrada");
        let mut v = View::default();
        v.preset(2, 2, &[c.id.clone()], false);
        s.save_view(&v)?;
        assert_eq!(s.views()?[0], v);
        s.delete_camera(&c.id)?;
        assert!(s.views()?[0].panels.is_empty());
        assert!(s.cameras()?.is_empty());
        assert!(s.save_view(&v).is_err());
        let p = Preferences {
            last_view: Some(v.id.clone()),
            ..Default::default()
        };
        s.save_preferences(&p)?;
        assert_eq!(s.preferences()?.last_view, p.last_view);
        s.delete_view(&v.id)?;
        assert!(s.views()?.is_empty());
        Ok(())
    }
}
