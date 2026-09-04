/*
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Sonnet 5
TIMESTAMP: 2026-09-04 17:05:00 Europe/London
REASON FOR CREATION: Task 14, Step 14.10 - permission boundary tests. Prove that the public
world landing page never receives the Control Room-only tunnel link, the LAN headplane
address, or an all-interfaces bind - all three are facts that only make sense on the
operator's own Windows machine, never on a device reached from the private network.
MECHANICS: Reads the two shipped static files under frontend/public/world as plain text and
asserts three forbidden substrings never appear in either one.
SYSTEM PART: Test suite / permission boundary between Control Room and the public world page.
ARCHITECTURE FUNCTION: Regression guard - if a future edit ever copies the tunnel href or the
LAN headplane address into the public page, this test fails before it ships.
DEPENDENCIES/LINKS: frontend/public/world/index.html, frontend/public/world/world.js.
TECH STACK: TypeScript + node:test, swiadomie zamiast Rusta - domyslnego jezyka tego projektu.
  (1) MUSI: uruchamiac sie w tym samym `node --test`, ktory juz czyta te same dwa pliki w
      headplaneStatus.test.ts (Control Room), zeby oba boki granicy byly sprawdzane jednym
      przebiegiem testow frontendu, bez osobnej instalacji ani drugiego runnera.
  (2) DLACZEGO NIE RUST: te dwa pliki to gotowy tekst HTML/JS wysylany do przegladarki -
      Rust nigdy ich nie generuje ani nie serwuje z tego katalogu (Vite kopiuje je jako
      zasob statyczny). Test w Rust musialby i tak otworzyc te same sciezki na dysku i
      dolozyc drugi harness dla jednego porownania tekstu.
  (3) TRACIMY: nic wspolnego z typami Rusta - to plaskie porownanie tekstu, bez zadnego
      sprzezenia z typami darkstar-core.
LOCAL WORKSPACE: D:\codex-fresh-2026-08-28\worktrees\polip-agi-darkstar-plan
GIT COMMIT: PENDING
GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
==========================================
*/

import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import test from 'node:test'

const worldDir = new URL('../public/world/', import.meta.url)
const indexHtml = readFileSync(new URL('index.html', worldDir), 'utf8')
const worldJs = readFileSync(new URL('world.js', worldDir), 'utf8')

test('the public world page never contains the Control Room tunnel href', () => {
  for (const [name, source] of [
    ['index.html', indexHtml],
    ['world.js', worldJs],
  ] as const) {
    assert.doesNotMatch(
      source,
      /127\.0\.0\.1:3001/,
      `${name} must not know about the Control Room-only SSH tunnel port`,
    )
  }
})

test('the public world page never proposes the LAN headplane address', () => {
  for (const [name, source] of [
    ['index.html', indexHtml],
    ['world.js', worldJs],
  ] as const) {
    assert.doesNotMatch(
      source,
      /192\.168\.2\.1:3000/,
      `${name} must not suggest a headplane listener that does not exist`,
    )
  }
})

test('the public world page never binds or links to all interfaces', () => {
  for (const [name, source] of [
    ['index.html', indexHtml],
    ['world.js', worldJs],
  ] as const) {
    assert.doesNotMatch(source, /0\.0\.0\.0/, `${name} must never reference 0.0.0.0`)
  }
})

test('the memory tile carries a context status badge but stays a closed div, never a link', () => {
  const tileMatch = indexHtml.match(/<li>\s*<div class="tile[^>]*>[\s\S]*?<\/li>/g)?.find((block) =>
    block.includes('data-world-status="context"'),
  )
  assert.ok(tileMatch, 'index.html must have a tile with data-world-status="context"')
  const tile = tileMatch as string

  assert.match(
    tile,
    /^<li>\s*<div class="tile/,
    'the memory tile must open with <div>, not <a> - it has no working address to link to',
  )
  assert.doesNotMatch(tile, /<a\b/i, 'the memory tile must never contain an <a> element')
  assert.doesNotMatch(
    tile,
    /href=/i,
    'the memory tile must never carry an href, on the tile or on its status badge',
  )
})
