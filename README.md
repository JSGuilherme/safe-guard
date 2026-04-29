# CofreSenhaRust

Aplicacao em Rust para armazenar senhas localmente com criptografia forte, com foco no core compartilhado, CLI e API local para Windows.

O projeto esta sendo construido em fases:
- Um core em Rust com criptografia e persistencia do cofre.
- Uma CLI para operacoes rapidas de manutencao.
- Uma API local para integracao com extensoes e outras interfaces.

## Funcionalidades atuais

- Criacao de cofre com senha mestra.
- Armazenamento criptografado das entradas.
- Listagem, consulta, adicao e remocao de credenciais.
- API local para unlock, listagem, leitura e escrita de entradas.
- Persistencia local no perfil do usuario do Windows.

## Estrutura do projeto

- `src/lib.rs`: nucleo do cofre, criptografia e persistencia.
- `src/main.rs`: CLI principal.
- `src/bin/cofre_api.rs`: API local para integracao com extensao e outras interfaces.
- `src/bin/cofre_desktop.rs`: interface desktop Rust existente.
- `plan.md`: plano de evolucao do projeto.

## Requisitos

- Rust recente instalado via rustup.
- Windows para a experiencia principal de desktop e API local.

## Como executar

### Verificar compilacao

```bash
cargo check --all-targets
```

### Iniciar a CLI

```bash
cargo run -- init
cargo run -- list
cargo run -- add --servico github --usuario seu_usuario
cargo run -- get --servico github
cargo run -- remove --servico github
```

### Iniciar a API local

```bash
cargo run --bin cofre_api
```

Com parametros opcionais:

```bash
cargo run --bin cofre_api -- --port 5474 --session-ttl-secs 1800
```

### Iniciar a interface desktop Rust existente

```bash
cargo run --bin cofre_desktop
```

### Instalar API para iniciar com o Windows

Opcao mais simples:

- Execute `scripts\\windows\\install_cofre_api.cmd`.

No PowerShell, na raiz do projeto:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\windows\install_cofre_api.ps1
```

O script:
- compila `cofre_api` em modo release,
- copia o executavel para `%LOCALAPPDATA%\CofreSenhaRust\api`,
- cria ou atualiza a tarefa agendada `CofreApi`,
- configura inicio automatico no logon e reinicio automatico em falhas.

Parametros uteis:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\windows\install_cofre_api.ps1 -Port 5474 -SessionTtlSecs 1800 -TaskName CofreApi
```

Desinstalar:

- Execute `scripts\\windows\\uninstall_cofre_api.cmd`.

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\windows\uninstall_cofre_api.ps1
```

### Gerar Setup.exe da API

1. Instale o Inno Setup 6.
2. Na raiz do projeto, rode:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\windows\build_setup.ps1 -Version 0.1.0
```

O setup sera gerado em `dist\windows`.

Arquivos usados no empacotamento:
- `installer\windows\cofre_api.iss`
- `installer\windows\register_task.ps1`
- `installer\windows\unregister_task.ps1`

Comportamento do instalador:
- instala `cofre_api.exe` em `%LOCALAPPDATA%\CofreSenhaRust\api`,
- registra inicializacao automatica da API no logon,
- inicia a API apos a instalacao,
- remove a tarefa automatica na desinstalacao.

## Endpoints iniciais

- `GET /api/v1/health`
- `GET /api/v1/vault`
- `POST /api/v1/vault`
- `POST /api/v1/unlock`
- `GET /api/v1/entries/{session_token}`
- `POST /api/v1/entries/{session_token}`
- `PUT /api/v1/entries/{session_token}/{entry_id}`
- `DELETE /api/v1/entries/{session_token}/{entry_id}`
- `GET /api/v1/entries/{session_token}/{entry_id}/password`
- `GET /api/v1/entries/{session_token}/{entry_id}/notes`
- `POST /api/v1/lock/{session_token}`

## Onde o cofre fica salvo

O arquivo do cofre e gravado no diretorio local de dados do usuario, em algo como:

`%LOCALAPPDATA%\CofreSenhaRust\vault.dat`

## Seguranca

- O cofre inteiro e criptografado em disco.
- A chave e derivada da senha mestra com Argon2.
- A criptografia do arquivo usa ChaCha20-Poly1305.
- A senha mestra nao e armazenada em texto puro no arquivo.

## Limitacoes atuais

- Ainda nao ha autofill em navegador.
- Ainda nao ha gerenciador de senha com sincronizacao em nuvem.
- A API local ainda precisa de endurecimento adicional de seguranca para exposicao mais ampla.

## Roadmap

1. Reforcar seguranca e estabilidade da API local.
2. Implementar limpeza de dados sensiveis em memoria.
3. Adicionar limpeza automatica da area de transferencia.
4. Empacotar a versao Windows com instalador.
5. Evoluir a integracao com extensoes e interfaces externas.

## Licenca

Sem licenca definida ainda.
