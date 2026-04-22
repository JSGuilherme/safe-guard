import { invoke } from "@tauri-apps/api/core";

type EntrySummary = {
  id: string;
  servico: string;
  usuario: string;
  url?: string | null;
  atualizado_em: string;
};

type UnlockResponse = {
  expires_at_unix: number;
  ttl_secs: number;
};

const passwordInput = document.querySelector<HTMLInputElement>("#master-password");
const unlockButton = document.querySelector<HTMLButtonElement>("#unlock-btn");
const lockButton = document.querySelector<HTMLButtonElement>("#lock-btn");
const refreshButton = document.querySelector<HTMLButtonElement>("#refresh-btn");
const entriesEl = document.querySelector<HTMLDivElement>("#entries");
const statusEl = document.querySelector<HTMLElement>("#status");

function setStatus(message: string, kind: "ok" | "error" | "info" = "info") {
  if (!statusEl) {
    return;
  }

  statusEl.textContent = message;
  statusEl.classList.remove("ok", "error");
  if (kind === "ok") {
    statusEl.classList.add("ok");
  } else if (kind === "error") {
    statusEl.classList.add("error");
  }
}

function renderEntries(entries: EntrySummary[]) {
  if (!entriesEl) {
    return;
  }

  entriesEl.innerHTML = "";
  if (entries.length === 0) {
    entriesEl.textContent = "Cofre vazio.";
    return;
  }

  for (const entry of entries) {
    const card = document.createElement("article");
    card.className = "entry-card";

    const title = document.createElement("div");
    title.className = "entry-title";
    title.textContent = `${entry.servico} (${entry.usuario})`;

    const meta = document.createElement("div");
    meta.className = "entry-meta";
    meta.textContent = `URL: ${entry.url ?? "-"}`;

    const actions = document.createElement("div");
    actions.className = "entry-actions";

    const copyButton = document.createElement("button");
    copyButton.textContent = "Copiar senha";
    copyButton.addEventListener("click", async () => {
      try {
        await invoke("copy_password", { entryId: entry.id });
        setStatus(`Senha de '${entry.servico}' copiada.`, "ok");
      } catch (error) {
        setStatus(`Falha ao copiar senha: ${String(error)}`, "error");
      }
    });

    actions.appendChild(copyButton);
    card.appendChild(title);
    card.appendChild(meta);
    card.appendChild(actions);
    entriesEl.appendChild(card);
  }
}

async function refreshEntries() {
  try {
    const entries = await invoke<EntrySummary[]>("list_entries");
    renderEntries(entries);
    setStatus(`Entradas carregadas: ${entries.length}`, "ok");
  } catch (error) {
    setStatus(`Nao foi possivel carregar entradas: ${String(error)}`, "error");
  }
}

async function unlock() {
  if (!passwordInput) {
    return;
  }

  const masterPassword = passwordInput.value.trim();
  if (!masterPassword) {
    setStatus("Informe a senha mestra.", "error");
    return;
  }

  try {
    const response = await invoke<UnlockResponse>("unlock_vault", {
      masterPassword,
    });

    passwordInput.value = "";
    setStatus(
      `Cofre desbloqueado. Sessao valida por ${response.ttl_secs} segundos.`,
      "ok"
    );
    await refreshEntries();
  } catch (error) {
    setStatus(`Falha no desbloqueio: ${String(error)}`, "error");
  }
}

async function lock() {
  try {
    await invoke("lock_vault");
    renderEntries([]);
    setStatus("Sessao bloqueada.", "ok");
  } catch (error) {
    setStatus(`Falha ao bloquear: ${String(error)}`, "error");
  }
}

unlockButton?.addEventListener("click", () => {
  void unlock();
});

lockButton?.addEventListener("click", () => {
  void lock();
});

refreshButton?.addEventListener("click", () => {
  void refreshEntries();
});

void invoke("health")
  .then(() => {
    setStatus("Tauri conectado ao backend local.", "ok");
  })
  .catch((error) => {
    setStatus(`Falha ao inicializar backend: ${String(error)}`, "error");
  });
