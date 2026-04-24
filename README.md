# CofreSenhaRust

Aplicacao desktop em Rust para armazenar senhas localmente com criptografia forte e interface para Windows.

O projeto esta sendo construido em fases:
- Um core em Rust com criptografia e persistencia do cofre.
- Uma CLI para operacoes rapidas de manutencao.
- Uma interface desktop com egui/eframe para uso diario.

## Funcionalidades atuais

- Criacao de cofre com senha mestra.
- Armazenamento criptografado das entradas.
- Listagem, consulta, adicao e remocao de credenciais.
- Interface desktop inicial com fluxo de primeiro acesso e login.
- Persistencia local no perfil do usuario do Windows.

## Estrutura do projeto

- `src/lib.rs`: nucleo do cofre, criptografia e persistencia.
- `src/main.rs`: CLI principal.
- `src/bin/cofre_desktop.rs`: interface desktop inicial.
- `src/bin/cofre_api.rs`: API local inicial para integracao com extensao.
- `plan.md`: plano de evolucao do projeto.

## Requisitos

- Rust recente instalado via rustup.
- Windows para a experiencia principal de desktop.

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

### Iniciar a interface desktop

```bash
cargo run --bin cofre_desktop
```

### Iniciar a API local (fase inicial da extensao)

```bash
cargo run --bin cofre_api
```

Com parametros opcionais:

```bash
cargo run --bin cofre_api -- --port 5474 --session-ttl-secs 1800
```

### Instalar API para iniciar com o Windows (facil para usuario)

Opcao mais simples (duplo clique):

- Execute `scripts\\windows\\install_cofre_api.cmd`.

No PowerShell (executar como Administrador), rode na raiz do projeto:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\windows\install_cofre_api.ps1
```

O script:
- compila `cofre_api` em modo release,
- copia o executavel para `%LOCALAPPDATA%\CofreSenhaRust\api`,
- cria/atualiza a tarefa agendada `CofreApi`,
- configura inicio automatico no logon e reinicio automatico em falhas.

Parametros uteis:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\windows\install_cofre_api.ps1 -Port 5474 -SessionTtlSecs 1800 -TaskName CofreApi
```

Desinstalar:

Opcao por duplo clique:

- Execute `scripts\\windows\\uninstall_cofre_api.cmd`.

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\windows\uninstall_cofre_api.ps1
```

### Gerar Setup.exe da API (distribuicao para usuario final)

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

Endpoints iniciais:

- `GET /api/v1/health`
- `GET /api/v1/vault`
- `POST /api/v1/vault`
- `POST /api/v1/unlock`
- `GET /api/v1/entries/{session_token}`
- `POST /api/v1/entries/{session_token}`
- `DELETE /api/v1/entries/{session_token}/{entry_id}`
- `GET /api/v1/entries/{session_token}/{entry_id}/password`
- `POST /api/v1/lock/{session_token}`

## Fluxo da tela inicial

Ao abrir a interface desktop, a aplicacao verifica se o cofre ja existe:

- Se nao existir, mostra a tela de cadastro inicial da senha mestra.
- Se existir, mostra a tela de login para desbloquear o cofre.

Isso evita tentar descriptografar o cofre antes da hora e deixa o primeiro acesso mais claro para o usuario.

## Onde o cofre fica salvo

O arquivo do cofre e gravado no diretorio local de dados do usuario, em algo como:

`%LOCALAPPDATA%\CofreSenhaRust\vault.dat`

## Seguranca

- O cofre inteiro e criptografado em disco.
- A chave e derivada da senha mestra com Argon2.
- A criptografia do arquivo usa ChaCha20-Poly1305.
- A senha mestra nao e armazenada em texto puro no arquivo.

## Limitações atuais

- Ainda nao ha autofill em navegador.
- Ainda nao ha gerenciador de senha com sincronizacao em nuvem.
- A interface desktop esta em evolucao e ainda pode mudar de layout.

## Roadmap

1. Refinar a tela inicial e o onboarding.
2. Adicionar tela de detalhe e edicao de entradas.
3. Implementar limpeza de dados sensiveis em memoria.
4. Adicionar limpeza automatica da area de transferencia.
5. Empacotar a versao Windows com instalador.
6. Evoluir para autofill em aplicacoes web.

## Licenca

Sem licenca definida ainda.