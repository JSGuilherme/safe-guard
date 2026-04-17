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

Endpoints iniciais:

- `GET /api/v1/health`
- `POST /api/v1/unlock`
- `GET /api/v1/entries/:session_token`
- `GET /api/v1/entries/:session_token/:entry_id/password`
- `POST /api/v1/lock/:session_token`

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