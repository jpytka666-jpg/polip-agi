/*
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Opus 5
TIMESTAMP: 2026-09-02 08:10:00
REASON FOR CREATION: Czytelny, interaktywny graf systemu w Control Room (Task 10/11).
MECHANICS: Pobiera GET /v1/system-graph i renderuje go w React Flow wlasnym typem wezla o stalej
szerokosci 232x64 - nazwa miesci sie w calosci, bez przycinania. Najechanie podswietla wezel i
pokazuje dymek z id, rodzajem i rola; klikniecie otwiera panel szczegolow w tej samej stronie,
bez nawigacji. Kolor krawedzi wezla koduje rodzaj. Uklad kolumnowy liczony lokalnie, bez
dodatkowej biblioteki ukladajacej.
SYSTEM PART: Control Room / widok architektury.
ARCHITECTURE FUNCTION: Jedyne miejsce, gdzie operator oglada strukture systemu; dane pochodza z
tego samego zrodla co strona serwerowa, wiec nie ma drugiej prawdy.
DEPENDENCIES/LINKS: api.ts (fetchSystemGraph), @xyflow/react (juz w zaleznosciach).
TECH STACK: TypeScript 6 + React 19 + React Flow 12, swiadomie zamiast Rusta - domyslnego jezyka.
  (1) MUSI: rysowac interaktywny graf w przegladarce, z hover, klikaniem i zoomem.
  (2) DLACZEGO NIE RUST: React Flow to biblioteka DOM; Rust przez WebAssembly nie ma tu
      odpowiednika i wymagalby warstwy TS tak czy inaczej. Snapshot nadal wytwarza Rust.
  (3) TRACIMY: typy wspolne z rdzeniem; odbicie kontraktu trzymane w api.ts.
LOCAL WORKSPACE: D:\codex-fresh-2026-08-28\worktrees\polip-agi-darkstar-plan
GIT COMMIT: PENDING
GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
==========================================
*/

import { useCallback, useEffect, useMemo, useState } from 'react'
import {
  Background,
  Controls,
  Handle,
  Position,
  ReactFlow,
  type Edge,
  type Node,
  type NodeProps,
} from '@xyflow/react'
import '@xyflow/react/dist/style.css'
import { fetchSystemGraph, type ArchitectureNode, type ArchitectureSnapshot } from './api'

/** Kolumna wedlug rodzaju wezla - prosty uklad bez dodatkowej biblioteki. */
const COLUMN: Record<string, number> = {
  repository: 0,
  directory: 1,
  module: 2,
  file: 3,
  dependency: 4,
  runtime: 5,
}

const KIND_LABEL: Record<string, string> = {
  repository: 'repozytorium',
  directory: 'katalog',
  module: 'modul',
  file: 'plik',
  dependency: 'zaleznosc',
  runtime: 'runtime',
}

const NODE_W = 232
const NODE_H = 64

type NodeData = ArchitectureNode & { active: boolean }

/** Wlasny wezel: pelna nazwa, staly rozmiar, rodzaj jako podpis. */
function ArchNode({ data }: NodeProps) {
  const d = data as unknown as NodeData
  // Dymek natywny przegladarki - dziala tez po powiekszeniu i nie wymaga biblioteki.
  const tip = [
    `id: ${d.id}`,
    `rodzaj: ${KIND_LABEL[d.kind] ?? d.kind}`,
    `rola: ${d.role ?? 'brak'}`,
  ].join('\n')

  return (
    <div
      className={`gnode gnode--${d.kind}${d.active ? ' gnode--active' : ''}`}
      title={tip}
    >
      <Handle type="target" position={Position.Top} />
      <span className="gnode__name">{d.name}</span>
      <span className="gnode__kind">{KIND_LABEL[d.kind] ?? d.kind}</span>
      <Handle type="source" position={Position.Bottom} />
    </div>
  )
}

const nodeTypes = { arch: ArchNode }

function toFlow(snapshot: ArchitectureSnapshot, activeId: string | null) {
  const used: Record<number, number> = {}

  const nodes: Node[] = snapshot.nodes.map((node) => {
    const column = COLUMN[node.kind] ?? 6
    const row = used[column] ?? 0
    used[column] = row + 1
    return {
      id: node.id,
      type: 'arch',
      position: { x: column * (NODE_W + 96), y: row * (NODE_H + 46) },
      data: { ...node, active: node.id === activeId } as unknown as Record<string, unknown>,
      draggable: true,
    }
  })

  const edges: Edge[] = snapshot.edges.map((edge) => ({
    id: edge.id,
    source: edge.from,
    target: edge.to,
    label: edge.kind,
    animated: edge.from === activeId || edge.to === activeId,
    style: {
      strokeWidth: edge.from === activeId || edge.to === activeId ? 2.6 : 1.8,
    },
  }))

  return { nodes, edges }
}

export function SystemGraph({ token }: { token: string }) {
  const [snapshot, setSnapshot] = useState<ArchitectureSnapshot | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [selected, setSelected] = useState<ArchitectureNode | null>(null)

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
    () => (snapshot ? toFlow(snapshot, selected?.id ?? null) : { nodes: [], edges: [] }),
    [snapshot, selected],
  )

  const onNodeClick = useCallback(
    (_: unknown, node: Node) => {
      setSelected((node.data as unknown as ArchitectureNode) ?? null)
    },
    [],
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

      <div className="graph-layout">
        <div className="graph-canvas">
          <ReactFlow
            nodes={flow.nodes}
            edges={flow.edges}
            nodeTypes={nodeTypes}
            onNodeClick={onNodeClick}
            onPaneClick={() => setSelected(null)}
            fitView
            fitViewOptions={{ padding: 0.18 }}
            minZoom={0.25}
            maxZoom={2}
            proOptions={{ hideAttribution: false }}
          >
            <Background gap={22} size={1} />
            <Controls showInteractive={false} />
          </ReactFlow>
        </div>

        {selected ? (
          <aside className="graph-details">
            <h3>{selected.name}</h3>
            <dl className="facts">
              <dt>id</dt>
              <dd>{selected.id}</dd>
              <dt>rodzaj</dt>
              <dd>{KIND_LABEL[selected.kind] ?? selected.kind}</dd>
              <dt>rola</dt>
              <dd>{selected.role ?? '—'}</dd>
              <dt>system</dt>
              <dd>{selected.system ?? '—'}</dd>
              <dt>jezyk</dt>
              <dd>{selected.language ?? '—'}</dd>
            </dl>
            <button type="button" className="graph-details__close" onClick={() => setSelected(null)}>
              Zamknij
            </button>
          </aside>
        ) : (
          <aside className="graph-details graph-details--empty">
            <p className="dim">Kliknij wezel, zeby zobaczyc szczegoly.</p>
          </aside>
        )}
      </div>
    </section>
  )
}
