use cofreSenhaRust::{
    NewEntry, PlainVault, create_new_vault, default_vault_path, load_vault, remove_entry,
    save_vault, upsert_entry, validate_master_password, vault_exists,
};
use eframe::egui;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Cofre de Senhas",
        native_options,
        Box::new(|_cc| Ok(Box::new(CofreDesktopApp::default()))),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HomeScreen {
    Onboarding,
    Login,
    Vault,
}

struct CofreDesktopApp {
    home_screen: HomeScreen,
    onboarding_master_password: String,
    onboarding_master_password_confirm: String,
    login_password: String,
    unlocked_master: Option<String>,
    error_message: Option<String>,
    status_message: Option<String>,
    vault: Option<PlainVault>,
    form_servico: String,
    form_usuario: String,
    form_senha: String,
    form_url: String,
    form_notas: String,
    remove_servico: String,
}

impl Default for CofreDesktopApp {
    fn default() -> Self {
        let home_screen = match vault_exists() {
            Ok(true) => HomeScreen::Login,
            Ok(false) | Err(_) => HomeScreen::Onboarding,
        };

        Self {
            home_screen,
            onboarding_master_password: String::new(),
            onboarding_master_password_confirm: String::new(),
            login_password: String::new(),
            unlocked_master: None,
            error_message: None,
            status_message: None,
            vault: None,
            form_servico: String::new(),
            form_usuario: String::new(),
            form_senha: String::new(),
            form_url: String::new(),
            form_notas: String::new(),
            remove_servico: String::new(),
        }
    }
}

impl eframe::App for CofreDesktopApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Cofre de Senhas");
            ui.separator();

            match self.home_screen {
                HomeScreen::Onboarding => self.render_onboarding(ui),
                HomeScreen::Login => self.render_login(ui),
                HomeScreen::Vault => self.render_vault(ui),
            }
        });
    }
}

impl CofreDesktopApp {
    fn render_onboarding(&mut self, ui: &mut egui::Ui) {
        ui.heading("Primeiro acesso");
        ui.label("Crie sua senha mestra para proteger seu cofre local.");
        ui.label("Dica: use uma frase longa com letras, numeros e simbolos.");
        ui.colored_label(
            egui::Color32::YELLOW,
            "Importante: sem a senha mestra nao ha recuperacao dos dados.",
        );
        ui.separator();

        ui.label("Senha mestra");
        ui.add(
            egui::TextEdit::singleline(&mut self.onboarding_master_password).password(true),
        );
        ui.label("Confirmar senha mestra");
        ui.add(
            egui::TextEdit::singleline(&mut self.onboarding_master_password_confirm)
                .password(true),
        );
        ui.small("Requisito minimo: 8 caracteres.");

        if ui.button("Criar cofre").clicked() {
            self.error_message = None;
            self.status_message = None;

            let master_password = self.onboarding_master_password.trim().to_string();
            let confirmation = self.onboarding_master_password_confirm.trim().to_string();
            let exists = vault_exists().unwrap_or(false);
            let vault_path = match default_vault_path() {
                Ok(path) => path,
                Err(err) => {
                    self.error_message = Some(format!(
                        "Nao foi possivel localizar a pasta de dados local. Detalhe: {err}"
                    ));
                    return;
                }
            };

            if exists {
                self.error_message = Some(
                    "Ja existe um cofre neste computador. Use a tela de login para desbloquear."
                        .to_string(),
                );
                return;
            }

            if master_password.is_empty() || confirmation.is_empty() {
                self.error_message =
                    Some("Preencha os dois campos de senha antes de continuar.".to_string());
                return;
            }

            if master_password != confirmation {
                self.error_message = Some("As senhas nao conferem".to_string());
                return;
            }

            if let Err(err) = validate_master_password(master_password.as_str()) {
                self.error_message = Some(err);
                return;
            }

            match create_new_vault(vault_path.as_path(), master_password.as_str()) {
                Ok(()) => match load_vault_or_empty(master_password.as_str()) {
                    Ok(vault) => {
                        self.vault = Some(vault);
                        self.unlocked_master = Some(master_password);
                        self.home_screen = HomeScreen::Vault;
                        self.onboarding_master_password.clear();
                        self.onboarding_master_password_confirm.clear();
                        self.status_message = Some("Cofre criado com sucesso".to_string());
                    }
                    Err(err) => self.error_message = Some(err),
                }
                Err(err) => {
                    self.error_message = Some(format!(
                        "Falha ao criar o cofre. Verifique permissao de escrita no disco. Detalhe: {err}"
                    ));
                }
            }
        }

        if ui.button("Ja tenho um cofre").clicked() {
            self.error_message = None;
            self.status_message = None;
            self.home_screen = HomeScreen::Login;
        }

        if let Some(err) = &self.error_message {
            ui.colored_label(egui::Color32::RED, err);
        }

        if let Some(msg) = &self.status_message {
            ui.label(msg);
        }
    }

    fn render_login(&mut self, ui: &mut egui::Ui) {
        ui.heading("Entrar no cofre");
        ui.label("Digite sua senha mestra para desbloquear.");
        ui.small("Se a senha estiver incorreta, o acesso sera negado.");
        ui.separator();

        ui.label("Senha mestra");
        ui.add(egui::TextEdit::singleline(&mut self.login_password).password(true));

        if ui.button("Desbloquear").clicked() {
            self.error_message = None;
            self.status_message = None;

            if self.login_password.trim().is_empty() {
                self.error_message = Some("Informe sua senha mestra para continuar.".to_string());
                return;
            }

            match load_vault_or_error(self.login_password.as_str()) {
                Ok(vault) => {
                    self.vault = Some(vault);
                    self.unlocked_master = Some(self.login_password.clone());
                    self.home_screen = HomeScreen::Vault;
                    self.login_password.clear();
                }
                Err(err) => self.error_message = Some(err),
            }
        }

        if ui.button("Nao tenho um cofre ainda").clicked() {
            self.error_message = None;
            self.status_message = None;
            self.home_screen = HomeScreen::Onboarding;
        }

        ui.small("Esqueceu a senha? Sem a senha mestra nao e possivel recuperar o conteudo.");

        if let Some(err) = &self.error_message {
            ui.colored_label(egui::Color32::RED, err);
        }
    }

    fn render_vault(&mut self, ui: &mut egui::Ui) {
        let mut should_lock = false;
        ui.horizontal(|ui| {
            let count = self.vault.as_ref().map(|v| v.entries.len()).unwrap_or(0);
            ui.label(format!("Entradas salvas: {}", count));
            if ui.button("Bloquear").clicked() {
                should_lock = true;
            }
        });

        if should_lock {
            self.vault = None;
            self.unlocked_master = None;
            self.login_password.clear();
            self.onboarding_master_password.clear();
            self.onboarding_master_password_confirm.clear();
            self.error_message = None;
            self.status_message = None;
            self.home_screen = if vault_exists().unwrap_or(false) {
                HomeScreen::Login
            } else {
                HomeScreen::Onboarding
            };
            return;
        }

        let Some(vault) = self.vault.as_ref() else {
            return;
        };

        ui.separator();

        if vault.entries.is_empty() {
            ui.label("Seu cofre ainda esta vazio.");
        } else {
            let mut copied_feedback: Option<String> = None;

            egui::ScrollArea::vertical().show(ui, |ui| {
                for entry in &vault.entries {
                    ui.group(|ui| {
                        ui.label(format!("Servico: {}", entry.servico));
                        ui.label(format!("Usuario: {}", entry.usuario));
                        if let Some(url) = &entry.url {
                            ui.label(format!("URL: {}", url));
                        } else {
                            ui.label("URL: -");
                        }

                        ui.add_space(6.0);
                        let copy_btn = ui.add_sized(
                            [180.0, 30.0],
                            egui::Button::new("Copiar senha"),
                        );

                        if copy_btn.clicked() {
                            ui.ctx().copy_text(entry.senha.clone());
                            copied_feedback = Some(format!(
                                "Senha de '{}' copiada para a area de transferencia.",
                                entry.servico
                            ));
                        }
                    });
                }
            });

            if let Some(msg) = copied_feedback {
                self.status_message = Some(msg);
            }
        }

        ui.separator();
        ui.heading("Adicionar ou atualizar entrada");
        ui.horizontal(|ui| {
            ui.label("Servico");
            ui.text_edit_singleline(&mut self.form_servico);
        });
        ui.horizontal(|ui| {
            ui.label("Usuario");
            ui.text_edit_singleline(&mut self.form_usuario);
        });
        ui.horizontal(|ui| {
            ui.label("Senha");
            ui.add(egui::TextEdit::singleline(&mut self.form_senha).password(true));
        });
        ui.horizontal(|ui| {
            ui.label("URL");
            ui.text_edit_singleline(&mut self.form_url);
        });
        ui.horizontal(|ui| {
            ui.label("Notas");
            ui.text_edit_singleline(&mut self.form_notas);
        });

        if ui.button("Salvar entrada").clicked() {
            let result = self.save_entry();
            self.status_message = Some(match result {
                Ok(msg) => msg,
                Err(err) => err,
            });
        }

        ui.separator();
        ui.heading("Remover entrada");
        ui.horizontal(|ui| {
            ui.label("Servico");
            ui.text_edit_singleline(&mut self.remove_servico);
            if ui.button("Remover").clicked() {
                let result = self.delete_entry();
                self.status_message = Some(match result {
                    Ok(msg) => msg,
                    Err(err) => err,
                });
            }
        });

        if let Some(msg) = &self.status_message {
            ui.separator();
            ui.label(msg);
        }
    }

    fn save_entry(&mut self) -> Result<String, String> {
        if self.form_servico.trim().is_empty() {
            return Err("Servico e obrigatorio".to_string());
        }
        if self.form_usuario.trim().is_empty() {
            return Err("Usuario e obrigatorio".to_string());
        }
        if self.form_senha.trim().is_empty() {
            return Err("Senha e obrigatoria".to_string());
        }

        let vault = self
            .vault
            .as_mut()
            .ok_or_else(|| "Cofre bloqueado".to_string())?;

        let was_update = upsert_entry(
            vault,
            NewEntry {
                servico: self.form_servico.trim().to_string(),
                usuario: self.form_usuario.trim().to_string(),
                senha: self.form_senha.clone(),
                url: normalize_optional(self.form_url.as_str()),
                notas: normalize_optional(self.form_notas.as_str()),
            },
        );

        persist_vault(vault, self.unlocked_master.as_deref())?;
        self.form_senha.clear();

        if was_update {
            Ok("Entrada atualizada com sucesso".to_string())
        } else {
            Ok("Entrada adicionada com sucesso".to_string())
        }
    }

    fn delete_entry(&mut self) -> Result<String, String> {
        let servico = self.remove_servico.trim().to_string();
        if servico.is_empty() {
            return Err("Informe o servico para remover".to_string());
        }

        let vault = self
            .vault
            .as_mut()
            .ok_or_else(|| "Cofre bloqueado".to_string())?;

        if !remove_entry(vault, servico.as_str()) {
            return Err("Servico nao encontrado".to_string());
        }

        persist_vault(vault, self.unlocked_master.as_deref())?;
        self.remove_servico.clear();
        Ok("Entrada removida com sucesso".to_string())
    }
}

fn normalize_optional(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn persist_vault(vault: &PlainVault, master_password: Option<&str>) -> Result<(), String> {
    let master = master_password.ok_or_else(|| "Sessao sem senha mestra".to_string())?;
    let path = default_vault_path()?;
    save_vault(path.as_path(), master, vault)
}

fn load_vault_or_error(master_password: &str) -> Result<PlainVault, String> {
    let path = default_vault_path()?;
    if !path.exists() {
        return Err("Cofre nao encontrado. Crie um novo cofre na tela de primeiro acesso.".to_string());
    }

    match load_vault(path.as_path(), master_password) {
        Ok(vault) => Ok(vault),
        Err(err) => {
            let friendly = if err.contains("Senha mestra incorreta ou cofre corrompido") {
                "Nao foi possivel desbloquear. Verifique se a senha esta correta. Se o erro persistir, o arquivo pode estar corrompido; restaure um backup ou recrie o cofre.".to_string()
            } else if err.contains("Arquivo de cofre invalido")
                || err.contains("Conteudo do cofre invalido")
                || err.contains("Versao de cofre nao suportada")
            {
                "O arquivo do cofre parece invalido ou incompativel. Recomenda-se restaurar backup antes de criar um novo cofre.".to_string()
            } else {
                format!("Falha ao abrir o cofre. Detalhe: {err}")
            };

            Err(friendly)
        }
    }
}

fn load_vault_or_empty(master_password: &str) -> Result<PlainVault, String> {
    let path = default_vault_path()?;
    if path.exists() {
        return load_vault(path.as_path(), master_password);
    }

    Ok(PlainVault::default())
}
