#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

//! Configuration UI for CofreSenha API using iced.

use iced::{Alignment, Element, Length, Sandbox, Settings};
use iced::widget::{button, column, container, row, text, text_input};
use std::fs;
use std::process::Command;
use std::path::PathBuf;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

const CONFIG_DIR: &str = "CofreSenhaRust";
const CONFIG_FILE: &str = "config.yaml";
const APP_ICON_BYTES: &[u8] = include_bytes!("../img/logo-sg.ico");

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
    "5474".to_string()
}

fn default_session_ttl_secs() -> String {
    "7200".to_string()
}

fn default_session_max_ttl_secs() -> String {
    "43200".to_string()
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

fn get_config_path() -> PathBuf {
    dirs::data_local_dir()
        .map(|dir| dir.join(CONFIG_DIR).join(CONFIG_FILE))
        .unwrap_or_else(|| PathBuf::from(CONFIG_FILE))
}

fn load_config() -> ApiConfig {
    let config_path = get_config_path();
    if let Ok(content) = fs::read_to_string(&config_path) {
        if let Ok(config) = serde_yaml::from_str::<ApiConfig>(&content) {
            return config;
        }
    }
    ApiConfig::default()
}

fn save_config(config: &ApiConfig) -> Result<(), String> {
    let config_path = get_config_path();

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let yaml = serde_yaml::to_string(config).map_err(|e| e.to_string())?;
    fs::write(&config_path, yaml).map_err(|e| e.to_string())?;

    Ok(())
}

#[derive(Debug, Clone)]
pub enum Message {
    ApiPortChanged(String),
    SessionTtlChanged(String),
    SessionMaxTtlChanged(String),
    SavePressed,
    RestartNowPressed,
    RestartLaterPressed,
    CancelPressed,
}

pub struct ConfigApp {
    config: ApiConfig,
    message: Option<String>,
    ask_restart_after_save: bool,
}

fn config_row<'a>(label: &'a str, input: iced::widget::TextInput<'a, Message>) -> iced::widget::Row<'a, Message> {
    row![
        text(label).width(Length::FillPortion(3)),
        input.width(Length::FillPortion(2))
    ]
    .spacing(10)
    .align_items(Alignment::Center)
}

impl Sandbox for ConfigApp {
    type Message = Message;

    fn new() -> Self {
        let config = load_config();
        ConfigApp {
            config,
            message: None,
            ask_restart_after_save: false,
        }
    }

    fn title(&self) -> String {
        "Configuração - CofreSenha API".to_string()
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::ApiPortChanged(port) => {
                self.config.api_port = port;
                self.message = None;
            }
            Message::SessionTtlChanged(ttl) => {
                self.config.session_ttl_secs = ttl;
                self.message = None;
            }
            Message::SessionMaxTtlChanged(max_ttl) => {
                self.config.session_max_ttl_secs = max_ttl;
                self.message = None;
            }
            Message::SavePressed => {
                match save_config(&self.config) {
                    Ok(_) => {
                        self.message = Some("Configuração salva com sucesso. Deseja reiniciar a API agora?".to_string());
                        self.ask_restart_after_save = true;
                    }
                    Err(e) => {
                        self.message = Some(format!("Erro ao salvar: {}", e));
                        self.ask_restart_after_save = false;
                    }
                }
            }
            Message::RestartNowPressed => {
                match restart_api_now(&self.config) {
                    Ok(_) => {
                        self.message = Some("API reiniciada com sucesso.".to_string());
                    }
                    Err(e) => {
                        self.message = Some(format!("Configuração salva, mas falha ao reiniciar a API: {}", e));
                    }
                }
                self.ask_restart_after_save = false;
            }
            Message::RestartLaterPressed => {
                self.message = Some("Configuração salva. Reinicie a API manualmente para aplicar as alterações.".to_string());
                self.ask_restart_after_save = false;
            }
            Message::CancelPressed => {
                std::process::exit(0);
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let api_port_input = text_input("5474", &self.config.api_port)
            .on_input(Message::ApiPortChanged)
            .padding(8);

        let ttl_input = text_input("7200", &self.config.session_ttl_secs)
            .on_input(Message::SessionTtlChanged)
            .padding(8);

        let max_ttl_input = text_input("43200", &self.config.session_max_ttl_secs)
            .on_input(Message::SessionMaxTtlChanged)
            .padding(8);

        let save_btn = button("Salvar")
            .padding(8)
            .on_press(Message::SavePressed);

        let cancel_btn = button("Cancelar")
            .padding(8)
            .on_press(Message::CancelPressed);

        let buttons = row![save_btn, cancel_btn].spacing(10);

        if self.ask_restart_after_save {
            let restart_popup = container(
                column![
                    text("Configuração salva").size(22),
                    text("Deseja reiniciar a API agora?"),
                    row![
                        button("Reiniciar agora")
                            .padding(8)
                            .on_press(Message::RestartNowPressed),
                        button("Mais tarde")
                            .padding(8)
                            .on_press(Message::RestartLaterPressed)
                    ]
                    .spacing(10)
                ]
                .spacing(12)
                .align_items(Alignment::Center),
            )
            .padding(20)
            .max_width(360);

            return container(restart_popup)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y()
                .into();
        }

        let mut content = column![
            text("Configuração da API").size(24),
            text("Ajuste porta e tempo de sessão.").size(14),
            config_row("Porta da API", api_port_input),
            config_row("Timeout inatividade (s)", ttl_input),
            config_row("Timeout máximo sessão (s)", max_ttl_input),
            buttons
        ]
        .spacing(12)
        .padding(16)
        .max_width(460)
        .align_items(Alignment::Start);

        if let Some(msg) = &self.message {
            content = content.push(text(msg.clone()));
        }

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

pub fn main() -> iced::Result {
    let window_icon = iced::window::icon::from_file_data(APP_ICON_BYTES, None).ok();

    <ConfigApp as Sandbox>::run(Settings {
        window: iced::window::Settings {
            size: iced::Size::new(520.0, 320.0),
            resizable: false,
            icon: window_icon,
            ..Default::default()
        },
        ..Settings::default()
    })
}

fn restart_api_now(config: &ApiConfig) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let api_exe = api_exe_path();

        if !api_exe.exists() {
            return Err(format!("cofre_api.exe nao encontrado em {}", api_exe.display()));
        }

        let mut taskkill = Command::new("taskkill");
        taskkill
            .args(["/IM", "cofre_api.exe", "/F", "/T"])
            .creation_flags(CREATE_NO_WINDOW);
        let _ = taskkill.output();

        let api_args = vec![
            "--port".to_string(),
            config.api_port.clone(),
            "--session-ttl-secs".to_string(),
            config.session_ttl_secs.clone(),
            "--session-max-ttl-secs".to_string(),
            config.session_max_ttl_secs.clone(),
        ];

        let mut api_command = Command::new(api_exe);
        api_command
            .args(&api_args)
            .creation_flags(CREATE_NO_WINDOW);

        api_command
            .spawn()
            .map_err(|err| format!("Falha ao reiniciar a API: {err}"))?;

        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = config;
        Err("Reiniciar a API nesta plataforma não é suportado".to_string())
    }
}

fn api_exe_path() -> PathBuf {
    match std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.join("cofre_api.exe")))
    {
        Some(path) => path,
        None => PathBuf::from("cofre_api.exe"),
    }
}
