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

import { ContextPanel } from './ContextPanel'
import { GatewayPanel } from './GatewayPanel'
import { GitPanel } from './GitPanel'
import { SystemGraph } from './SystemGraph'
import './App.css'

/**
 * Sterownia jest otwarta: wejscie na strone od razu pokazuje brame, pamiec, graf i Git.
 *
 * Panele nadal przyjmuja `token`, tylko dostaja pusty napis - a pusty napis znaczy
 * "nie wysylaj naglowka Authorization". Wpuszczenie zalatwia serwer w Ruscie: zapytanie
 * z petli zwrotnej przechodzi bez naglowka, adres spoza petli nadal dostaje 401.
 *
 * Przewod na token zostaje CELOWO. Zamkniecie Sterowni z powrotem to podanie tym czterem
 * miejscom prawdziwej wartosci, a nie przepisywanie kazdego panelu od nowa.
 */
const NO_TOKEN = ''

function App() {
  return (
    <div className="control-room">
      <header className="control-room__header">
        <h1>Darkstar Control Room</h1>
      </header>

      <main className="control-room__body">
        <div className="stack">
          <GatewayPanel token={NO_TOKEN} />
          <ContextPanel token={NO_TOKEN} />
        </div>
        <SystemGraph token={NO_TOKEN} />
        <aside className="git-rail-column" aria-label="Graf commitow z darkstar-server">
          <GitPanel token={NO_TOKEN} />
        </aside>
      </main>

      <footer className="control-room__footer">
        Wszystko tylko do odczytu. Git pozwala wylacznie odswiezyc widok — bez checkout, merge i reset.
      </footer>
    </div>
  )
}

export default App
