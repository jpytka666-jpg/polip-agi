/*
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: GPT-5 Codex
TIMESTAMP: 2026-09-02 17:58:31 Europe/London
REASON FOR CREATION: Lock the four-cell operator PIN and Authorization header contract without using the real secret.
==========================================
*/

import assert from 'node:assert/strict'
import test from 'node:test'

import {
  authorizationHeaders,
  createEmptyPinCells,
  pinCellsFromText,
  pinFromCells,
  replacePinCell,
} from '../src/operatorPin.ts'

test('operator pin starts with exactly four cells', () => {
  assert.deepEqual(createEmptyPinCells(), ['', '', '', ''])
})

test('incomplete operator pin does not authorize a request', () => {
  assert.equal(pinFromCells(['x', 'Y', '7', '']), '')
  assert.deepEqual(authorizationHeaders(''), {})
})

test('complete operator pin gets one bearer scheme prefix', () => {
  const pin = pinFromCells(['x', 'Y', '!', '7'])

  assert.equal(pin, 'xY!7')
  assert.deepEqual(authorizationHeaders(pin), {
    authorization: 'Bearer xY!7',
  })
})

test('pasted operator pin fills no more than four cells', () => {
  assert.deepEqual(pinCellsFromText('xY!7ignored'), ['x', 'Y', '!', '7'])
})

test('editing one operator pin cell preserves the other cells', () => {
  assert.deepEqual(replacePinCell(['x', '', '!', ''], 1, 'Z'), [
    'x',
    'Z',
    '!',
    '',
  ])
})
