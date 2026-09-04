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
  if (state === 'up') return { label: 'up', state: 'up' }
  return { label: 'down', state: 'down' }
}

export async function fetchWorldStatus(fetchImpl = globalThis.fetch, timeoutMs = 5_000) {
  const controller = new AbortController()
  let timeoutId
  const timeout = new Promise((_, reject) => {
    timeoutId = globalThis.setTimeout(() => {
      controller.abort()
      reject(new Error('world status timeout'))
    }, timeoutMs)
  })

  try {
    const response = await Promise.race([
      fetchImpl('/v1/world/status', {
        method: 'GET',
        headers: { accept: 'application/json' },
        cache: 'no-store',
        credentials: 'omit',
        signal: controller.signal,
      }),
      timeout,
    ])
    if (!response.ok) throw new Error(`world status HTTP ${response.status}`)
    return response.json()
  } finally {
    globalThis.clearTimeout(timeoutId)
  }
}

export function renderWorldStatus(payload, root = document) {
  root.querySelectorAll('[data-world-status]').forEach((element) => {
    const service = payload?.services?.[element.dataset.worldStatus]
    const view = serviceStatusView(service?.state)
    element.textContent = view.label
    element.dataset.state = view.state
  })
}

export async function refreshWorldStatus(
  fetchImpl = globalThis.fetch,
  root = globalThis.document,
  timeoutMs = 5_000,
) {
  // Brak pomiaru jest stanem negatywnym, nigdy nieskonczonym napisem "pomiar...".
  renderWorldStatus({ services: {} }, root)
  try {
    renderWorldStatus(await fetchWorldStatus(fetchImpl, timeoutMs), root)
    return true
  } catch {
    return false
  }
}

if (typeof document !== 'undefined') {
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => void refreshWorldStatus(), { once: true })
  } else {
    void refreshWorldStatus()
  }
  globalThis.setInterval(() => void refreshWorldStatus(), 15_000)
}
