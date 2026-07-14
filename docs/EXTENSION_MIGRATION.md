# Guia de migracao da extensao — API v0.1.6

Instrucoes para adaptar a extensao de navegador as mudancas de seguranca S1 (validacao de Host/Origin) e S2 (token no header Authorization) da API local do CofreSenhaRust.

**As mudancas sao BREAKING**: a extensao atual vai receber `404` (rotas antigas nao existem mais) ate ser adaptada.

## Resumo das mudancas

1. **O token de sessao saiu da URL e foi para o header** `Authorization: Bearer <session_token>` em todos os endpoints autenticados.
2. **As rotas autenticadas mudaram** — o segmento `{session_token}` foi removido do path.
3. **A API agora valida `Host` e `Origin`** e responde `403` para origens nao permitidas. Origens de extensao (`chrome-extension://`, `moz-extension://`, `safari-web-extension://`) sao aceitas.

## Mapeamento de rotas (antiga -> nova)

| Antes | Depois |
|---|---|
| `POST /api/v1/session/{token}/touch` | `POST /api/v1/session/touch` |
| `PUT /api/v1/session/{token}/password` | `PUT /api/v1/session/password` |
| `GET /api/v1/entries/{token}` | `GET /api/v1/entries` |
| `POST /api/v1/entries/{token}` | `POST /api/v1/entries` |
| `PUT /api/v1/entries/{token}/{entry_id}` | `PUT /api/v1/entries/{entry_id}` |
| `DELETE /api/v1/entries/{token}/{entry_id}` | `DELETE /api/v1/entries/{entry_id}` |
| `GET /api/v1/entries/{token}/{entry_id}/password` | `GET /api/v1/entries/{entry_id}/password` |
| `GET /api/v1/entries/{token}/{entry_id}/notes` | `GET /api/v1/entries/{entry_id}/notes` |
| `POST /api/v1/lock/{token}` | `POST /api/v1/lock` |

Rotas publicas **sem mudanca**: `GET /api/v1/health`, `GET /api/v1/vault`, `POST /api/v1/vault`, `POST /api/v1/unlock`.

Os bodies de request e response **nao mudaram**. O `session_token` continua vindo no JSON de resposta do `unlock`/`create vault`.

## O que mudar no codigo da extensao

### 1. Enviar o token no header

Em toda chamada autenticada, adicionar:

```js
headers: {
  "Content-Type": "application/json",
  "Authorization": `Bearer ${sessionToken}`,
}
```

E remover o token da montagem da URL.

Exemplo — listar entradas:

```js
// ANTES
const res = await fetch(`http://127.0.0.1:${port}/api/v1/entries/${sessionToken}`);

// DEPOIS
const res = await fetch(`http://127.0.0.1:${port}/api/v1/entries`, {
  headers: { Authorization: `Bearer ${sessionToken}` },
});
```

Exemplo — criar/atualizar entrada:

```js
const res = await fetch(`http://127.0.0.1:${port}/api/v1/entries`, {
  method: "POST",
  headers: {
    "Content-Type": "application/json",
    Authorization: `Bearer ${sessionToken}`,
  },
  body: JSON.stringify({ servico, usuario, senha, url, notas }),
});
```

### 2. Fazer as chamadas a partir do background/service worker

A validacao de `Origin` aceita `chrome-extension://...` (e equivalentes de Firefox/Safari), que e o Origin enviado quando o fetch parte do **service worker/background** da extensao. Requisicoes disparadas de **content scripts** injetados em paginas usam o Origin da pagina visitada e serao rejeitadas com `403`.

- Centralizar as chamadas HTTP no background e usar mensagens (`chrome.runtime.sendMessage`) para popup/content scripts.
- Garantir no `manifest.json` a permissao de host, necessaria para evitar bloqueio por CORS (a API nao envia headers CORS):

```json
"host_permissions": ["http://127.0.0.1/*", "http://localhost/*"]
```

### 3. Tratar os novos codigos de erro

| Codigo | Situacao | Acao sugerida na extensao |
|---|---|---|
| `401` + `"Header Authorization nao informado"` | header ausente | bug na chamada — corrigir |
| `401` + `"Use o formato: Authorization: Bearer <token>"` | formato errado | bug na chamada — corrigir |
| `401` + `"Sessao invalida"` / `"Sessao expirada"` | token invalido/expirado | limpar token salvo e pedir unlock novamente |
| `403` + `"Host nao permitido"` / `"Origem nao permitida"` | chamada fora do contexto permitido | mover a chamada para o background da extensao |
| `404` em rota com token no path | extensao antiga contra API nova | migrar para as rotas novas |

## Checklist de migracao

- [ ] Remover `session_token` da montagem de todas as URLs.
- [ ] Adicionar `Authorization: Bearer <token>` em todas as chamadas autenticadas.
- [ ] Atualizar os paths conforme a tabela de mapeamento.
- [ ] Garantir que os fetches autenticados partem do background/service worker.
- [ ] Conferir `host_permissions` no manifest.
- [ ] Tratar `401` (repedir unlock) e `403` (erro de contexto) explicitamente.
- [ ] Testar fluxo completo: unlock -> list -> get password -> create -> edit -> delete -> touch -> lock.

## Teste rapido da API nova (fora da extensao)

```bash
# unlock (publico, sem mudanca)
curl -s -X POST http://127.0.0.1:5474/api/v1/unlock \
  -H "Content-Type: application/json" \
  -d '{"master_password":"<senha>"}'

# listar entradas com o token retornado
curl -s http://127.0.0.1:5474/api/v1/entries \
  -H "Authorization: Bearer <session_token>"
```

Contrato completo e atualizado: `docs/API_SPEC.md`.
