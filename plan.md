## Plan: Desktop Windows para Cofre de Senhas

## Status Atual (2026-04-17)

- Fase 1 concluida parcialmente: nucleo extraido para biblioteca compartilhada em src/lib.rs e CLI refatorada para usar o core.
- Fase 2 concluida (MVP): app desktop inicial criado em src/bin/cofre_desktop.rs com fluxo bloqueado/desbloqueado.
- Fase 3 iniciada parcialmente: desktop ja consegue desbloquear cofre e listar entradas.
- Fase 4 iniciada parcialmente: desktop com CRUD basico (adicionar/atualizar/remover) e persistencia no vault.
- Tela inicial em implementacao: primeira abertura agora separa onboarding de primeiro acesso e login por cofre existente.
- UX do onboarding/login refinada parcialmente: mensagens de ajuda em contexto, navegacao entre telas e erros orientados a acao.
- Recurso de produtividade adicionado: botao Copiar senha em cada card da listagem (servico, usuario e url), com feedback visual de copia.
- Fase A da integracao com extensao iniciada: API local minima criada em src/bin/cofre_api.rs com unlock, listagem de entradas, leitura de senha por id e lock de sessao.
- Proximos passos imediatos: reforcar seguranca da API local (auth por header, whitelist de origem e rate limit), depois iniciar scaffold da extensao.

Recomendacao: usar egui/eframe (Rust puro) para a primeira versao desktop Windows, reaproveitando a logica de cofre existente. Essa abordagem minimiza superficie de ataque para um app de senhas, simplifica distribuicao (exe unico) e acelera entrega do MVP.

**Steps**
1. Fase 1 - Extrair nucleo de dominio/criptografia para biblioteca reutilizavel (cofre_core), mantendo a CLI como cliente do core. Isso reduz risco de regressao e prepara base para UI.  
2. Fase 2 - Criar app desktop (cofre_desktop) com egui/eframe e fluxo inicial de telas: bloqueado/desbloqueado (*depends on 1*).  
3. Fase 3 - Integrar leitura/escrita do vault no desktop: unlock, listagem, detalhe de item, tratamento de erro de senha invalida (*depends on 2*).  
4. Fase 4 - Implementar CRUD completo na UI (adicionar/editar/remover), com confirmacoes e estados de loading/erro (*depends on 3*).  
5. Fase 5 - Endurecimento de seguranca: limpar buffers sensiveis em memoria, evitar logs de segredo, mascarar senha por padrao, limpar clipboard automaticamente (*depends on 3; parallel with 4 parcialmente*).  
6. Fase 6 - Empacotamento Windows (build release, instalador opcional), smoke tests em maquina limpa, checklist de release (*depends on 4 and 5*).

**Relevant files**
- c:/projetos-rust/cofreSenhaRust/src/main.rs - fonte atual do fluxo CLI; deve virar cliente fino do core.
- c:/projetos-rust/cofreSenhaRust/Cargo.toml - pode evoluir para workspace e separar crates.
- c:/projetos-rust/cofreSenhaRust/crates/cofre_core/src/lib.rs - novo core (planejado).
- c:/projetos-rust/cofreSenhaRust/crates/cofre_desktop/src/main.rs - app desktop (planejado).

**Verification**
1. Validar regressao da CLI apos refactor do core (init/add/list/get/remove).
2. Testar unlock com senha correta/incorreta na UI.
3. Testar CRUD completo e persistencia no arquivo de vault.
4. Verificar que senha nao aparece em logs nem mensagens de erro.
5. Testar limpeza de clipboard apos timeout e comportamento de mascaramento.
6. Rodar build release e smoke test de instalacao/execucao em Windows limpo.

**Decisions**
- Incluido agora: recomendacao de stack, arquitetura em camadas e plano de migracao para desktop.
- Fora de escopo imediato: autologin em navegador/extensao (ficara para fase posterior).
- Decisao tecnica: priorizar seguranca e simplicidade de manutencao sobre UI web mais sofisticada.

**Further Considerations**
1. Opcional recomendado: guardar apenas metadata no keyring do Windows no MVP e deixar auto-unlock completo para etapa seguinte.
2. Se a prioridade mudar para UI mais rica, alternativa secundaria: Tauri mantendo cofre_core em Rust e API minima para frontend.
