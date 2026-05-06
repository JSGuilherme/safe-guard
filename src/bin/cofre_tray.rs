#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

//! Tray app to manage cofre_api.exe as a background process.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tray_item::TrayItem;

const ICON_RESOURCE: &str = "COFRE_TRAY";
const DEFAULT_API_PORT: &str = "5474";
const DEFAULT_SESSION_TTL_SECS: &str = "7200";
const DEFAULT_SESSION_MAX_TTL_SECS: &str = "43200";
const CONFIG_DIR: &str = "CofreSenhaRust";
const CONFIG_FILE: &str = "config.json";

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
struct ApiConfig {
    #[serde(default = "default_api_port")]
    api_port: String,
    #[serde(default = "default_session_ttl_secs")]
    session_ttl_secs: String,
    #[serde(default = "default_session_max_ttl_secs")]
    session_max_ttl_secs: String,
}

fn default_api_port() -> String {
    DEFAULT_API_PORT.to_string()
}

fn default_session_ttl_secs() -> String {
    DEFAULT_SESSION_TTL_SECS.to_string()
}

fn default_session_max_ttl_secs() -> String {
    DEFAULT_SESSION_MAX_TTL_SECS.to_string()
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            api_port: default_api_port(),
            session_ttl_secs: default_session_ttl_secs(),
            session_max_ttl_secs: default_session_max_ttl_secs(),
        }
    }
}

fn get_config_path() -> Result<PathBuf, String> {
    let local_app_data = dirs::data_local_dir()
        .ok_or_else(|| "Nao foi possivel obter pasta local de dados".to_string())?;
    Ok(local_app_data.join(CONFIG_DIR).join(CONFIG_FILE))
}

fn load_config() -> ApiConfig {
    // Tenta carregar do arquivo de configuração primeiro
    if let Ok(config_path) = get_config_path() {
        if let Ok(content) = fs::read_to_string(&config_path) {
            if let Ok(config) = serde_json::from_str::<ApiConfig>(&content) {
                return config;
            }
        }
    }

    // Fallback: carrega do .env se existir
    dotenv::dotenv().ok();
    let mut config = ApiConfig::default();

    if let Ok(port) = env::var("API_PORT") {
        config.api_port = port;
    }
    if let Ok(ttl) = env::var("SESSION_TTL_SECS") {
        config.session_ttl_secs = ttl;
    }
    if let Ok(max_ttl) = env::var("SESSION_MAX_TTL_SECS") {
        config.session_max_ttl_secs = max_ttl;
    }

    config
}

fn save_config(config: &ApiConfig) -> Result<(), String> {
    let config_path = get_config_path()?;

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }

    let json = serde_json::to_string_pretty(config).map_err(|err| err.to_string())?;
    fs::write(&config_path, json).map_err(|err| err.to_string())?;

    Ok(())
}

fn open_config_file() -> Result<(), String> {
    let config_path = get_config_path()?;

    // Se o arquivo não existe, cria um com valores padrão
    if !config_path.exists() {
        let default_config = ApiConfig::default();
        save_config(&default_config)?;
    }

    // Abre o arquivo no editor padrão
    #[cfg(target_os = "windows")]
    {
        Command::new("notepad")
            .arg(&config_path)
            .spawn()
            .map_err(|err| format!("Falha ao abrir arquivo de configuração: {err}"))?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        return Err("Abrir arquivo de configuração não é suportado nesta plataforma".to_string());
    }

    Ok(())
}

fn get_api_args() -> Vec<String> {
    let config = load_config();

    vec![
        "--port".to_string(),
        config.api_port,
        "--session-ttl-secs".to_string(),
        config.session_ttl_secs,
        "--session-max-ttl-secs".to_string(),
        config.session_max_ttl_secs,
    ]
}

fn main() {
    let api_exe_path = api_exe_path();
    let api_process: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));

    match TrayItem::new("CofreSenha API", ICON_RESOURCE) {
        Ok(mut tray) => {
            if let Err(err) = start_api(&api_process, &api_exe_path) {
                eprintln!("Erro ao iniciar API automaticamente: {err}");
            }

            let api_process_clone = api_process.clone();
            let api_exe_path_clone = api_exe_path.clone();
            tray.add_menu_item("Iniciar API", move || {
                if let Err(err) = start_api(&api_process_clone, &api_exe_path_clone) {
                    eprintln!("Erro ao iniciar API: {err}");
                }
            })
            .unwrap();

            let api_process_clone = api_process.clone();
            tray.add_menu_item("Parar API", move || {
                if let Err(err) = stop_api(&api_process_clone) {
                    eprintln!("Erro ao parar API: {err}");
                }
            })
            .unwrap();

            let api_process_clone = api_process.clone();
            let api_exe_path_clone = api_exe_path.clone();
            tray.add_menu_item("Reiniciar API", move || {
                if let Err(err) = restart_api(&api_process_clone, &api_exe_path_clone) {
                    eprintln!("Erro ao reiniciar API: {err}");
                }
            })
            .unwrap();

            tray.add_menu_item("Abrir Configuração", || {
                if let Err(err) = open_config_file() {
                    eprintln!("Erro ao abrir configuração: {err}");
                }
            })
            .unwrap();

            tray.add_menu_item("Sair", || {
                std::process::exit(0);
            })
            .unwrap();

            loop {
                thread::sleep(Duration::from_secs(60));
            }
        }
        Err(e) => {
            eprintln!("Erro ao criar TrayItem: {e:?}");
            std::process::exit(1);
        }
    }
}

fn api_exe_path() -> PathBuf {
    match env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.join("cofre_api.exe")))
    {
        Some(path) => path,
        None => PathBuf::from("cofre_api.exe"),
    }
}

fn start_api(
    api_process: &Arc<Mutex<Option<Child>>>,
    api_exe_path: &PathBuf,
) -> Result<(), String> {
    let mut proc_guard = api_process
        .lock()
        .map_err(|_| "estado interno do processo bloqueado".to_string())?;

    if let Some(child) = proc_guard.as_mut() {
        match child.try_wait() {
            Ok(None) => return Ok(()),
            Ok(Some(_)) => *proc_guard = None,
            Err(err) => return Err(format!("falha ao verificar processo atual: {err}")),
        }
    }

    if !api_exe_path.exists() {
        return Err(format!(
            "cofre_api.exe nao encontrado em {}",
            api_exe_path.display()
        ));
    }

    let api_args = get_api_args();
    let child = Command::new(api_exe_path)
        .args(&api_args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|err| format!("falha ao iniciar {}: {err}", api_exe_path.display()))?;

    *proc_guard = Some(child);
    Ok(())
}

fn stop_api(api_process: &Arc<Mutex<Option<Child>>>) -> Result<(), String> {
    let mut proc_guard = api_process
        .lock()
        .map_err(|_| "estado interno do processo bloqueado".to_string())?;

    if let Some(mut child) = proc_guard.take() {
        child
            .kill()
            .map_err(|err| format!("falha ao encerrar processo iniciado pelo tray: {err}"))?;
        let _ = child.wait();
        return Ok(());
    }

    stop_existing_api_process()
}

fn restart_api(
    api_process: &Arc<Mutex<Option<Child>>>,
    api_exe_path: &PathBuf,
) -> Result<(), String> {
    stop_api(api_process)?;
    start_api(api_process, api_exe_path)
}

#[cfg(target_os = "windows")]
fn stop_existing_api_process() -> Result<(), String> {
    let status = Command::new("taskkill")
        .args(["/IM", "cofre_api.exe", "/F", "/T"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|err| format!("falha ao executar taskkill: {err}"))?;

    if status.success() {
        Ok(())
    } else {
        Err("nenhum processo cofre_api.exe iniciado pelo tray foi encontrado".to_string())
    }
}

#[cfg(not(target_os = "windows"))]
fn stop_existing_api_process() -> Result<(), String> {
    Err("nenhum processo cofre_api iniciado pelo tray foi encontrado".to_string())
}
