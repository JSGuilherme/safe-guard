use std::fs;
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};

use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use chacha20poly1305::{
    ChaCha20Poly1305, Key, Nonce,
    aead::{Aead, KeyInit},
};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const APP_DIR_NAME: &str = "CofreSenhaRust";
pub const VAULT_FILE_NAME: &str = "vault.dat";

#[derive(Debug, Serialize, Deserialize)]
struct VaultFile {
    version: u8,
    salt_b64: String,
    nonce_b64: String,
    ciphertext_b64: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PlainVault {
    pub entries: Vec<PasswordEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordEntry {
    pub id: Uuid,
    pub servico: String,
    pub usuario: String,
    pub senha: String,
    pub url: Option<String>,
    pub notas: Option<String>,
    pub criado_em: String,
    pub atualizado_em: String,
}

#[derive(Debug, Clone)]
pub struct NewEntry {
    pub servico: String,
    pub usuario: String,
    pub senha: String,
    pub url: Option<String>,
    pub notas: Option<String>,
}

pub fn default_vault_path() -> Result<PathBuf, String> {
    let base = dirs::data_local_dir()
        .ok_or_else(|| "Nao foi possivel obter pasta local de dados".to_string())?;
    Ok(base.join(APP_DIR_NAME).join(VAULT_FILE_NAME))
}

pub fn vault_exists() -> Result<bool, String> {
    Ok(default_vault_path()?.exists())
}

pub fn validate_master_password(master_password: &str) -> Result<(), String> {
    if master_password.len() < 8 {
        return Err("A senha mestra deve ter pelo menos 8 caracteres".to_string());
    }

    Ok(())
}

pub fn now_epoch_secs() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    secs.to_string()
}

pub fn create_new_vault(path: &Path, master_password: &str) -> Result<(), String> {
    validate_master_password(master_password)?;

    let plain = PlainVault::default();
    save_vault(path, master_password, &plain)
}

pub fn save_vault(path: &Path, master_password: &str, plain: &PlainVault) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(io_err)?;
    }

    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);
    let key = derive_key(master_password, &salt)?;

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);

    let cipher = ChaCha20Poly1305::new(Key::from_slice(&key));
    let plaintext = serde_json::to_vec(plain).map_err(serde_err)?;
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce_bytes), plaintext.as_ref())
        .map_err(|_| "Falha ao criptografar cofre".to_string())?;

    let file = VaultFile {
        version: 1,
        salt_b64: STANDARD.encode(salt),
        nonce_b64: STANDARD.encode(nonce_bytes),
        ciphertext_b64: STANDARD.encode(ciphertext),
    };

    let encoded = serde_json::to_vec_pretty(&file).map_err(serde_err)?;
    write_vault_file_atomic(path, &encoded)
}

/// Grava o cofre de forma atomica: escreve em arquivo temporario com sync,
/// preserva a versao anterior em `.bak` e substitui o original via rename.
/// Um crash no meio da gravacao nunca deixa o `vault.dat` corrompido.
fn write_vault_file_atomic(path: &Path, encoded: &[u8]) -> Result<(), String> {
    let tmp_path = sibling_path(path, ".tmp")?;

    {
        let mut tmp_file = fs::File::create(&tmp_path).map_err(io_err)?;
        tmp_file.write_all(encoded).map_err(io_err)?;
        tmp_file.sync_all().map_err(io_err)?;
    }

    if path.exists() {
        let bak_path = sibling_path(path, ".bak")?;
        if let Err(err) = fs::copy(path, &bak_path) {
            let _ = fs::remove_file(&tmp_path);
            return Err(io_err(err));
        }
    }

    if let Err(err) = fs::rename(&tmp_path, path) {
        let _ = fs::remove_file(&tmp_path);
        return Err(io_err(err));
    }

    Ok(())
}

pub fn backup_vault_path(path: &Path) -> Result<PathBuf, String> {
    sibling_path(path, ".bak")
}

fn sibling_path(path: &Path, suffix: &str) -> Result<PathBuf, String> {
    let file_name = path
        .file_name()
        .ok_or_else(|| "Caminho de cofre invalido".to_string())?;
    let mut name = file_name.to_os_string();
    name.push(suffix);
    Ok(path.with_file_name(name))
}

pub fn load_vault(path: &Path, master_password: &str) -> Result<PlainVault, String> {
    let encoded = fs::read(path).map_err(io_err)?;
    let file: VaultFile = serde_json::from_slice(&encoded)
        .map_err(|_| "Arquivo de cofre invalido ou corrompido".to_string())?;

    if file.version != 1 {
        return Err("Versao de cofre nao suportada".to_string());
    }

    let salt = STANDARD
        .decode(file.salt_b64)
        .map_err(|_| "Salt invalido".to_string())?;
    let nonce = STANDARD
        .decode(file.nonce_b64)
        .map_err(|_| "Nonce invalido".to_string())?;
    let ciphertext = STANDARD
        .decode(file.ciphertext_b64)
        .map_err(|_| "Conteudo criptografado invalido".to_string())?;

    if salt.len() != 16 || nonce.len() != 12 {
        return Err("Arquivo de cofre com parametros invalidos".to_string());
    }

    let key = derive_key(master_password, &salt)?;
    let cipher = ChaCha20Poly1305::new(Key::from_slice(&key));
    let plaintext = cipher
        .decrypt(Nonce::from_slice(&nonce), ciphertext.as_ref())
        .map_err(|_| "Senha mestra incorreta ou cofre corrompido".to_string())?;

    serde_json::from_slice(&plaintext).map_err(|_| "Conteudo do cofre invalido".to_string())
}

pub fn upsert_entry(vault: &mut PlainVault, new_entry: NewEntry) -> bool {
    let now = now_epoch_secs();
    if let Some(entry) = vault
        .entries
        .iter_mut()
        .find(|entry| entry.servico == new_entry.servico)
    {
        entry.usuario = new_entry.usuario;
        entry.senha = new_entry.senha;
        entry.url = new_entry.url;
        entry.notas = new_entry.notas;
        entry.atualizado_em = now;
        true
    } else {
        vault.entries.push(PasswordEntry {
            id: Uuid::new_v4(),
            servico: new_entry.servico,
            usuario: new_entry.usuario,
            senha: new_entry.senha,
            url: new_entry.url,
            notas: new_entry.notas,
            criado_em: now.clone(),
            atualizado_em: now,
        });
        false
    }
}

pub fn remove_entry(vault: &mut PlainVault, servico: &str) -> bool {
    let before = vault.entries.len();
    vault.entries.retain(|entry| entry.servico != servico);
    before != vault.entries.len()
}

fn derive_key(master_password: &str, salt: &[u8]) -> Result<[u8; 32], String> {
    let salt_string = SaltString::encode_b64(salt).map_err(|_| "Salt invalido".to_string())?;
    let password_hash = Argon2::default()
        .hash_password(master_password.as_bytes(), &salt_string)
        .map_err(|_| "Falha ao derivar chave".to_string())?;

    let mut key = [0u8; 32];
    let hash = password_hash
        .hash
        .ok_or_else(|| "Hash sem bytes".to_string())?;
    let hash_bytes = hash.as_bytes();

    if hash_bytes.len() < 32 {
        return Err("Hash derivado insuficiente".to_string());
    }

    key.copy_from_slice(&hash_bytes[..32]);
    Ok(key)
}

fn io_err(err: io::Error) -> String {
    err.to_string()
}

fn serde_err(err: serde_json::Error) -> String {
    err.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    const MASTER: &str = "senha-mestra-de-teste";

    struct TempVaultDir {
        dir: PathBuf,
    }

    impl TempVaultDir {
        fn new() -> Self {
            let dir = std::env::temp_dir()
                .join("cofre_senha_rust_tests")
                .join(Uuid::new_v4().to_string());
            fs::create_dir_all(&dir).expect("criar pasta temporaria de teste");
            Self { dir }
        }

        fn vault_path(&self) -> PathBuf {
            self.dir.join(VAULT_FILE_NAME)
        }
    }

    impl Drop for TempVaultDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.dir);
        }
    }

    fn entry(servico: &str, senha: &str) -> NewEntry {
        NewEntry {
            servico: servico.to_string(),
            usuario: "usuario".to_string(),
            senha: senha.to_string(),
            url: None,
            notas: None,
        }
    }

    #[test]
    fn save_e_load_roundtrip() {
        let tmp = TempVaultDir::new();
        let path = tmp.vault_path();

        let mut vault = PlainVault::default();
        upsert_entry(&mut vault, entry("github", "s3nh4"));
        save_vault(&path, MASTER, &vault).expect("salvar cofre");

        let loaded = load_vault(&path, MASTER).expect("carregar cofre");
        assert_eq!(loaded.entries.len(), 1);
        assert_eq!(loaded.entries[0].servico, "github");
        assert_eq!(loaded.entries[0].senha, "s3nh4");
    }

    #[test]
    fn load_com_senha_errada_falha() {
        let tmp = TempVaultDir::new();
        let path = tmp.vault_path();

        save_vault(&path, MASTER, &PlainVault::default()).expect("salvar cofre");
        assert!(load_vault(&path, "senha-errada").is_err());
    }

    #[test]
    fn salvar_sobre_existente_gera_backup_da_versao_anterior() {
        let tmp = TempVaultDir::new();
        let path = tmp.vault_path();

        let mut vault = PlainVault::default();
        upsert_entry(&mut vault, entry("github", "antiga"));
        save_vault(&path, MASTER, &vault).expect("salvar primeira versao");

        upsert_entry(&mut vault, entry("github", "nova"));
        save_vault(&path, MASTER, &vault).expect("salvar segunda versao");

        let atual = load_vault(&path, MASTER).expect("carregar versao atual");
        assert_eq!(atual.entries[0].senha, "nova");

        let bak_path = backup_vault_path(&path).expect("caminho do backup");
        let backup = load_vault(&bak_path, MASTER).expect("carregar backup");
        assert_eq!(backup.entries[0].senha, "antiga");
    }

    #[test]
    fn primeira_gravacao_nao_cria_backup_nem_deixa_temporario() {
        let tmp = TempVaultDir::new();
        let path = tmp.vault_path();

        save_vault(&path, MASTER, &PlainVault::default()).expect("salvar cofre");

        assert!(path.exists());
        assert!(!backup_vault_path(&path).unwrap().exists());
        assert!(!sibling_path(&path, ".tmp").unwrap().exists());
    }

    #[test]
    fn upsert_atualiza_servico_existente_e_remove_funciona() {
        let mut vault = PlainVault::default();

        assert!(!upsert_entry(&mut vault, entry("github", "a")));
        assert!(upsert_entry(&mut vault, entry("github", "b")));
        assert_eq!(vault.entries.len(), 1);
        assert_eq!(vault.entries[0].senha, "b");

        assert!(remove_entry(&mut vault, "github"));
        assert!(!remove_entry(&mut vault, "github"));
        assert!(vault.entries.is_empty());
    }
}
