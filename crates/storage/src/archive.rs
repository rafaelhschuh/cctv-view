use super::*;
use base64::{engine::general_purpose::STANDARD, Engine};
use ring::{
    aead, pbkdf2,
    rand::{SecureRandom, SystemRandom},
};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, num::NonZeroU32};
pub const MAX_ARCHIVE_BYTES: u64 = 12 * 1024 * 1024;
const MAX_PLAINTEXT_BYTES: usize = 8 * 1024 * 1024;
const ITERATIONS: u32 = 600_000;
const AAD: &[u8] = b"CCTV View cameras v1 AES-256-GCM PBKDF2-HMAC-SHA256/600000";

// These types deliberately have no Debug. They may only be serialized inside encryption.
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CameraArchive {
    format: String,
    version: u32,
    pub folders: Vec<String>,
    pub cameras: Vec<ArchiveCamera>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArchiveCamera {
    pub camera: Camera,
    pub(crate) credentials: Option<ArchiveCredentials>,
}
impl ArchiveCamera {
    pub(crate) fn new(mut camera: Camera, credentials: Option<Credentials>) -> Self {
        camera.secret_ref = None;
        Self {
            camera,
            credentials: credentials.map(|c| ArchiveCredentials {
                username: c.username,
                password: c.password,
            }),
        }
    }
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ArchiveCredentials {
    username: String,
    password: String,
}
impl From<ArchiveCredentials> for Credentials {
    fn from(c: ArchiveCredentials) -> Self {
        Self {
            username: c.username,
            password: c.password,
        }
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    format: String,
    version: u32,
    cipher: String,
    kdf: String,
    iterations: u32,
    salt: String,
    nonce: String,
    ciphertext: String,
}
fn key(password: &str, salt: &[u8]) -> Result<aead::LessSafeKey> {
    ensure!(
        !password.is_empty() && password.len() <= 1024,
        "Informe uma senha de até 1024 bytes."
    );
    let mut bytes = [0u8; 32];
    let iterations = NonZeroU32::new(ITERATIONS).context("Configuração KDF inválida.")?;
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA256,
        iterations,
        salt,
        password.as_bytes(),
        &mut bytes,
    );
    let unbound = aead::UnboundKey::new(&aead::AES_256_GCM, &bytes)
        .map_err(|_| anyhow::anyhow!("Falha ao preparar criptografia."));
    bytes.fill(0);
    Ok(aead::LessSafeKey::new(unbound?))
}
impl CameraArchive {
    pub(crate) fn from_records(folders: Vec<String>, cameras: Vec<ArchiveCamera>) -> Self {
        Self {
            format: "cctv-view-cameras".into(),
            version: 2,
            folders,
            cameras,
        }
    }
    pub(crate) fn validate(&self) -> Result<()> {
        ensure!(
            self.format == "cctv-view-cameras" && self.version == 2,
            "Conteúdo de arquivo incompatível."
        );
        ensure!(
            self.cameras.len() <= 2000 && self.folders.len() <= 500,
            "O arquivo excede 2000 câmeras ou 500 pastas."
        );
        let mut names = HashSet::new();
        for name in &self.folders {
            ensure!(
                !name.is_empty()
                    && name.trim() == name
                    && name.len() <= 160
                    && !name.chars().any(char::is_control)
                    && names.insert(name.to_lowercase()),
                "Pasta inválida ou repetida no arquivo."
            );
        }
        for record in &self.cameras {
            let camera = &record.camera;
            camera.validate()?;
            ensure!(
                camera.secret_ref.is_none(),
                "O arquivo contém uma referência de chaveiro inválida."
            );
            ensure!(
                camera
                    .folder
                    .as_ref()
                    .is_none_or(|f| self.folders.contains(f)),
                "Câmera aponta para uma pasta ausente do arquivo."
            );
            for endpoint in [
                Some(camera.capabilities.device_url.as_str()),
                Some(camera.capabilities.media_url.as_str()),
                camera.capabilities.ptz_url.as_deref(),
            ]
            .into_iter()
            .flatten()
            .filter(|v| !v.is_empty())
            {
                let u = url::Url::parse(endpoint)
                    .map_err(|_| anyhow::anyhow!("Endpoint ONVIF inválido no arquivo."))?;
                ensure!(
                    endpoint.len() <= 4096
                        && matches!(u.scheme(), "http" | "https")
                        && u.host_str().is_some()
                        && u.username().is_empty()
                        && u.password().is_none(),
                    "Endpoint ONVIF inválido no arquivo."
                );
            }
            for profile in &camera.profiles {
                cctv_core::validate_stream(&profile.uri)?;
            }
        }
        Ok(())
    }
    pub fn encrypt(&self, password: &str) -> Result<String> {
        self.validate()?;
        let mut payload = serde_json::to_vec(self)?;
        ensure!(
            payload.len() <= MAX_PLAINTEXT_BYTES,
            "O conteúdo excede 8 MiB."
        );
        let mut salt = [0u8; 16];
        let mut nonce = [0u8; 12];
        let random = SystemRandom::new();
        random
            .fill(&mut salt)
            .and_then(|_| random.fill(&mut nonce))
            .map_err(|_| anyhow::anyhow!("Não foi possível gerar aleatoriedade para o arquivo."))?;
        // Every export has an independent random salt/key and a fresh 96-bit nonce.
        key(password, &salt)?
            .seal_in_place_append_tag(
                aead::Nonce::assume_unique_for_key(nonce),
                aead::Aad::from(AAD),
                &mut payload,
            )
            .map_err(|_| anyhow::anyhow!("Falha ao criptografar o arquivo."))?;
        Ok(serde_json::to_string_pretty(&Envelope {
            format: "cctv-view-encrypted".into(),
            version: 1,
            cipher: "AES-256-GCM".into(),
            kdf: "PBKDF2-HMAC-SHA256".into(),
            iterations: ITERATIONS,
            salt: STANDARD.encode(salt),
            nonce: STANDARD.encode(nonce),
            ciphertext: STANDARD.encode(payload),
        })?)
    }
    pub fn decrypt(data: &[u8], password: &str) -> Result<Self> {
        ensure!(
            data.len() as u64 <= MAX_ARCHIVE_BYTES,
            "O arquivo excede 12 MiB."
        );
        let envelope: Envelope = serde_json::from_slice(data)
            .map_err(|_| anyhow::anyhow!("Use um JSON criptografado exportado pelo CCTV View."))?;
        ensure!(
            envelope.format == "cctv-view-encrypted"
                && envelope.version == 1
                && envelope.cipher == "AES-256-GCM"
                && envelope.kdf == "PBKDF2-HMAC-SHA256"
                && envelope.iterations == ITERATIONS,
            "Formato ou versão de criptografia incompatível."
        );
        let invalid = || anyhow::anyhow!("Arquivo criptografado inválido.");
        let salt = STANDARD.decode(envelope.salt).map_err(|_| invalid())?;
        let nonce: [u8; 12] = STANDARD
            .decode(envelope.nonce)
            .map_err(|_| invalid())?
            .try_into()
            .map_err(|_| invalid())?;
        let mut payload = STANDARD
            .decode(envelope.ciphertext)
            .map_err(|_| invalid())?;
        ensure!(
            salt.len() == 16 && payload.len() >= 16 && payload.len() <= MAX_PLAINTEXT_BYTES + 16,
            "Arquivo criptografado inválido."
        );
        let key = key(password, &salt)?;
        let plaintext = key
            .open_in_place(
                aead::Nonce::assume_unique_for_key(nonce),
                aead::Aad::from(AAD),
                &mut payload,
            )
            .map_err(|_| anyhow::anyhow!("Senha do arquivo incorreta ou arquivo alterado."))?;
        let archive: Result<Self> = serde_json::from_slice(plaintext)
            .map_err(|_| anyhow::anyhow!("Conteúdo descriptografado inválido."));
        payload.fill(0);
        let archive = archive?;
        archive.validate()?;
        Ok(archive)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn encryption_authenticates_content_and_keeps_urls() -> Result<()> {
        let c = Camera {
            name: "Secret camera".into(),
            stream_url: "rtsp://camera.test/live?channel=1&token=fixture#part".into(),
            ..Default::default()
        };
        let archive = CameraArchive::from_records(
            vec![],
            vec![ArchiveCamera::new(
                c.clone(),
                Some(Credentials {
                    username: "alice".into(),
                    password: "private-camera-password".into(),
                }),
            )],
        );
        let a = archive.encrypt("export-password")?;
        let b = archive.encrypt("export-password")?;
        assert_ne!(a, b);
        for forbidden in [
            "Secret camera",
            "private-camera-password",
            "alice",
            "camera.test",
            "token=fixture",
        ] {
            assert!(!a.contains(forbidden));
        }
        let recovered = CameraArchive::decrypt(a.as_bytes(), "export-password")?;
        assert_eq!(recovered.cameras[0].camera, c);
        assert_eq!(
            recovered.cameras[0].credentials.as_ref().unwrap().password,
            "private-camera-password"
        );
        assert!(CameraArchive::decrypt(a.as_bytes(), "wrong").is_err());
        let mut changed: Envelope = serde_json::from_str(&a)?;
        let mut bytes = STANDARD.decode(&changed.ciphertext)?;
        bytes[0] ^= 1;
        changed.ciphertext = STANDARD.encode(bytes);
        assert!(CameraArchive::decrypt(
            serde_json::to_string(&changed)?.as_bytes(),
            "export-password"
        )
        .is_err());
        changed.iterations = 1;
        assert!(CameraArchive::decrypt(
            serde_json::to_string(&changed)?.as_bytes(),
            "export-password"
        )
        .is_err());
        assert!(archive.encrypt("").is_err());
        assert!(CameraArchive::decrypt(b"{}", "password").is_err());
        assert!(
            CameraArchive::decrypt(&vec![b' '; MAX_ARCHIVE_BYTES as usize + 1], "password")
                .is_err()
        );
        Ok(())
    }
}
