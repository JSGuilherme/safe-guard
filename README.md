# CofreSenhaRust

![Rust](https://img.shields.io/badge/Rust-000000?style=flat&logo=rust&logoColor=white)
![Windows](https://img.shields.io/badge/Windows-0078D6?style=flat&logo=windows&logoColor=white)
![API Local](https://img.shields.io/badge/API-Local-2E8B57?style=flat)
![Iced UI](https://img.shields.io/badge/UI-Iced-6A5ACD?style=flat)

Aplicacao em Rust para armazenar senhas localmente com criptografia forte, incluindo core compartilhado, CLI, API local, app de tray e tela grafica de configuracao para Windows.

## Objetivo

Fluxo principal implementado:

1. Usuario cria o cofre com senha mestra.
2. Credenciais sao armazenadas de forma criptografada em disco.
3. API local permite unlock e operacoes de leitura/escrita de entradas.
4. App de tray gerencia ciclo da API (iniciar, parar, reiniciar).
5. Tela de configuracao permite ajustar porta e timeouts da API.

## Funcionalidades atuais

- Criacao de cofre com senha mestra.
- Armazenamento criptografado das entradas.
- Listagem, consulta, adicao e remocao de credenciais.
- API local com sessao e expiracao.
- Persistencia local no perfil do usuario do Windows.
- Configuracao via UI (`cofre_config_ui`) e menu de tray.

## Estrutura do projeto

- `src/lib.rs`: nucleo do cofre, criptografia e persistencia.
- `src/main.rs`: CLI principal.
- `src/bin/cofre_api.rs`: API local REST para integracao com extensoes e outras interfaces.
- `src/bin/cofre_tray.rs`: app de bandeja do Windows para gerenciar a API.
- `src/bin/cofre_config_ui.rs`: interface grafica (Iced) para editar configuracoes da API.
- `scripts/windows/*`: scripts de instalacao, desinstalacao e geracao de setup.
- `installer/windows/cofre_api.iss`: script do Inno Setup.

## Requisitos

- Rust recente instalado via rustup.
- Windows para experiencia completa de tray + API local + instalador.
- Inno Setup 6 para gerar o `Setup.exe`.

## Setup local

### 1. Verificar compilacao

```bash
cargo check --all-targets
```

### 2. Usar CLI

```bash
cargo run -- init
cargo run -- list
cargo run -- add --servico github --usuario seu_usuario
cargo run -- get --servico github
cargo run -- remove --servico github
```

### 3. Iniciar API local

```bash
cargo run --bin cofre_api
```

Com parametros opcionais:

```bash
cargo run --bin cofre_api -- --port 5474 --session-ttl-secs 7200 --session-max-ttl-secs 43200
```

### 4. Abrir tela de configuracao em desenvolvimento

```bash
cargo run --bin cofre_config_ui
```

## Sessao da API

- `--session-ttl-secs`: timeout de inatividade (padrao: `7200`, 2 horas).
- `--session-max-ttl-secs`: vida maxima absoluta da sessao (padrao: `43200`, 12 horas).
- Chamadas autenticadas renovam `expires_at_unix` ate `max_expires_at_unix`.
- `POST /api/v1/lock/{session_token}` invalida a sessao imediatamente.

## Instalacao no Windows

### Instalar API para iniciar com o Windows

- Execute `scripts\windows\install_cofre_api.cmd`.

No PowerShell (na raiz do projeto):

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\windows\install_cofre_api.ps1
```

O script:
- compila `cofre_api`, `cofre_tray` e `cofre_config_ui` em modo release,
- copia os executaveis para `%LOCALAPPDATA%\CofreSenhaRust\api`,
- registra `cofre_tray.exe` para iniciar automaticamente no logon,
- inicia o tray em segundo plano (exceto com `-DoNotStartNow`).

Parametros uteis:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\windows\install_cofre_api.ps1 -Port 5474 -SessionTtlSecs 7200 -TaskName CofreApi
```

### Desinstalar

- Execute `scripts\windows\uninstall_cofre_api.cmd`.

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\windows\uninstall_cofre_api.ps1
```

### Gerar Setup.exe da API

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\windows\build_setup.ps1 -Version 0.1.5
```

Saida esperada:

`dist\windows\CofreSenhaRustApi-Setup-0.1.5.exe`

Comportamento do instalador:
- instala `cofre_api.exe`, `cofre_tray.exe` e `cofre_config_ui.exe` em `%LOCALAPPDATA%\CofreSenhaRust\api`,
- registra `cofre_tray.exe` em `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`,
- inicia o tray app ao final da instalacao,
- remove tarefas agendadas antigas chamadas `CofreApi`, se existirem,
- encerra `cofre_tray.exe` e `cofre_api.exe` na desinstalacao.

## Endpoints atuais

- `GET /api/v1/health`
- `GET /api/v1/vault`
- `POST /api/v1/vault`
- `POST /api/v1/unlock`
- `POST /api/v1/session/{session_token}/touch`
- `PUT /api/v1/session/{session_token}/password`
- `GET /api/v1/entries/{session_token}`
- `POST /api/v1/entries/{session_token}`
- `PUT /api/v1/entries/{session_token}/{entry_id}`
- `DELETE /api/v1/entries/{session_token}/{entry_id}`
- `GET /api/v1/entries/{session_token}/{entry_id}/password`
- `GET /api/v1/entries/{session_token}/{entry_id}/notes`
- `POST /api/v1/lock/{session_token}`

Contrato detalhado: `docs\API_SPEC.md`  
Postman collection: `docs\cofre_api.postman_collection.json`

## Configuracao da API local

### Via menu do tray

A opcao "Abrir Configuracao" permite editar:
- porta da API (padrao: `5474`),
- timeout de inatividade (padrao: `7200`),
- timeout maximo da sessao (padrao: `43200`).

Arquivo de configuracao:

`%LOCALAPPDATA%\CofreSenhaRust\config.yaml`

### Via `.env` (desenvolvimento)

Crie um arquivo `.env` na raiz do projeto:

```env
API_PORT=5474
SESSION_TTL_SECS=7200
SESSION_MAX_TTL_SECS=43200
```

Veja `CONFIG.md` para mais detalhes.

## Armazenamento local

O cofre e salvo em:

`%LOCALAPPDATA%\CofreSenhaRust\vault.dat`

## Seguranca

- O cofre inteiro e criptografado em disco.
- A chave e derivada da senha mestra com Argon2.
- O arquivo usa ChaCha20-Poly1305.
- A senha mestra nao e armazenada em texto puro.

## Limitacoes atuais

- Ainda nao ha autofill em navegador neste repositorio.
- Ainda nao ha sincronizacao em nuvem.
- A API local ainda precisa de endurecimento adicional para cenarios mais amplos.

## Roadmap

1. Reforcar seguranca e estabilidade da API local.
2. Implementar limpeza de dados sensiveis em memoria.
3. Adicionar limpeza automatica da area de transferencia.
4. Evoluir integracao com extensoes e interfaces externas.

## Licenca

Sem licenca definida ainda.
