#![cfg_attr(all(target_os = "windows", not(debug_assertions)), windows_subsystem = "windows")]

//! Tray app to manage cofre_api.exe as a background process.

use std::env;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tray_item::TrayItem;

const API_ARGS: &[&str] = &["--port", "5474", "--session-ttl-secs", "1800"];
const ICON_RESOURCE: &str = "COFRE_TRAY";

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

    let child = Command::new(api_exe_path)
        .args(API_ARGS)
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
