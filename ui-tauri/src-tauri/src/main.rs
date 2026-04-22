use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use arboard::Clipboard;
use cofreSenhaRust::{default_vault_path, load_vault};
use serde::Serialize;

#[derive(Debug)]
struct SessionState {
    master_password: String,
    expires_at_unix: u64,
}

#[derive(Default)]
struct AppState {
    session: Mutex<Option<SessionState>>,
    ttl_secs: u64,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    now_unix: u64,
}

#[derive(Serialize)]
struct UnlockResponse {
    expires_at_unix: u64,
    ttl_secs: u64,
}

#[derive(Serialize)]
struct EntrySummary {
    id: String,
    servico: String,
    usuario: String,
    url: Option<String>,
    atualizado_em: String,
}

#[tauri::command]
fn health() -> HealthResponse {
    HealthResponse {
        status: "ok",
        now_unix: now_unix(),
    }
}

#[tauri::command]
fn unlock_vault(master_password: String, state: tauri::State<AppState>) -> Result<UnlockResponse, String> {
    if master_password.trim().is_empty() {
        return Err("Senha mestra nao informada".to_string());
    }

    let path = default_vault_path()?;
    if !path.exists() {
        return Err("Cofre nao encontrado".to_string());
    }

    load_vault(path.as_path(), master_password.as_str())?;

    let expires_at_unix = now_unix().saturating_add(state.ttl_secs);
    let mut guard = state
        .session
        .lock()
        .map_err(|_| "Falha ao atualizar sessao".to_string())?;

    *guard = Some(SessionState {
        master_password,
        expires_at_unix,
    });

    Ok(UnlockResponse {
        expires_at_unix,
        ttl_secs: state.ttl_secs,
    })
}

#[tauri::command]
fn list_entries(state: tauri::State<AppState>) -> Result<Vec<EntrySummary>, String> {
    let master_password = session_master_password(&state)?;
    let path = default_vault_path()?;
    let vault = load_vault(path.as_path(), master_password.as_str())?;

    let entries = vault
        .entries
        .into_iter()
        .map(|entry| EntrySummary {
            id: entry.id.to_string(),
            servico: entry.servico,
            usuario: entry.usuario,
            url: entry.url,
            atualizado_em: entry.atualizado_em,
        })
        .collect();

    Ok(entries)
}

#[tauri::command]
fn copy_password(entry_id: String, state: tauri::State<AppState>) -> Result<(), String> {
    let master_password = session_master_password(&state)?;
    let path = default_vault_path()?;
    let vault = load_vault(path.as_path(), master_password.as_str())?;

    let entry = vault
        .entries
        .iter()
        .find(|item| item.id.to_string() == entry_id)
        .ok_or_else(|| "Entrada nao encontrada".to_string())?;

    let mut clipboard = Clipboard::new().map_err(|_| "Falha ao acessar area de transferencia".to_string())?;
    clipboard
        .set_text(entry.senha.clone())
        .map_err(|_| "Falha ao copiar senha".to_string())
}

#[tauri::command]
fn lock_vault(state: tauri::State<AppState>) -> Result<(), String> {
    let mut guard = state
        .session
        .lock()
        .map_err(|_| "Falha ao bloquear sessao".to_string())?;

    *guard = None;
    Ok(())
}

fn session_master_password(state: &tauri::State<AppState>) -> Result<String, String> {
    let mut guard = state
        .session
        .lock()
        .map_err(|_| "Falha ao acessar sessao".to_string())?;

    let Some(session) = guard.as_ref() else {
        return Err("Sessao bloqueada".to_string());
    };

    if now_unix() >= session.expires_at_unix {
        *guard = None;
        return Err("Sessao expirada".to_string());
    }

    Ok(session.master_password.clone())
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs()
}

fn main() {
    tauri::Builder::default()
        .manage(AppState {
            session: Mutex::new(None),
            ttl_secs: 1800,
        })
        .invoke_handler(tauri::generate_handler![
            health,
            unlock_vault,
            list_entries,
            copy_password,
            lock_vault
        ])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar app tauri");
}
