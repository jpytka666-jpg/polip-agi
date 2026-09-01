/*
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Opus 5
TIMESTAMP: 2026-09-02 06:10:00
REASON FOR CREATION: Panel pamieci AIONS w Control Room - kondycja obu nog i wyszukiwanie kolekcji.
MECHANICS: Odpytuje GET /v1/context/health oraz GET /v1/context/search. Pokazuje osobno noge
lokalna na CBMS i zdalna na E:, zeby bylo widac, ktora obsluzyla odpowiedz. Nie ma tu zadnego
przycisku zapisu - API pamieci wystawia wylacznie odczyt.
SYSTEM PART: Control Room / widok pamieci.
ARCHITECTURE FUNCTION: Operator widzi stan pamieci bez logowania sie na host i bez laczenia sie
z Chroma bezposrednio.
DEPENDENCIES/LINKS: api.ts (fetchContextHealth, searchContext), darkstar-server /v1/context/*.
TECH STACK: TypeScript 6 + React 19, swiadomie zamiast Rusta - domyslnego jezyka projektu.
  (1) MUSI: renderowac sie w przegladarce operatora i odswiezac widok.
  (2) DLACZEGO NIE RUST: to komponent DOM; Rust przez WebAssembly wymagalby i tak warstwy TS,
      a decyzje i uprawnienia zostaja po stronie serwera w Ruscie.
  (3) TRACIMY: wspolne typy z rdzeniem; odbicie kontraktu trzymane w api.ts.
LOCAL WORKSPACE: D:\codex-fresh-2026-08-28\worktrees\polip-agi-darkstar-plan
GIT COMMIT: PENDING
GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
==========================================
*/

import { useEffect, useState } from 'react'
import {
  fetchContextHealth,
  searchContext,
  type ContextHealth,
  type ContextListing,
} from './api'

const LEG_LABEL: Record<string, string> = {
  local_cbms: 'noga lokalna (CBMS)',
  remote_e: 'noga zdalna (E:)',
}

export function ContextPanel({ token }: { token: string }) {
  const [health, setHealth] = useState<ContextHealth | null>(null)
  const [listing, setListing] = useState<ContextListing | null>(null)
  const [query, setQuery] = useState('session')
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    let cancelled = false

    const load = () => {
      fetchContextHealth(token)
        .then((next) => {
          if (!cancelled) {
            setHealth(next)
            setError(null)
          }
        })
        .catch((err: Error) => {
          if (!cancelled) setError(err.message)
        })

      searchContext(token, query, 5)
        .then((next) => {
          if (!cancelled) setListing(next)
        })
        .catch(() => {
          if (!cancelled) setListing(null)
        })
    }

    load()
    const timer = window.setInterval(load, 15_000)
    return () => {
      cancelled = true
      window.clearInterval(timer)
    }
  }, [token, query])

  if (error) {
    return (
      <section className="panel panel--error">
        <h2>Pamiec</h2>
        <p role="alert">{error}</p>
      </section>
    )
  }

  const anyLeg = health ? health.local_cbms_ok || health.remote_e_ok : false

  return (
    <section className={`panel panel--${anyLeg ? 'ready' : 'offline'}`}>
      <h2>
        Pamiec <span className="badge">{anyLeg ? 'dostepna' : 'niedostepna'}</span>
      </h2>

      <dl className="facts">
        <dt>Noga lokalna</dt>
        <dd>{health ? (health.local_cbms_ok ? 'odpowiada' : 'cisza') : '...'}</dd>
        <dt>Noga zdalna</dt>
        <dd>{health ? (health.remote_e_ok ? 'odpowiada' : 'cisza') : '...'}</dd>
        <dt>Obsluzyla</dt>
        <dd>{listing ? (LEG_LABEL[listing.served_by] ?? listing.served_by) : '—'}</dd>
      </dl>

      <label className="search-row">
        <span>Szukaj</span>
        <input
          type="search"
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          placeholder="np. session"
        />
      </label>

      <ul className="collections">
        {listing?.collections.length ? (
          listing.collections.map((c) => (
            <li key={c.id}>
              {c.name}
              {c.dimension ? <span className="dim"> · {c.dimension}</span> : null}
            </li>
          ))
        ) : (
          <li className="dim">brak dopasowan</li>
        )}
      </ul>

      <p className="note">Widok tylko do odczytu. Pamiec nie jest modyfikowana z przegladarki.</p>
    </section>
  )
}
