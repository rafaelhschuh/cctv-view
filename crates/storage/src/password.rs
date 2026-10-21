use super::*;
use ring::{
    pbkdf2,
    rand::{SecureRandom, SystemRandom},
};
use std::num::NonZeroU32;
#[derive(serde::Serialize, serde::Deserialize)]
struct PasswordRecord {
    version: u8,
    salt: Vec<u8>,
    hash: Vec<u8>,
}
fn iterations() -> Result<NonZeroU32> {
    NonZeroU32::new(600_000).context("Configuração KDF inválida.")
}
impl PasswordRecord {
    fn verify(&self, password: &str) -> Result<bool> {
        ensure!(
            self.version == 1 && self.salt.len() == 16 && self.hash.len() == 32,
            "Registro de senha inválido."
        );
        if password.len() > 1024 {
            return Ok(false);
        }
        Ok(pbkdf2::verify(
            pbkdf2::PBKDF2_HMAC_SHA256,
            iterations()?,
            &self.salt,
            password.as_bytes(),
            &self.hash,
        )
        .is_ok())
    }
    fn new(password: &str) -> Result<Self> {
        ensure!(
            password.chars().count() >= 8 && password.len() <= 1024,
            "Use pelo menos 8 caracteres (até 1024 bytes)."
        );
        let mut record = Self {
            version: 1,
            salt: vec![0; 16],
            hash: vec![0; 32],
        };
        SystemRandom::new()
            .fill(&mut record.salt)
            .map_err(|_| anyhow::anyhow!("Não foi possível gerar salt."))?;
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            iterations()?,
            &record.salt,
            password.as_bytes(),
            &mut record.hash,
        );
        Ok(record)
    }
}
impl Store {
    pub fn has_app_password(&self) -> Result<bool> {
        Ok(self.db.query_row(
            "SELECT EXISTS(SELECT 1 FROM settings WHERE key='app_lock')",
            [],
            |r| r.get(0),
        )?)
    }
    pub fn verify_app_password(&self, password: &str) -> Result<bool> {
        let value: Option<String> = self
            .db
            .query_row("SELECT value FROM settings WHERE key='app_lock'", [], |r| {
                r.get(0)
            })
            .optional()?;
        match value {
            Some(value) => serde_json::from_str::<PasswordRecord>(&value)?.verify(password),
            None => Ok(true),
        }
    }
    pub fn set_app_password(&mut self, current: &str, next: Option<&str>) -> Result<()> {
        let tx = self
            .db
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let old: Option<String> = tx
            .query_row("SELECT value FROM settings WHERE key='app_lock'", [], |r| {
                r.get(0)
            })
            .optional()?;
        if let Some(value) = old {
            ensure!(
                serde_json::from_str::<PasswordRecord>(&value)?.verify(current)?,
                "Senha atual incorreta."
            );
        }
        if let Some(next) = next {
            let value = serde_json::to_string(&PasswordRecord::new(next)?)?;
            tx.execute("INSERT INTO settings VALUES('app_lock',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value",[value])?;
        } else {
            tx.execute("DELETE FROM settings WHERE key='app_lock'", [])?;
        }
        tx.commit()?;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn password_change_removal_and_salts() -> Result<()> {
        let mut store = Store::memory()?;
        assert!(!store.has_app_password()?);
        store.set_app_password("", Some("initial-fixture-password"))?;
        let old: String =
            store
                .db
                .query_row("SELECT value FROM settings WHERE key='app_lock'", [], |r| {
                    r.get(0)
                })?;
        assert!(!old.contains("initial-fixture-password"));
        assert!(store.verify_app_password("initial-fixture-password")?);
        assert!(!store.verify_app_password("wrong")?);
        assert!(store.set_app_password("wrong", None).is_err());
        assert!(store.has_app_password()?);
        store.set_app_password("initial-fixture-password", Some("initial-fixture-password"))?;
        let new: String =
            store
                .db
                .query_row("SELECT value FROM settings WHERE key='app_lock'", [], |r| {
                    r.get(0)
                })?;
        assert_ne!(old, new);
        store.set_app_password("initial-fixture-password", Some("next-fixture-password"))?;
        assert!(!store.verify_app_password("initial-fixture-password")?);
        assert!(store.verify_app_password("next-fixture-password")?);
        store.set_app_password("next-fixture-password", None)?;
        assert!(!store.has_app_password()?);
        Ok(())
    }
}
