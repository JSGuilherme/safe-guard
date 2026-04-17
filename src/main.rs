use std::fs;
use std::io;
use std::path::PathBuf;

use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use chacha20poly1305::{
    ChaCha20Poly1305, Key, Nonce,
    aead::{Aead, KeyInit},
};
use clap::{Parser, Subcommand};
use rand::RngCore;
use rpassword::read_password;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(name = "cofre-senhas")]
#[command(about = "Cofre local de senhas com criptografia", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Inicializa o cofre com uma senha mestra
    Init,
    /// Adiciona uma senha ao cofre
    Add {
        #[arg(long)]
        servico: String,
        #[arg(long)]
        usuario: String,
        #[arg(long)]
        senha: Option<String>,
        #[arg(long)]
        url: Option<String>,
        #[arg(long)]
        notas: Option<String>,
    },
    /// Lista serviços salvos no cofre
    List,
    /// Mostra um item do cofre
    Get {
        #[arg(long)]
        servico: String,
        #[arg(long, default_value_t = false)]
        mostrar_senha: bool,
    },
    /// Remove um item do cofre
    Remove {
        #[arg(long)]
        servico: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct VaultFile {
    version: u8,
    salt_b64: String,
    nonce_b64: String,
    ciphertext_b64: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct PlainVault {
    entries: Vec<PasswordEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PasswordEntry {
    id: Uuid,
    servico: String,
    usuario: String,
    senha: String,
    url: Option<String>,
    notas: Option<String>,
    criado_em: String,
    atualizado_em: String,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Erro: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let cli = Cli::parse();
    let vault_path = default_vault_path()?;

    match cli.command {
        Commands::Init => {
            if vault_path.exists() {
                return Err(format!(
                    "Cofre ja existe em {}",
                    vault_path.to_string_lossy()
                ));
            }

            println!("Defina a senha mestra:");
            let senha1 = read_password().map_err(io_err)?;
            println!("Confirme a senha mestra:");
            let senha2 = read_password().map_err(io_err)?;

            if senha1 != senha2 {
                return Err("As senhas nao conferem".to_string());
            }

            if senha1.len() < 8 {
                return Err("A senha mestra deve ter pelo menos 8 caracteres".to_string());
            }

            let plain = PlainVault::default();
            save_vault(&vault_path, &senha1, &plain)?;
            println!("Cofre criado em {}", vault_path.to_string_lossy());
        }
        Commands::Add {
            servico,
            usuario,
            senha,
            url,
            notas,
        } => {
            let (mut vault, master) = unlock_vault(&vault_path)?;
            let now = now_iso();
            let senha = match senha {
                Some(valor) => valor,
                None => {
                    println!("Digite a senha do servico (entrada oculta):");
                    read_password().map_err(io_err)?
                }
            };

            if let Some(entry) = vault.entries.iter_mut().find(|e| e.servico == servico) {
                entry.usuario = usuario;
                entry.senha = senha;
                entry.url = url;
                entry.notas = notas;
                entry.atualizado_em = now;
                println!("Servico existente atualizado: {}", entry.servico);
            } else {
                let entry = PasswordEntry {
                    id: Uuid::new_v4(),
                    servico,
                    usuario,
                    senha,
                    url,
                    notas,
                    criado_em: now.clone(),
                    atualizado_em: now,
                };
                println!("Servico adicionado: {}", entry.servico);
                vault.entries.push(entry);
            }

            save_vault(&vault_path, &master, &vault)?;
        }
        Commands::List => {
            let (vault, _) = unlock_vault(&vault_path)?;
            if vault.entries.is_empty() {
                println!("Cofre vazio");
            } else {
                for entry in &vault.entries {
                    println!("- {} ({})", entry.servico, entry.usuario);
                }
            }
        }
        Commands::Get {
            servico,
            mostrar_senha,
        } => {
            let (vault, _) = unlock_vault(&vault_path)?;
            let Some(entry) = vault.entries.iter().find(|e| e.servico == servico) else {
                return Err("Servico nao encontrado".to_string());
            };

            println!("Servico: {}", entry.servico);
            println!("Usuario: {}", entry.usuario);
            println!("URL: {}", entry.url.as_deref().unwrap_or("-"));
            println!("Notas: {}", entry.notas.as_deref().unwrap_or("-"));
            println!("Criado em: {}", entry.criado_em);
            println!("Atualizado em: {}", entry.atualizado_em);

            if mostrar_senha {
                println!("Senha: {}", entry.senha);
            } else {
                println!("Senha: ******** (use --mostrar-senha para exibir)");
            }
        }
        Commands::Remove { servico } => {
            let (mut vault, master) = unlock_vault(&vault_path)?;
            let before = vault.entries.len();
            vault.entries.retain(|e| e.servico != servico);

            if vault.entries.len() == before {
                return Err("Servico nao encontrado".to_string());
            }

            save_vault(&vault_path, &master, &vault)?;
            println!("Servico removido: {servico}");
        }
    }

    Ok(())
}

fn default_vault_path() -> Result<PathBuf, String> {
    let base = dirs::data_local_dir()
        .ok_or_else(|| "Nao foi possivel obter pasta local de dados".to_string())?;
    Ok(base.join("CofreSenhaRust").join("vault.dat"))
}

fn now_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    secs.to_string()
}

fn ask_master_password() -> Result<String, String> {
    println!("Digite a senha mestra:");
    read_password().map_err(io_err)
}

fn unlock_vault(vault_path: &PathBuf) -> Result<(PlainVault, String), String> {
    if !vault_path.exists() {
        return Err(format!(
            "Cofre nao encontrado em {}. Rode 'cofre-senhas init' primeiro.",
            vault_path.to_string_lossy()
        ));
    }

    let master = ask_master_password()?;
    let vault = load_vault(vault_path, &master)?;
    Ok((vault, master))
}

fn save_vault(path: &PathBuf, master_password: &str, plain: &PlainVault) -> Result<(), String> {
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
    fs::write(path, encoded).map_err(io_err)
}

fn load_vault(path: &PathBuf, master_password: &str) -> Result<PlainVault, String> {
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
