/*
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: GPT-5 Codex
TIMESTAMP: 2026-09-04 14:32:00 Europe/London
REASON FOR CREATION: Prove that Control Room reads Headplane status without credentials or mutation and renders measured up, down and unknown states.
==========================================
*/

import assert from 'node:assert/strict'
import test from 'node:test'

import { fetchWorldStatus, headplanePanelView } from '../src/api.ts'

test('Control Room reads the public status endpoint with a credential-free GET', async () => {
  const previousFetch = globalThis.fetch
  let requestedUrl = ''
  let requestedInit: RequestInit | undefined
  globalThis.fetch = async (input, init) => {
    requestedUrl = String(input)
    requestedInit = init
    return Response.json({
      readOnly: true,
      services: {
        darkstar: { state: 'up', probe: 'http', target: '127.0.0.1:18080' },
        headscale: { state: 'up', probe: 'http', target: '192.168.2.1:8080' },
        headplane: { state: 'up', probe: 'tcp', target: '127.0.0.1:3000' },
      },
    })
  }

  try {
    const status = await fetchWorldStatus()
    assert.equal(requestedUrl, '/v1/world/status')
    assert.equal(requestedInit?.method, 'GET')
    assert.deepEqual(requestedInit?.headers, { accept: 'application/json' })
    assert.equal(requestedInit && 'body' in requestedInit, false)
    assert.equal(new Headers(requestedInit?.headers).has('authorization'), false)
    assert.equal(status.services.headplane.state, 'up')
  } finally {
    globalThis.fetch = previousFetch
  }
})

test('Headplane panel distinguishes up, down and unavailable readings', () => {
  assert.deepEqual(
    headplanePanelView({ state: 'up', probe: 'tcp', target: '127.0.0.1:3000' }),
    { state: 'up', label: 'UP', listen: '127.0.0.1:3000' },
  )
  assert.deepEqual(
    headplanePanelView({ state: 'down', probe: 'tcp', target: '127.0.0.1:3000' }),
    { state: 'down', label: 'DOWN', listen: '127.0.0.1:3000' },
  )
  assert.deepEqual(headplanePanelView(undefined), {
    state: 'unknown',
    label: 'BRAK ODCZYTU',
    listen: '127.0.0.1:3000',
  })
})
