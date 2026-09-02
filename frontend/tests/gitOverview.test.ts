/*
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: GPT-5 Codex
TIMESTAMP: 2026-09-02 18:40:37 Europe/London
REASON FOR CREATION: Verify that Control Room reads structured Git data from darkstar-server and treats a missing server endpoint as an unavailable Git view.
==========================================
*/

import assert from 'node:assert/strict'
import test from 'node:test'

import { fetchGitOverview } from '../src/api.ts'

test('missing Git endpoint becomes an unavailable view instead of a raw error', async () => {
  const previousFetch = globalThis.fetch
  globalThis.fetch = async () => new Response('', { status: 404 })

  try {
    assert.equal(await fetchGitOverview('xY!'), null)
  } finally {
    globalThis.fetch = previousFetch
  }
})

test('Git overview is read from the authenticated darkstar-server endpoint', async () => {
  const previousFetch = globalThis.fetch
  let requestedUrl = ''
  let requestedAuthorization = ''
  globalThis.fetch = async (input, init) => {
    requestedUrl = String(input)
    requestedAuthorization = new Headers(init?.headers).get('authorization') ?? ''
    return Response.json({
      branch: 'docs/example',
      head: '0123456789abcdef0123456789abcdef01234567',
      dirty: false,
      ahead: 1,
      behind: 0,
      hasUpstream: true,
      commits: [],
    })
  }

  try {
    const overview = await fetchGitOverview('xY!')
    assert.equal(requestedUrl, '/v1/git/overview')
    assert.equal(requestedAuthorization, 'Bearer xY!')
    assert.equal(overview?.branch, 'docs/example')
  } finally {
    globalThis.fetch = previousFetch
  }
})
