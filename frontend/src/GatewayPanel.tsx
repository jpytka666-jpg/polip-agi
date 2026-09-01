/*
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Opus 5
TIMESTAMP: 2026-09-01 23:25:00
REASON FOR CREATION: Panel stanu bramy Darkstar w Control Room (Task 10).
MECHANICS: Odpytuje GET /v1/gateway/status co 10 sekund i pokazuje kondycje, adresacje oraz
liczbe klientow. Nie ma zadnego przycisku sterujacego - API bramy wystawia wylacznie odczyt,
wiec panel tez wylacznie czyta.
SYSTEM PART: Control Room / widok bramy.
ARCHITECTURE FUNCTION: Pierwsze miejsce, w ktorym operator widzi zywy stan bramy bez logowania
sie na host przez SSH.
DEPENDENCIES/LINKS: api.ts (fetchGatewayStatus), darkstar-server /v1/gateway/status.
TECH STACK: TypeScript 6 + React 19, swiadomie zamiast Rusta - domyslnego jezyka projektu.
  (1) MUSI: renderowac sie w przegladarce i odswiezac widok w tle.
  (2) DLACZEGO NIE RUST: to komponent DOM; Rust przez WebAssembly wymagalby i tak warstwy TS,
      a zaufany control plane pozostaje w Ruscie po stronie serwera.
  (3) TRACIMY: wspolne typy z rdzeniem; odbicie kontraktu trzymane w api.ts.
LOCAL WORKSPACE: D:\codex-fresh-2026-08-28\worktrees\polip-agi-darkstar-plan
GIT COMMIT: PENDING
GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
==========================================
*/

import { useEffect, useState } from 'react'
import { fetchGatewayStatus, type GatewayStatus } from './api'

const HEALTH_LABEL: Record<string, string> = {
  ready: 'gotowa',
  degraded: 'z zastrzezeniem',
  offline: 'wylaczona',
  starting: 'startuje',
  failed: 'awaria',
}

export function GatewayPanel({ token }: { token: string }) {
  const [status, setStatus] = useState<GatewayStatus | null>(null)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    let cancelled = false

    const load = () => {
      fetchGatewayStatus(token)
        .then((next) => {
          if (cancelled) return
          setStatus(next)
          setError(null)
        })
        .catch((err: Error) => {
          if (cancelled) return
          setError(err.message)
        })
    }

    load()
    const timer = window.setInterval(load, 10_000)
    return () => {
      cancelled = true
      window.clearInterval(timer)
    }
  }, [token])

  if (error) {
    return (
      <section className="panel panel--error">
        <h2>Brama</h2>
        <p role="alert">{error}</p>
      </section>
    )
  }

  if (!status) {
    return (
      <section className="panel">
        <h2>Brama</h2>
        <p>Odczyt...</p>
      </section>
    )
  }

  return (
    <section className={`panel panel--${status.health}`}>
      <h2>
        Brama <span className="badge">{HEALTH_LABEL[status.health] ?? status.health}</span>
      </h2>
      <dl className="facts">
        <dt>Uplink</dt>
        <dd>{status.upstream_interface}</dd>
        <dt>Downstream</dt>
        <dd>{status.downstream_interface}</dd>
        <dt>Adres bramy</dt>
        <dd>{status.downstream_cidr}</dd>
        <dt>Siec prywatna</dt>
        <dd>{status.downstream_subnet}</dd>
        <dt>Profil</dt>
        <dd>{status.connection_profile}</dd>
        <dt>Klienci</dt>
        <dd>{status.connected_clients}</dd>
        <dt>Tryb</dt>
        <dd>{status.mode}</dd>
      </dl>
      <p className="note">
        Widok tylko do odczytu. Sterowanie brama nie jest wystawione w API.
      </p>
    </section>
  )
}
