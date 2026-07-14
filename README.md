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
- Endpoints autenticados exigem o header `Authorization: Bearer <session_token>`.
- `POST /api/v1/lock` invalida a sessao imediatamente.

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

Publicos:

- `GET /api/v1/health`
- `GET /api/v1/vault`
- `POST /api/v1/vault`
- `POST /api/v1/unlock`

Autenticados (header `Authorization: Bearer <session_token>`):

- `POST /api/v1/session/touch`
- `PUT /api/v1/session/password`
- `GET /api/v1/entries`
- `POST /api/v1/entries`
- `PUT /api/v1/entries/{entry_id}`
- `DELETE /api/v1/entries/{entry_id}`
- `GET /api/v1/entries/{entry_id}/password`
- `GET /api/v1/entries/{entry_id}/notes`
- `POST /api/v1/lock`

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

A cada gravacao, a versao anterior e preservada em `vault.dat.bak` na mesma pasta. Se o cofre principal ficar inacessivel, renomeie o `.bak` para `vault.dat` para restaurar o estado anterior a ultima alteracao.

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

Itens levantados na revisao de seguranca e desempenho de 2026-07-13. Marcar com `[x]` conforme forem resolvidos.

### Seguranca — Critico / Alto

- [x] **S1. Protecao contra DNS rebinding na API** — resolvido: middleware `enforce_local_origin` rejeita com `403` requisicoes cujo `Host` nao seja `127.0.0.1`/`localhost` e origens que nao sejam extensao de navegador ou pagina local. (`src/bin/cofre_api.rs`)
- [x] **S2. Mover token de sessao da URL para header** — resolvido: todas as rotas autenticadas usam `Authorization: Bearer <token>` via extractor `SessionToken`; o token nao aparece mais no path. Guia para adaptar a extensao: `docs/EXTENSION_MIGRATION.md`. (`src/bin/cofre_api.rs`)
- [ ] **S3. Rate limiting / backoff no `/unlock`** — sem limite de tentativas, permite brute force da senha mestra. Acao: backoff exponencial apos N falhas consecutivas. (`src/bin/cofre_api.rs:260`)
- [ ] **S4. Zerar segredos na memoria** — `SessionState` guarda a senha mestra em `String` plana por ate 12h, clonada a cada requisicao; `#[derive(Debug)]` em tipos sensiveis pode vaza-la em logs. Acao: usar `zeroize`/`secrecy`, armazenar chave derivada em vez da senha, remover `Debug` de tipos sensiveis. (`src/bin/cofre_api.rs:46-50`, `src/lib.rs:35-44`)
- [x] **S5. Escrita atomica do cofre + backup** — `fs::write` sobrescrevia `vault.dat` direto; crash no meio da escrita corrompia o arquivo. Resolvido: gravacao em arquivo temporario com sync + `rename` atomico, e a versao anterior e preservada em `vault.dat.bak`. (`src/lib.rs`, `write_vault_file_atomic`)

### Seguranca — Medio

- [ ] **S6. Lock contra escritas concorrentes** — handlers fazem ler-modificar-gravar sem sincronizacao; requisicoes simultaneas (ou API + CLI) perdem atualizacoes. Acao: `Mutex` global nas operacoes de escrita da API; avaliar lock de arquivo para CLI+API.
- [ ] **S7. Limpeza periodica de sessoes expiradas** — sessoes so sao removidas quando alguem tenta usa-las; o `HashMap` cresce com senhas mestras dentro. Acao: task periodica de limpeza. (`src/bin/cofre_api.rs:551-556`)
- [ ] **S8. Fortalecer politica de senha mestra** — minimo atual de 8 caracteres, sem outros criterios. Acao: minimo 12+; considerar checagem contra senhas comuns. (`src/lib.rs:65-71`)
- [ ] **S9. Substituir dependencias abandonadas (RUSTSEC)** — `serde_yaml` (RUSTSEC-2024-0320) e `dotenv` (RUSTSEC-2021-0141). Acao: migrar para `serde_yml` (ou JSON) e `dotenvy`; adicionar `cargo audit` ao fluxo. (`Cargo.toml`)
- [ ] **S10. Parar a API pelo PID, nao por nome de imagem** — `taskkill /IM cofre_api.exe /F /T` mata qualquer instancia, inclusive de outros usuarios. Acao: usar o PID do processo filho iniciado pelo tray/config UI. (`src/bin/cofre_tray.rs:313-318`, `src/bin/cofre_config_ui.rs:272-276`)

### Seguranca — Menor

- [ ] **S11. Uniformizar mensagens de erro do `/unlock`** — nao distinguir "cofre nao existe" de "senha incorreta".
- [ ] **S12. Endurecer parametros do Argon2 e grava-los no arquivo do cofre** — hoje usa `Argon2::default()` (19 MiB, t=2) com parametros implicitos. Acao: subir para ~64 MiB / t=3 e serializar os parametros do KDF no `VaultFile`. (`src/lib.rs:184-202`)
- [ ] **S13. Logging em arquivo para a API** — com `windows_subsystem = "windows"`, todo `eprintln!` e descartado; falhas ficam invisiveis. (`src/bin/cofre_api.rs`)

### Desempenho

- [ ] **P1. Eliminar re-derivacao Argon2 e releitura do disco por requisicao** *(maior impacto)* — cada requisicao refaz o KDF (~50-100 ms) e re-decripta o arquivo; cada `save_vault` gera salt novo forcando nova derivacao. Acao: cachear o cofre decriptado na sessao (invalidar por mtime); guardar chave derivada + salt na sessao e reutilizar o salt na gravacao (o nonce continua aleatorio); mover o Argon2 para `spawn_blocking` (hoje bloqueia o executor do tokio). (`src/bin/cofre_api.rs:628-643`, `src/lib.rs:89-116`)
- [ ] **P2. Comparar UUIDs sem alocar strings** — `item.id.to_string() == entry_id` aloca uma `String` por entrada em cada busca. Acao: usar `Path<Uuid>` do axum e comparar `Uuid == Uuid`. (`src/bin/cofre_api.rs:420,456,504`)
- [ ] **P3. `upsert_entry` retornar o `Uuid`** — `create_entry` varre o vetor de novo apos o upsert so para achar o ID. (`src/lib.rs:150-176`, `src/bin/cofre_api.rs:391-395`)
- [ ] **P4. Perfil de release otimizado** — adicionar `[profile.release]` com `lto = true`, `codegen-units = 1`, `strip = true`; tornar a feature `debug` do iced exclusiva de dev. (`Cargo.toml`)
- [ ] **P5. Tray: substituir sleep-loop por `thread::park()` e monitorar o processo filho** — usar `try_wait()` para detectar se a API morreu e reinicia-la. (`src/bin/cofre_tray.rs:231-233`)

### Qualidade geral

- [ ] **Q1. Adicionar testes** — nao ha nenhum teste. Prioridade: roundtrip `save_vault`/`load_vault`, rejeicao de senha errada, `upsert_entry`/`remove_entry`, compatibilidade do formato do arquivo.
- [ ] **Q2. Renomear pacote para `cofre_senha_rust`** — convencao snake_case; elimina warnings.
- [ ] **Q3. Timestamps como `u64` (ou RFC 3339) em vez de `String`.** (`src/lib.rs:73-80`)

### Ordem sugerida de execucao

1. **S5** (escrita atomica — risco de perda de dados) e **Q1** (testes, para dar seguranca as demais mudancas)
2. **S1 + S2** (Host check + token no header — mudam o contrato da API junto com a extensao)
3. **P1** (cache de sessao/chave — maior ganho de desempenho)
4. **S3, S4, S6, S7** (endurecimento da API)
5. Restante (S8-S13, P2-P5, Q2-Q3)

### Itens futuros (ja planejados)

- Autofill em navegador via extensao.
- Limpeza automatica da area de transferencia.
- Sincronizacao em nuvem.

## Licenca

Sem licenca definida ainda.
