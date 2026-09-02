/*
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Opus 5
TIMESTAMP: 2026-09-01 23:35:00
REASON FOR CREATION: Szkielet Control Room zastepujacy starter Vite (Task 10).
MECHANICS: Sklada dwa widoki - stan bramy i graf systemu - wokol pola na token operatora.
Token trafia do pamieci przegladarki, nigdy do repozytorium. Aplikacja wykonuje wylacznie
zapytania GET; nie ma tu zadnego przycisku, ktory zmienialby cokolwiek na hoscie.
SYSTEM PART: Control Room / szkielet aplikacji.
ARCHITECTURE FUNCTION: Pierwsza wersja panelu operatora. Zadna przegladarka nie uruchamia
polecen systemowych - to pozostaje rola zaufanego control plane w Ruscie.
DEPENDENCIES/LINKS: GatewayPanel, SystemGraph, api.ts.
TECH STACK: TypeScript 6 + React 19 + Vite 8, swiadomie zamiast Rusta - domyslnego jezyka.
  (1) MUSI: dzialac jako strona w przegladarce operatora.
  (2) DLACZEGO NIE RUST: to warstwa prezentacji w DOM; Rust przez WebAssembly nadal wymaga
      warstwy TS, a logika decyzyjna ma zostac po stronie serwera w Ruscie.
  (3) TRACIMY: jeden jezyk w calym stosie. Rekompensata: przegladarka nie ma zadnej wladzy -
      caly zakres dzialania to zapytania odczytu.
LOCAL WORKSPACE: D:\codex-fresh-2026-08-28\worktrees\polip-agi-darkstar-plan
GIT COMMIT: PENDING
GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
==========================================
*/

import { useState } from 'react'
import { ContextPanel } from './ContextPanel'
import { GatewayPanel } from './GatewayPanel'
import { GitPanel } from './GitPanel'
import { SystemGraph } from './SystemGraph'
import { readToken, storeToken } from './api'
import './App.css'

function App() {
  const [token, setToken] = useState(readToken)
  const [draft, setDraft] = useState(readToken)

  const applyToken = () => {
    storeToken(draft)
    setToken(draft)
  }

  return (
    <div className="control-room">
      <header className="control-room__header">
        <h1>Darkstar Control Room</h1>
        <div className="token-row">
          <label htmlFor="token">Token operatora</label>
          <input
            id="token"
            type="password"
            value={draft}
            onChange={(event) => setDraft(event.target.value)}
            placeholder="Bearer ..."
          />
          <button type="button" onClick={applyToken}>
            Zastosuj
          </button>
        </div>
      </header>

      <main className="control-room__body">
        <div className="stack">
          <GatewayPanel token={token} />
          <ContextPanel token={token} />
        </div>
        <div className="workspace">
          <GitPanel />
          <SystemGraph token={token} />
        </div>
      </main>

      <footer className="control-room__footer">
        Widok tylko do odczytu. Zadne polecenie systemowe nie jest uruchamiane z przegladarki.
      </footer>
    </div>
  )
}

export default App
