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

import { fetchWorldStatus, serviceStatusView } from '../public/world/world.js'

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
  assert.equal('body' in request.init, false)
  assert.equal('authorization' in request.init.headers, false)
})

test('tile labels distinguish live, down and unknown states', () => {
  assert.deepEqual(serviceStatusView('up'), {
    label: 'żywe',
    state: 'up',
  })
  assert.deepEqual(serviceStatusView('down'), {
    label: 'brak odpowiedzi',
    state: 'down',
  })
  assert.deepEqual(serviceStatusView(undefined), {
    label: 'stan nieznany',
    state: 'unknown',
  })
})
