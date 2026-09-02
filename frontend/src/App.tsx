/*
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Opus 5
TIMESTAMP: 2026-09-01 23:35:00
REASON FOR CREATION: Szkielet Control Room zastepujacy starter Vite (Task 10).
MECHANICS: Sklada dwa widoki - stan bramy i graf systemu - wokol czterech pol PIN operatora.
PIN pozostaje w pamieci karty, nigdy w repozytorium ani localStorage. Aplikacja wykonuje wylacznie
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

import { useRef, useState, type ClipboardEvent, type KeyboardEvent } from 'react'
import { ContextPanel } from './ContextPanel'
import { GatewayPanel } from './GatewayPanel'
import { GitPanel } from './GitPanel'
import { SystemGraph } from './SystemGraph'
import {
  OPERATOR_PIN_LENGTH,
  createEmptyPinCells,
  pinCellsFromText,
  pinFromCells,
  replacePinCell,
} from './operatorPin'
import './App.css'

function App() {
  const [pin, setPin] = useState('')
  const [pinCells, setPinCells] = useState(createEmptyPinCells)
  const pinInputRefs = useRef<Array<HTMLInputElement | null>>([])
  const draftPin = pinFromCells(pinCells)

  const applyPin = () => {
    if (draftPin) {
      setPin(draftPin)
    }
  }

  const updatePinCell = (index: number, value: string) => {
    setPinCells((current) => replacePinCell(current, index, value))
    if (value && index < OPERATOR_PIN_LENGTH - 1) {
      pinInputRefs.current[index + 1]?.focus()
    }
  }

  const handlePinKeyDown = (index: number, event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key === 'Backspace' && !pinCells[index] && index > 0) {
      pinInputRefs.current[index - 1]?.focus()
    }
  }

  const handlePinPaste = (event: ClipboardEvent<HTMLDivElement>) => {
    event.preventDefault()
    const pastedCells = pinCellsFromText(event.clipboardData.getData('text'))
    setPinCells(pastedCells)
    const lastFilled = Math.max(0, pastedCells.findLastIndex(Boolean))
    pinInputRefs.current[lastFilled]?.focus()
  }

  return (
    <div className="control-room">
      <header className="control-room__header">
        <h1>Darkstar Control Room</h1>
        <div className="operator-pin">
          <span className="operator-pin__label">PIN operatora</span>
          <div
            className="operator-pin__cells"
            role="group"
            aria-label="Czteroznakowy PIN operatora"
            onPaste={handlePinPaste}
          >
            {pinCells.map((value, index) => (
              <input
                key={index}
                ref={(element) => {
                  pinInputRefs.current[index] = element
                }}
                className="operator-pin__cell"
                type="password"
                inputMode="text"
                autoComplete="off"
                autoCapitalize="none"
                spellCheck={false}
                maxLength={1}
                value={value}
                aria-label={`Znak PIN ${index + 1} z ${OPERATOR_PIN_LENGTH}`}
                onChange={(event) => updatePinCell(index, event.target.value)}
                onKeyDown={(event) => handlePinKeyDown(index, event)}
              />
            ))}
          </div>
          <button type="button" disabled={!draftPin} onClick={applyPin}>
            Zastosuj
          </button>
        </div>
      </header>

      <main className="control-room__body">
        <div className="stack">
          <GatewayPanel token={pin} />
          <ContextPanel token={pin} />
        </div>
        <SystemGraph token={pin} />
        <aside className="git-rail-column" aria-label="Graf commitow lokalnego Git">
          <GitPanel />
        </aside>
      </main>

      <footer className="control-room__footer">
        Graf systemu tylko do odczytu. Git pozwala wylacznie odswiezyc widok albo pobrac origin — bez checkout.
      </footer>
    </div>
  )
}

export default App
