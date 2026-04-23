use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use clap::Parser;
use cofreSenhaRust::{
    NewEntry, create_new_vault, default_vault_path, load_vault, save_vault, upsert_entry,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(name = "cofre-api")]
#[command(about = "API local para integracao com extensao do navegador")]
struct ApiArgs {
    #[arg(long, default_value_t = 5474)]
    port: u16,

    #[arg(long, default_value_t = 1800)]
    session_ttl_secs: u64,
}

#[derive(Clone)]
struct AppState {
    sessions: Arc<Mutex<HashMap<String, SessionState>>>,
    session_ttl_secs: u64,
}

#[derive(Debug, Clone)]
struct SessionState {
    master_password: String,
    expires_at_unix: u64,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    now_unix: u64,
}

#[derive(Debug, Deserialize)]
struct UnlockRequest {
    master_password: String,
}

#[derive(Debug, Deserialize)]
struct CreateVaultRequest {
    master_password: String,
}

#[derive(Debug, Serialize)]
struct UnlockResponse {
    session_token: String,
    expires_at_unix: u64,
    ttl_secs: u64,
}

#[derive(Debug, Serialize)]
struct VaultStatusResponse {
    exists: bool,
}

#[derive(Debug, Deserialize)]
struct EntryUpsertRequest {
    servico: String,
    usuario: String,
    senha: String,
    url: Option<String>,
    notas: Option<String>,
}

#[derive(Debug, Deserialize)]
struct EntryEditRequest {
    servico: Option<String>,
    usuario: Option<String>,
    senha: Option<String>,
    url: Option<String>,
    notas: Option<String>,
}

#[derive(Debug, Serialize)]
struct EntryUpsertResponse {
    entry_id: String,
    created: bool,
}

#[derive(Debug, Serialize)]
struct EntrySummary {
    id: String,
    servico: String,
    usuario: String,
    url: Option<String>,
    atualizado_em: String,
}

#[derive(Debug, Serialize)]
struct ListEntriesResponse {
    entries: Vec<EntrySummary>,
}

#[derive(Debug, Serialize)]
struct PasswordResponse {
    senha: String,
}

#[derive(Debug, Serialize)]
struct NotesResponse {
    notas: Option<String>,
}

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("Erro: {err}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), String> {
    let args = ApiArgs::parse();

    let shared = AppState {
        sessions: Arc::new(Mutex::new(HashMap::new())),
        session_ttl_secs: args.session_ttl_secs,
    };

    let app = Router::new()
        .route("/api/v1/health", get(health))
        .route("/api/v1/vault", get(vault_status).post(create_vault))
        .route("/api/v1/unlock", post(unlock))
        .route("/api/v1/entries/{session_token}", get(list_entries))
        .route("/api/v1/entries/{session_token}", post(create_entry))
        .route(
            "/api/v1/entries/{session_token}/{entry_id}/password",
            get(get_entry_password),
        )
        .route(
            "/api/v1/entries/{session_token}/{entry_id}/notes",
            get(get_entry_notes),
        )
        .route(
            "/api/v1/entries/{session_token}/{entry_id}",
            put(edit_entry).delete(delete_entry),
        )
        .route("/api/v1/lock/{session_token}", post(lock_session))
        .with_state(shared);

    let addr = format!("127.0.0.1:{}", args.port);
    let listener = tokio::net::TcpListener::bind(addr.as_str())
        .await
        .map_err(|err| err.to_string())?;

    println!("Cofre API local em http://{}", addr);
    axum::serve(listener, app)
        .await
        .map_err(|err| err.to_string())
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        now_unix: now_unix(),
    })
}

async fn vault_status() -> Result<Json<VaultStatusResponse>, (StatusCode, Json<ErrorResponse>)> {
    let vault_path = default_vault_path().map_err(err_internal)?;
    Ok(Json(VaultStatusResponse {
        exists: vault_path.exists(),
    }))
}

async fn create_vault(
    State(state): State<AppState>,
    Json(payload): Json<CreateVaultRequest>,
) -> Result<(StatusCode, Json<UnlockResponse>), (StatusCode, Json<ErrorResponse>)> {
    if payload.master_password.trim().is_empty() {
        return Err(err_bad_request("Senha mestra nao informada"));
    }

    let vault_path = default_vault_path().map_err(err_internal)?;
    if vault_path.exists() {
        return Err(err_conflict("Cofre ja existe"));
    }

    create_new_vault(vault_path.as_path(), payload.master_password.as_str()).map_err(|err| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Nao foi possivel criar o cofre: {err}"),
            }),
        )
    })?;

    let session = create_session_for_master(&state, payload.master_password)?;
    Ok((StatusCode::CREATED, Json(session)))
}

async fn unlock(
    State(state): State<AppState>,
    Json(payload): Json<UnlockRequest>,
) -> Result<Json<UnlockResponse>, (StatusCode, Json<ErrorResponse>)> {
    if payload.master_password.trim().is_empty() {
        return Err(err_bad_request("Senha mestra nao informada"));
    }

    let vault_path = default_vault_path().map_err(err_internal)?;
    if !vault_path.exists() {
        return Err(err_not_found("Cofre nao encontrado"));
    }

    load_vault(vault_path.as_path(), payload.master_password.as_str()).map_err(|err| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: format!("Nao foi possivel desbloquear o cofre: {err}"),
            }),
        )
    })?;

    let session = create_session_for_master(&state, payload.master_password)?;
    Ok(Json(session))
}

async fn list_entries(
    State(state): State<AppState>,
    Path(session_token): Path<String>,
) -> Result<Json<ListEntriesResponse>, (StatusCode, Json<ErrorResponse>)> {
    let vault = read_vault_for_session(&state, session_token.as_str())?;

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

    Ok(Json(ListEntriesResponse { entries }))
}

async fn create_entry(
    State(state): State<AppState>,
    Path(session_token): Path<String>,
    Json(payload): Json<EntryUpsertRequest>,
) -> Result<(StatusCode, Json<EntryUpsertResponse>), (StatusCode, Json<ErrorResponse>)> {
    if payload.servico.trim().is_empty() {
        return Err(err_bad_request("Servico nao informado"));
    }
    if payload.usuario.trim().is_empty() {
        return Err(err_bad_request("Usuario nao informado"));
    }
    if payload.senha.trim().is_empty() {
        return Err(err_bad_request("Senha nao informada"));
    }

    let (master_password, mut vault) = read_vault_and_master_for_session(&state, session_token.as_str())?;
    let was_update = upsert_entry(
        &mut vault,
        NewEntry {
            servico: payload.servico.trim().to_string(),
            usuario: payload.usuario.trim().to_string(),
            senha: payload.senha,
            url: normalize_optional(payload.url.as_deref()),
            notas: normalize_optional(payload.notas.as_deref()),
        },
    );

    save_vault_for_session(master_password.as_str(), &vault)?;

    let entry = vault
        .entries
        .iter()
        .find(|item| item.servico == payload.servico.trim())
        .ok_or_else(|| err_internal("Falha ao localizar entrada salva".to_string()))?;

    let status = if was_update { StatusCode::OK } else { StatusCode::CREATED };
    Ok((
        status,
        Json(EntryUpsertResponse {
            entry_id: entry.id.to_string(),
            created: !was_update,
        }),
    ))
}

async fn get_entry_password(
    State(state): State<AppState>,
    Path((session_token, entry_id)): Path<(String, String)>,
) -> Result<Json<PasswordResponse>, (StatusCode, Json<ErrorResponse>)> {
    let vault = read_vault_for_session(&state, session_token.as_str())?;

    let entry = vault
        .entries
        .iter()
        .find(|item| item.id.to_string() == entry_id)
        .ok_or_else(|| err_not_found("Entrada nao encontrada"))?;

    Ok(Json(PasswordResponse {
        senha: entry.senha.clone(),
    }))
}

async fn get_entry_notes(
    State(state): State<AppState>,
    Path((session_token, entry_id)): Path<(String, String)>,
) -> Result<Json<NotesResponse>, (StatusCode, Json<ErrorResponse>)> {
    let vault = read_vault_for_session(&state, session_token.as_str())?;

    let entry = vault
        .entries
        .iter()
        .find(|item| item.id.to_string() == entry_id)
        .ok_or_else(|| err_not_found("Entrada nao encontrada"))?;

    Ok(Json(NotesResponse {
        notas: entry.notas.clone(),
    }))
}

async fn edit_entry(
    State(state): State<AppState>,
    Path((session_token, entry_id)): Path<(String, String)>,
    Json(payload): Json<EntryEditRequest>,
) -> Result<Json<EntryUpsertResponse>, (StatusCode, Json<ErrorResponse>)> {
    let (master_password, mut vault) = read_vault_and_master_for_session(&state, session_token.as_str())?;

    let entry = vault
        .entries
        .iter_mut()
        .find(|item| item.id.to_string() == entry_id)
        .ok_or_else(|| err_not_found("Entrada nao encontrada"))?;

    if let Some(servico) = payload.servico {
        if !servico.trim().is_empty() {
            entry.servico = servico.trim().to_string();
        }
    }

    if let Some(usuario) = payload.usuario {
        if !usuario.trim().is_empty() {
            entry.usuario = usuario.trim().to_string();
        }
    }

    if let Some(senha) = payload.senha {
        if !senha.trim().is_empty() {
            entry.senha = senha;
        }
    }

    if let Some(url) = payload.url {
        entry.url = normalize_optional(Some(url.as_str()));
    }

    if let Some(notas) = payload.notas {
        entry.notas = normalize_optional(Some(notas.as_str()));
    }

    let entry_id_response = entry.id.to_string();
    save_vault_for_session(master_password.as_str(), &vault)?;

    Ok(Json(EntryUpsertResponse {
        entry_id: entry_id_response,
        created: false,
    }))
}

async fn delete_entry(
    State(state): State<AppState>,
    Path((session_token, entry_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let (master_password, mut vault) = read_vault_and_master_for_session(&state, session_token.as_str())?;

    let before = vault.entries.len();
    vault.entries.retain(|entry| entry.id.to_string() != entry_id);

    if vault.entries.len() == before {
        return Err(err_not_found("Entrada nao encontrada"));
    }

    save_vault_for_session(master_password.as_str(), &vault)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn lock_session(
    State(state): State<AppState>,
    Path(session_token): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let mut guard = state
        .sessions
        .lock()
        .map_err(|_| err_internal("Falha de sincronizacao de sessao".to_string()))?;

    if guard.remove(session_token.as_str()).is_none() {
        return Err(err_not_found("Sessao nao encontrada"));
    }

    Ok(StatusCode::NO_CONTENT)
}

fn resolve_session_master_password(
    state: &AppState,
    session_token: &str,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    let mut guard = state
        .sessions
        .lock()
        .map_err(|_| err_internal("Falha de sincronizacao de sessao".to_string()))?;

    let Some(session) = guard.get(session_token) else {
        return Err(err_unauthorized("Sessao invalida"));
    };

    if is_expired(session.expires_at_unix) {
        guard.remove(session_token);
        return Err(err_unauthorized("Sessao expirada"));
    }

    Ok(session.master_password.clone())
}

fn create_session_for_master(
    state: &AppState,
    master_password: String,
) -> Result<UnlockResponse, (StatusCode, Json<ErrorResponse>)> {
    let token = format!("{}{}", Uuid::new_v4(), Uuid::new_v4());
    let expires_at_unix = now_unix().saturating_add(state.session_ttl_secs);

    let mut guard = state
        .sessions
        .lock()
        .map_err(|_| err_internal("Falha de sincronizacao de sessao".to_string()))?;

    guard.insert(
        token.clone(),
        SessionState {
            master_password,
            expires_at_unix,
        },
    );

    Ok(UnlockResponse {
        session_token: token,
        expires_at_unix,
        ttl_secs: state.session_ttl_secs,
    })
}

fn read_vault_for_session(
    state: &AppState,
    session_token: &str,
) -> Result<cofreSenhaRust::PlainVault, (StatusCode, Json<ErrorResponse>)> {
    let master_password = resolve_session_master_password(state, session_token)?;
    read_vault_with_master(master_password.as_str())
}

fn read_vault_and_master_for_session(
    state: &AppState,
    session_token: &str,
) -> Result<(String, cofreSenhaRust::PlainVault), (StatusCode, Json<ErrorResponse>)> {
    let master_password = resolve_session_master_password(state, session_token)?;
    let vault = read_vault_with_master(master_password.as_str())?;
    Ok((master_password, vault))
}

fn read_vault_with_master(
    master_password: &str,
) -> Result<cofreSenhaRust::PlainVault, (StatusCode, Json<ErrorResponse>)> {
    let vault_path = default_vault_path().map_err(err_internal)?;
    load_vault(vault_path.as_path(), master_password).map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Falha ao ler cofre: {err}"),
            }),
        )
    })
}

fn save_vault_for_session(
    master_password: &str,
    vault: &cofreSenhaRust::PlainVault,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let vault_path = default_vault_path().map_err(err_internal)?;
    save_vault(vault_path.as_path(), master_password, vault).map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Falha ao salvar cofre: {err}"),
            }),
        )
    })
}

fn normalize_optional(value: Option<&str>) -> Option<String> {
    value.and_then(|text| {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn is_expired(expires_at_unix: u64) -> bool {
    now_unix() >= expires_at_unix
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs()
}

fn err_bad_request(message: &str) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            error: message.to_string(),
        }),
    )
}

fn err_not_found(message: &str) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            error: message.to_string(),
        }),
    )
}

fn err_unauthorized(message: &str) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
            error: message.to_string(),
        }),
    )
}

fn err_conflict(message: &str) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::CONFLICT,
        Json(ErrorResponse {
            error: message.to_string(),
        }),
    )
}

fn err_internal(message: String) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse { error: message }),
    )
}
