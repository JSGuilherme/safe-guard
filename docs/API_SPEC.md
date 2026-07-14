# API Local v1

Este documento descreve o contrato da API local usada para integracao com extensao de navegador.

## Base URL

- http://127.0.0.1:5474

## Autenticacao

Endpoints autenticados exigem o token de sessao no header:

```
Authorization: Bearer <session_token>
```

- O token e obtido em `POST /api/v1/unlock` ou `POST /api/v1/vault`.
- Sem o header, ou com formato diferente de `Bearer <token>`, a API retorna `401`.
- O token NAO trafega mais na URL (mudanca da v0.1.5 para v0.1.6 — ver `docs/EXTENSION_MIGRATION.md`).

## Restricao de origem (anti DNS rebinding)

Todas as requisicoes passam por validacao de headers:

- `Host` precisa ser `127.0.0.1[:porta]` ou `localhost[:porta]`; caso contrario a API retorna `403`.
- `Origin`, quando presente, precisa ser `chrome-extension://`, `moz-extension://`, `safari-web-extension://` ou pagina em `http(s)://127.0.0.1`/`localhost`; caso contrario `403`.
- Requisicoes sem `Origin` (curl, apps nativos) sao aceitas.

## Sessao

- `ttl_secs` e o timeout de inatividade da sessao. Padrao: `7200` segundos, 2 horas.
- `max_ttl_secs` e a vida maxima absoluta da sessao. Padrao: `43200` segundos, 12 horas.
- `expires_at_unix` e renovado a cada chamada autenticada bem-sucedida, limitado por `max_expires_at_unix`.
- Quando `expires_at_unix` ou `max_expires_at_unix` passam, a sessao e removida e a API retorna `401`.
- `POST /api/v1/lock` remove a sessao imediatamente.

## Endpoints

### 1. Health

- Metodo: GET
- Rota: /api/v1/health
- Resposta 200:

```json
{
  "status": "ok",
  "now_unix": 1763382000
}
```

### 2. Unlock de Sessao

- Metodo: POST
- Rota: /api/v1/unlock
- Body:

```json
{
  "master_password": "senha-mestra"
}
```

- Resposta 200:

```json
{
  "session_token": "token-gerado",
  "expires_at_unix": 1763383800,
  "max_expires_at_unix": 1763425200,
  "ttl_secs": 7200,
  "max_ttl_secs": 43200
}
```

- Erros:
- 400: senha nao informada
- 401: senha incorreta/cofre invalido
- 404: cofre nao encontrado

### 3. Renovar Atividade da Sessao

- Metodo: POST
- Rota: /api/v1/session/touch
- Autenticacao: `Authorization: Bearer <token>`
- Resposta 200:

```json
{
  "expires_at_unix": 1763384700,
  "max_expires_at_unix": 1763425200,
  "ttl_secs": 7200,
  "max_ttl_secs": 43200
}
```

- Uso esperado: manter a sessao ativa quando a UI estiver em uso e obter o novo prazo de inatividade (2 horas).
- Erros:
- 401: sessao invalida ou expirada

### 4. Status do Cofre

- Metodo: GET
- Rota: /api/v1/vault
- Resposta 200:

```json
{
  "exists": true
}
```

### 5. Criar Cofre

- Metodo: POST
- Rota: /api/v1/vault
- Body:

```json
{
  "master_password": "senha-mestra"
}
```

- Resposta 201:

```json
{
  "session_token": "token-gerado",
  "expires_at_unix": 1763383800,
  "max_expires_at_unix": 1763425200,
  "ttl_secs": 7200,
  "max_ttl_secs": 43200
}
```

- Erros:
- 400: senha mestra invalida
- 409: cofre ja existe

### 6. Listagem de Entradas

- Metodo: GET
- Rota: /api/v1/entries
- Autenticacao: `Authorization: Bearer <token>`
- Resposta 200:

```json
{
  "entries": [
    {
      "id": "8ad74fca-97fc-4cff-a0d7-a381f7189b29",
      "servico": "github",
      "usuario": "joao",
      "url": "https://github.com/login",
      "atualizado_em": "1763381900"
    }
  ]
}
```

- Erros:
- 401: sessao invalida ou expirada

Observacao: chamada autenticada bem-sucedida renova `expires_at_unix`, mas esta resposta nao retorna o novo prazo. Use `/api/v1/session/touch` quando a UI precisar sincronizar o contador.

### 7. Cadastrar Chave

- Metodo: POST
- Rota: /api/v1/entries
- Autenticacao: `Authorization: Bearer <token>`
- Body:

```json
{
  "servico": "github",
  "usuario": "joao",
  "senha": "segredo",
  "url": "https://github.com/login",
  "notas": "opcional"
}
```

- Resposta 201:

```json
{
  "entry_id": "8ad74fca-97fc-4cff-a0d7-a381f7189b29",
  "created": true
}
```

- Resposta 200: quando atualiza uma chave existente com o mesmo servico.

- Erros:
- 400: servico, usuario ou senha ausentes
- 401: sessao invalida ou expirada

### 8. Editar Chave por ID

- Metodo: PUT
- Rota: /api/v1/entries/{entry_id}
- Autenticacao: `Authorization: Bearer <token>`
- Body (todos os campos sao opcionais):

```json
{
  "servico": "github",
  "usuario": "novo-usuario",
  "senha": "nova-senha",
  "url": "https://github.com/novo-login",
  "notas": "notas atualizadas"
}
```

- Resposta 200:

```json
{
  "entry_id": "8ad74fca-97fc-4cff-a0d7-a381f7189b29",
  "created": false
}
```

- Erros:
- 401: sessao invalida ou expirada
- 404: entrada nao encontrada

### 9. Excluir Chave

- Metodo: DELETE
- Rota: /api/v1/entries/{entry_id}
- Autenticacao: `Authorization: Bearer <token>`
- Resposta 204 sem body

- Erros:
- 401: sessao invalida ou expirada
- 404: entrada nao encontrada

### 10. Obter Senha por ID

- Metodo: GET
- Rota: /api/v1/entries/{entry_id}/password
- Autenticacao: `Authorization: Bearer <token>`
- Resposta 200:

```json
{
  "senha": "segredo"
}
```

- Erros:
- 401: sessao invalida ou expirada
- 404: entrada nao encontrada

### 11. Obter Notas por ID

- Metodo: GET
- Rota: /api/v1/entries/{entry_id}/notes
- Autenticacao: `Authorization: Bearer <token>`
- Resposta 200:

```json
{
  "notas": "texto opcional"
}
```

- Se a entrada nao tiver notas, retorna `"notas": null`.

- Erros:
- 401: sessao invalida ou expirada
- 404: entrada nao encontrada

### 12. Lock da Sessao

- Metodo: POST
- Rota: /api/v1/lock
- Autenticacao: `Authorization: Bearer <token>`
- Resposta 204 sem body

- Erros:
- 401: header ausente ou formato invalido
- 404: sessao nao encontrada

### 13. Trocar Senha Mestra

- Metodo: PUT
- Rota: /api/v1/session/password
- Autenticacao: `Authorization: Bearer <token>`
- Observacao: a sessao que executa a troca permanece ativa; todas as outras sessoes ativas sao invalidada e precisarao relogar.
- Body:

```json
{
  "new_master_password": "nova-senha-mestra",
  "confirm_new_master_password": "nova-senha-mestra"
}
```

- Resposta 200:

```json
{
  "session_token": "token-atual",
  "expires_at_unix": 1763384700,
  "max_expires_at_unix": 1763425200,
  "ttl_secs": 7200,
  "max_ttl_secs": 43200,
  "invalidated_sessions": 2
}
```

- Erros:
- 400: nova senha ausente, confirmacao ausente, senhas divergentes ou senha mestra nova invalida
- 401: sessao invalida ou expirada
- 404: sessao nao encontrada

## Estado atual de seguranca

- Implementado: sessao com timeout de inatividade configuravel, vida maxima absoluta, renovacao em atividade autenticada e invalidacao manual.
- Implementado: troca de senha mestra com recriptografia do cofre, mantendo ativa apenas a sessao que realizou a alteracao.
- Implementado: autenticacao por header `Authorization: Bearer` (token fora da URL).
- Implementado: validacao de `Host`/`Origin` contra DNS rebinding e sites externos.
- Implementado: escrita atomica do cofre com backup `.bak`.
- Pendente: rate limit no unlock e auditoria local de acesso (roadmap S3 no README).

## Roadmap imediato da API

1. Rate limit / backoff no `POST /api/v1/unlock` (S3).
2. Padronizar codigos de erro e schema de resposta de falha.
3. Adicionar testes de integracao dos fluxos principais.
