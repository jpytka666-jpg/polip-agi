/*
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Opus 5
TIMESTAMP: 2026-09-02 09:30:00
REASON FOR CREATION: Interaktywna krawedz grafu - zywy ruch wzdluz nitki i przelacznik na jej srodku.
MECHANICS: Rysuje gruba sciezke Beziera - zielona gdy nitka zyje, czerwona i przerywana gdy jest
wygaszona - a po niej puszcza dwie kropki animowane przez SVG animateMotion, wiec ruch widac bez
zadnej petli w JavaScripcie. Na srodku siedzi przelacznik, ktory wlacza i
wylacza te nitke NA PLOTNIE.
UWAGA - ZAKRES: przelacznik zmienia WIDOK, nie system. API Darkstara wystawia wylacznie odczyt
(GET), wiec z przegladarki nie da sie ani zatrzymac, ani uruchomic zadnego polaczenia. Etykieta
mowi to wprost, zeby nikt nie wzial wygaszonej nitki za wylaczona usluge.
SYSTEM PART: Control Room / widok architektury.
ARCHITECTURE FUNCTION: Pozwala operatorowi wyciszyc szum na gestym grafie i przesledzic jedna
sciezke, bez dotykania czegokolwiek na hoscie.
DEPENDENCIES/LINKS: SystemGraph.tsx, @xyflow/react (BaseEdge, EdgeLabelRenderer).
TECH STACK: TypeScript 6 + React 19 + React Flow 12, swiadomie zamiast Rusta - domyslnego jezyka.
  (1) MUSI: rysowac animowana sciezke SVG i klikalny przelacznik w warstwie DOM nad plotnem.
  (2) DLACZEGO NIE RUST: to render SVG i zdarzenia myszy w przegladarce; Rust przez WebAssembly
      wymagalby tej samej warstwy TS. Dane i decyzje zostaja po stronie serwera w Ruscie.
  (3) TRACIMY: typy wspolne z rdzeniem; kontrakt odbity w api.ts.
LOCAL WORKSPACE: D:\codex-fresh-2026-08-28\worktrees\polip-agi-darkstar-plan
GIT COMMIT: PENDING
GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
==========================================
*/

import { BaseEdge, EdgeLabelRenderer, getBezierPath, type EdgeProps } from '@xyflow/react'

export type FlowEdgeData = {
  label: string
  enabled: boolean
  lit: boolean
  dimmed: boolean
  onToggle: (id: string) => void
}

export function FlowEdge({
  id,
  sourceX,
  sourceY,
  targetX,
  targetY,
  sourcePosition,
  targetPosition,
  data,
}: EdgeProps) {
  const d = data as unknown as FlowEdgeData
  const [path, labelX, labelY] = getBezierPath({
    sourceX,
    sourceY,
    sourcePosition,
    targetX,
    targetY,
    targetPosition,
  })

  // Zywa nitka = zielona, martwa = czerwona i przerywana. Kolory dobrane pod jasne
  // plotno (#dcdcd8): #2b8a3e daje 3.2:1, #c92a2a 4.0:1 - to elementy graficzne,
  // nie tekst, wiec prog 3:1 dla grafiki jest spelniony z zapasem przy tej grubosci.
  const stroke = d.enabled ? (d.lit ? '#1f7a32' : '#2b8a3e') : '#c92a2a'
  const width = d.lit ? 9 : 6

  return (
    <>
      <BaseEdge
        id={id}
        path={path}
        style={{
          stroke,
          strokeWidth: width,
          opacity: d.dimmed && !d.lit ? 0.22 : 1,
          strokeLinecap: 'round',
          strokeDasharray: d.enabled ? undefined : '10 8',
        }}
      />

      {/* Zywy ruch: kropki wedruja po tej samej sciezce. Czysty SVG, bez petli w JS. */}
      {d.enabled ? (
        <>
          <circle r={d.lit ? 5.5 : 4.5} fill={d.lit ? 'var(--accent)' : '#b8f27a'} stroke="#14532d" strokeWidth="1">
            <animateMotion dur={d.lit ? '1.6s' : '3.2s'} repeatCount="indefinite" path={path} />
          </circle>
          <circle r={d.lit ? 4 : 3.2} fill={d.lit ? 'var(--gold)' : '#e8d48a'} stroke="#5a4a10" strokeWidth="0.8">
            <animateMotion
              dur={d.lit ? '1.6s' : '3.2s'}
              begin="0.8s"
              repeatCount="indefinite"
              path={path}
            />
          </circle>
        </>
      ) : null}

      <EdgeLabelRenderer>
        <div
          className={`eswitch${d.enabled ? ' eswitch--on' : ''}${d.lit ? ' eswitch--lit' : ''}`}
          style={{ transform: `translate(-50%, -50%) translate(${labelX}px, ${labelY}px)` }}
        >
          <button
            type="button"
            className="eswitch__toggle"
            role="switch"
            aria-checked={d.enabled}
            aria-label={`${d.label}: pokaz lub ukryj te nitke na plotnie`}
            title={
              d.enabled
                ? `${d.label} — widoczna. Klik ukrywa nitke NA PLOTNIE (nie zmienia systemu).`
                : `${d.label} — ukryta. Klik przywraca ja NA PLOTNIE (nie zmienia systemu).`
            }
            onClick={(e) => {
              e.stopPropagation()
              d.onToggle(id)
            }}
          >
            <span className="eswitch__knob" />
          </button>
          <span className="eswitch__label">{d.label}</span>
        </div>
      </EdgeLabelRenderer>
    </>
  )
}
