use clap::{Parser, Subcommand};
use rpassword::read_password;
use cofreSenhaRust::{
    NewEntry, PlainVault, create_new_vault, default_vault_path, load_vault, remove_entry, save_vault,
    upsert_entry,
};

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
            let senha1 = read_password().map_err(|err| err.to_string())?;
            println!("Confirme a senha mestra:");
            let senha2 = read_password().map_err(|err| err.to_string())?;

            if senha1 != senha2 {
                return Err("As senhas nao conferem".to_string());
            }

            create_new_vault(vault_path.as_path(), &senha1)?;
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
            let senha = match senha {
                Some(valor) => valor,
                None => {
                    println!("Digite a senha do servico (entrada oculta):");
                    read_password().map_err(|err| err.to_string())?
                }
            };

            let was_update = upsert_entry(
                &mut vault,
                NewEntry {
                    servico,
                    usuario,
                    senha,
                    url,
                    notas,
                },
            );

            if was_update {
                println!("Servico existente atualizado");
            } else {
                println!("Servico adicionado");
            }

            save_vault(vault_path.as_path(), &master, &vault)?;
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
            if !remove_entry(&mut vault, &servico) {
                return Err("Servico nao encontrado".to_string());
            }

            save_vault(vault_path.as_path(), &master, &vault)?;
            println!("Servico removido: {servico}");
        }
    }

    Ok(())
}

fn ask_master_password() -> Result<String, String> {
    println!("Digite a senha mestra:");
    read_password().map_err(|err| err.to_string())
}

fn unlock_vault(vault_path: &std::path::PathBuf) -> Result<(PlainVault, String), String> {
    if !vault_path.exists() {
        return Err(format!(
            "Cofre nao encontrado em {}. Rode 'cofre-senhas init' primeiro.",
            vault_path.to_string_lossy()
        ));
    }

    let master = ask_master_password()?;
    let vault = load_vault(vault_path.as_path(), &master)?;
    Ok((vault, master))
}
