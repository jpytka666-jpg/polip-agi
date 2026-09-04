/*
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: GPT-5 Codex
TIMESTAMP: 2026-09-04 14:08:55
REASON FOR CREATION: Refresh world tile health through one credential-free, read-only Darkstar GET.
==========================================
*/

export function serviceStatusView(state) {
  if (state === 'up') return { label: 'żywe', state: 'up' }
  if (state === 'down') return { label: 'brak odpowiedzi', state: 'down' }
  return { label: 'stan nieznany', state: 'unknown' }
}

export async function fetchWorldStatus(fetchImpl = globalThis.fetch) {
  const response = await fetchImpl('/v1/world/status', {
    method: 'GET',
    headers: { accept: 'application/json' },
    cache: 'no-store',
    credentials: 'omit',
  })
  if (!response.ok) throw new Error(`world status HTTP ${response.status}`)
  return response.json()
}

export function renderWorldStatus(payload, root = document) {
  root.querySelectorAll('[data-world-status]').forEach((element) => {
    const service = payload?.services?.[element.dataset.worldStatus]
    const view = serviceStatusView(service?.state)
    element.textContent = view.label
    element.dataset.state = view.state
  })
}

async function refresh() {
  try {
    renderWorldStatus(await fetchWorldStatus())
  } catch {
    renderWorldStatus({ services: {} })
  }
}

if (typeof document !== 'undefined') {
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', refresh, { once: true })
  } else {
    refresh()
  }
  globalThis.setInterval(refresh, 15_000)
}
