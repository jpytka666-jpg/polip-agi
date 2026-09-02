/*
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Opus 5
TIMESTAMP: 2026-09-02 09:30:00
REASON FOR CREATION: Interaktywna krawedz grafu - zywy ruch wzdluz nitki i przelacznik na jej srodku.
MECHANICS: Rysuje gruby przewod prowadzony pod katem prostym (getSmoothStepPath) - zielona gdy nitka zyje, czerwona i przerywana gdy jest
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

import { BaseEdge, EdgeLabelRenderer, getSmoothStepPath, type EdgeProps } from '@xyflow/react'

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
  // Trasa jak w instalacji elektrycznej: odcinki poziome i pionowe, zaokraglone
  // narozniki, staly odstep od gniazda. Krzywe Beziera plataly sie na gestym grafie -
  // przy prostokatnym prowadzeniu widac, ktory przewod dokad biegnie.
  const [path, labelX, labelY] = getSmoothStepPath({
    sourceX,
    sourceY,
    sourcePosition,
    targetX,
    targetY,
    targetPosition,
    borderRadius: 10,
    offset: 26,
  })

  // Zywa nitka = zielona, martwa = ciemna krew i przerywana. Zmierzone na jasnym
  // plotnie #dcdcd8: zielen #2b8a3e 3.2:1, krew #6b0f1a 8.9:1. Biel na krwi 12.3:1,
  // wiec etykieta martwej nitki jest bialym tekstem na jej wlasnym kolorze.
  const stroke = d.enabled ? (d.lit ? '#1f7a32' : '#2b8a3e') : '#6b0f1a'
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

      {/* Zywy ruch: znaczniki kierunku wedruja po sciezce. rotate="auto" obraca grot
          zgodnie z biegiem nitki, wiec od razu widac, w ktora strone plynie.
          Pomarancz sam w sobie ma na jasnym plotnie tylko 1.7:1, dlatego dostaje
          ciemna obwodke, a grot jest czarny - 8.1:1 na pomaranczy. */}
      {d.enabled ? (
        <>
          <g>
            <circle
              r={d.lit ? 11 : 9}
              fill="#ff8c1a"
              stroke="#7a3b00"
              strokeWidth="2"
            />
            <path
              d={d.lit ? 'M -3.5 -5 L 5.5 0 L -3.5 5 Z' : 'M -3 -4 L 4.5 0 L -3 4 Z'}
              fill="#111111"
            />
            <animateMotion
              dur={d.lit ? '1.6s' : '3.2s'}
              repeatCount="indefinite"
              rotate="auto"
              path={path}
            />
          </g>
          <g>
            <circle r={d.lit ? 9 : 7.5} fill="#ffb45c" stroke="#7a3b00" strokeWidth="1.6" />
            <path d="M -2.6 -3.6 L 4 0 L -2.6 3.6 Z" fill="#111111" />
            <animateMotion
              dur={d.lit ? '1.6s' : '3.2s'}
              begin="0.8s"
              repeatCount="indefinite"
              rotate="auto"
              path={path}
            />
          </g>
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
