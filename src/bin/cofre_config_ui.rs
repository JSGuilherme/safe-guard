#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

//! Configuration UI for CofreSenha API using iced.

use iced::{Alignment, Element, Length, Sandbox, Settings};
use iced::widget::{button, column, container, row, text, text_input};
use std::fs;
use std::path::PathBuf;

const CONFIG_DIR: &str = "CofreSenhaRust";
const CONFIG_FILE: &str = "config.yaml";

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
    CancelPressed,
}

pub struct ConfigApp {
    config: ApiConfig,
    message: Option<String>,
}

impl Sandbox for ConfigApp {
    type Message = Message;

    fn new() -> Self {
        let config = load_config();
        ConfigApp {
            config,
            message: None,
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
                        self.message = Some("Configuração salva com sucesso!".to_string());
                    }
                    Err(e) => {
                        self.message = Some(format!("Erro ao salvar: {}", e));
                    }
                }
            }
            Message::CancelPressed => {
                std::process::exit(0);
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let api_port_input = text_input("5474", &self.config.api_port)
            .on_input(Message::ApiPortChanged)
            .padding(10);

        let ttl_input = text_input("7200", &self.config.session_ttl_secs)
            .on_input(Message::SessionTtlChanged)
            .padding(10);

        let max_ttl_input = text_input("43200", &self.config.session_max_ttl_secs)
            .on_input(Message::SessionMaxTtlChanged)
            .padding(10);

        let save_btn = button("Salvar")
            .padding(10)
            .on_press(Message::SavePressed);

        let cancel_btn = button("Cancelar")
            .padding(10)
            .on_press(Message::CancelPressed);

        let buttons = row![save_btn, cancel_btn].spacing(10);

        let mut content = column![
            text("Configuração da API CofreSenha"),
            text("Porta da API:"),
            api_port_input,
            text("Timeout de Inatividade (segundos):"),
            ttl_input,
            text("Timeout Máximo da Sessão (segundos):"),
            max_ttl_input,
            buttons
        ]
        .spacing(10)
        .padding(20)
        .align_items(Alignment::Center);

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
    <ConfigApp as Sandbox>::run(Settings::default())
}
