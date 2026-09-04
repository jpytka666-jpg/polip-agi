/*
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: GPT-5 Codex
TIMESTAMP: 2026-09-04 14:08:55
REASON FOR CREATION: Prove that world tiles read one public status endpoint without credentials or mutation.
==========================================
*/

import assert from 'node:assert/strict'
import test from 'node:test'

import {
  fetchWorldStatus,
  refreshWorldStatus,
  serviceStatusView,
} from '../public/world/world.js'

test('world status request is a credential-free GET', async () => {
  let request = null
  const payload = {
    readOnly: true,
    services: {
      darkstar: { state: 'up', target: '127.0.0.1:18080' },
      headscale: { state: 'up', target: '192.168.2.1:8080' },
      headplane: { state: 'up', target: '127.0.0.1:3000' },
    },
  }
  const fakeFetch = async (input, init) => {
    request = { input, init }
    return { ok: true, json: async () => payload }
  }

  assert.deepEqual(await fetchWorldStatus(fakeFetch), payload)
  assert.equal(request.input, '/v1/world/status')
  assert.equal(request.init.method, 'GET')
  assert.deepEqual(request.init.headers, { accept: 'application/json' })
  assert.equal(request.init.signal instanceof AbortSignal, true)
  assert.equal('body' in request.init, false)
  assert.equal('authorization' in request.init.headers, false)
})

test('tile labels expose only literal up or down states', () => {
  assert.deepEqual(serviceStatusView('up'), {
    label: 'up',
    state: 'up',
  })
  assert.deepEqual(serviceStatusView('down'), {
    label: 'down',
    state: 'down',
  })
  assert.deepEqual(serviceStatusView(undefined), {
    label: 'down',
    state: 'down',
  })
})

function statusRoot(names) {
  const elements = names.map((name) => ({
    dataset: { worldStatus: name, state: 'down' },
    textContent: 'down',
  }))
  return {
    elements,
    root: {
      querySelectorAll: () => elements,
    },
  }
}

test('a completed GET replaces all three badges with measured up/down values', async () => {
  const { elements, root } = statusRoot(['headscale', 'darkstar', 'headplane'])
  const payload = {
    readOnly: true,
    services: {
      darkstar: { state: 'up', probe: 'http_get', target: 'http://127.0.0.1:18080/health' },
      headscale: { state: 'down', probe: 'http_get', target: 'http://192.168.2.1:8080/health' },
      headplane: { state: 'up', probe: 'tcp_connect', target: '127.0.0.1:3000' },
    },
  }

  assert.equal(await refreshWorldStatus(async () => Response.json(payload), root, 50), true)
  assert.deepEqual(
    elements.map((element) => [element.dataset.worldStatus, element.textContent]),
    [
      ['headscale', 'down'],
      ['darkstar', 'up'],
      ['headplane', 'up'],
    ],
  )
})

test('a stalled client path becomes down instead of staying on measurement', { timeout: 1_000 }, async () => {
  const { elements, root } = statusRoot(['headscale', 'darkstar', 'headplane'])
  const neverResponds = async () => new Promise(() => {})

  assert.equal(await refreshWorldStatus(neverResponds, root, 10), false)
  assert.deepEqual(elements.map((element) => element.textContent), ['down', 'down', 'down'])
})
