/*
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Opus 5
TIMESTAMP: 2026-09-01 23:30:00
REASON FOR CREATION: Widok grafu systemu w Control Room, oparty na istniejacym GET /v1/system-graph (Task 10).
MECHANICS: Pobiera snapshot architektury i przeklada wezly oraz krawedzie na graf React Flow.
Uklad jest wyliczany prosto - kolumny wedlug rodzaju wezla - zeby nie dokladac biblioteki
ukladajacej. Widok nie edytuje grafu i nie wysyla niczego z powrotem.
SYSTEM PART: Control Room / widok architektury.
ARCHITECTURE FUNCTION: Zastepuje starter Vite pierwszym prawdziwym widokiem systemu; graf jest
czytany z tego samego zrodla, ktore obsluguje strona serwerowa, wiec nie ma drugiej prawdy.
DEPENDENCIES/LINKS: api.ts (fetchSystemGraph), @xyflow/react (juz w zaleznosciach projektu).
TECH STACK: TypeScript 6 + React 19 + React Flow 12, swiadomie zamiast Rusta - domyslnego jezyka.
  (1) MUSI: rysowac interaktywny graf w przegladarce.
  (2) DLACZEGO NIE RUST: React Flow to biblioteka DOM; Rust przez WebAssembly nie ma tu
      odpowiednika i wymagalby warstwy TS tak czy inaczej.
  (3) TRACIMY: typy wspolne z rdzeniem; snapshot pozostaje wytwarzany przez Rusta.
LOCAL WORKSPACE: D:\codex-fresh-2026-08-28\worktrees\polip-agi-darkstar-plan
GIT COMMIT: PENDING
GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
==========================================
*/

import { useEffect, useMemo, useState } from 'react'
import { ReactFlow, Background, Controls, type Edge, type Node } from '@xyflow/react'
import '@xyflow/react/dist/style.css'
import { fetchSystemGraph, type ArchitectureSnapshot } from './api'

/** Kolumna wedlug rodzaju wezla - prosty uklad bez dodatkowej biblioteki. */
const COLUMN: Record<string, number> = {
  repository: 0,
  directory: 1,
  module: 2,
  file: 3,
  dependency: 4,
  runtime: 5,
}

function toFlow(snapshot: ArchitectureSnapshot): { nodes: Node[]; edges: Edge[] } {
  const seen: Record<number, number> = {}

  const nodes: Node[] = snapshot.nodes.map((node) => {
    const column = COLUMN[node.kind] ?? 6
    const row = seen[column] ?? 0
    seen[column] = row + 1
    return {
      id: node.id,
      position: { x: column * 260, y: row * 90 },
      data: { label: node.name },
      className: `graph-node graph-node--${node.kind}`,
    }
  })

  const edges: Edge[] = snapshot.edges.map((edge) => ({
    id: edge.id,
    source: edge.from,
    target: edge.to,
    label: edge.kind,
  }))

  return { nodes, edges }
}

export function SystemGraph({ token }: { token: string }) {
  const [snapshot, setSnapshot] = useState<ArchitectureSnapshot | null>(null)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    let cancelled = false
    fetchSystemGraph(token)
      .then((next) => {
        if (!cancelled) {
          setSnapshot(next)
          setError(null)
        }
      })
      .catch((err: Error) => {
        if (!cancelled) setError(err.message)
      })
    return () => {
      cancelled = true
    }
  }, [token])

  const flow = useMemo(
    () => (snapshot ? toFlow(snapshot) : { nodes: [], edges: [] }),
    [snapshot],
  )

  if (error) {
    return (
      <section className="panel panel--error">
        <h2>Graf systemu</h2>
        <p role="alert">{error}</p>
      </section>
    )
  }

  return (
    <section className="panel panel--graph">
      <h2>
        Graf systemu{' '}
        <span className="badge">
          {flow.nodes.length} wezlow / {flow.edges.length} krawedzi
        </span>
      </h2>
      <div className="graph-canvas">
        <ReactFlow nodes={flow.nodes} edges={flow.edges} fitView nodesDraggable={false}>
          <Background />
          <Controls showInteractive={false} />
        </ReactFlow>
      </div>
    </section>
  )
}
