## Plan: Core Rust + API Local

## Status Atual (2026-04-27)

- Core compartilhado em `src/lib.rs` com criptografia, persistencia e operacoes de vault.
- CLI funcional em `src/main.rs` usando o core compartilhado.
- API local funcional em `src/bin/cofre_api.rs` com unlock, sessoes temporarias, listagem, criacao, edicao, remocao e leitura de senha/notas por id.
- Interface Tauri removida do repositorio.
- Estrategia atual: manter Rust como backend local e base de integracao para extensoes e outras interfaces.

Recomendacao: concentrar a evolucao imediata na API local, endurecendo seguranca e melhorando a superficie de integracao antes de introduzir outra camada de UI.

**Steps**
1. Endurecer a API local: autenticacao entre cliente e servico, whitelist de origem e protecao contra abuso.
2. Melhorar manejo de segredos em memoria: reduzir tempo de residencia, evitar logs sensiveis e revisar ciclo de vida da senha mestra em sessao.
3. Adicionar ergonomia de cliente: limpar clipboard automaticamente e melhorar mensagens de erro e status.
4. Consolidar empacotamento Windows da API com instalacao, atualizacao e desinstalacao previsiveis.
5. Evoluir integracao com extensao ou outra interface consumindo a API local.

**Relevant files**
- `src/lib.rs` - core de dominio, criptografia e persistencia.
- `src/main.rs` - CLI de manutencao.
- `src/bin/cofre_api.rs` - backend local HTTP para integracoes.
- `scripts/windows` e `installer/windows` - instalacao e empacotamento da API.

**Verification**
1. Validar regressao da CLI apos qualquer refactor do core.
2. Testar unlock, expiracao de sessao, CRUD e leitura de senha/notas pela API.
3. Verificar que segredos nao aparecem em logs nem mensagens de erro.
4. Rodar build release e smoke test de instalacao/execucao em Windows limpo.

**Decisions**
- Tauri foi removido do repositorio.
- Rust permanece como core, CLI e backend API local.
- Novas interfaces devem consumir a API local ou reutilizar o core sem reintroduzir Tauri por enquanto.
