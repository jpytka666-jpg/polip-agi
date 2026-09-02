/*
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Opus 5
TIMESTAMP: 2026-09-02 08:45:00
REASON FOR CREATION: Graf systemu w stylu kanwy n8n - czytelne wezly i zywa sciezka po kliknieciu.
MECHANICS: Wezel 280x96 na jasnej karcie (pomarancz-biel) z czarnym tekstem 18/14 px - najwyzszy
kontrast na oliwkowej kanwie; reszta plotna zostaje oliwkowa. Klikniecie wybiera
wezel: wszystkie krawedzie wchodzace i wychodzace dostaja 3 px w kolorze akcentu, a pozostale
spadaja do 20% krycia - widac cala zywa sciezke, nie pojedyncze polaczenie. Uklad liczony
lokalnie: kolumny wedlug rodzaju, a kolejnosc w kolumnie ustalana metoda barycentryczna, wiec
polaczone wezly laduja obok siebie zamiast na przeciwleglych koncach. Wezly bez zadnej krawedzi
sa dociagane do srodka wlasnej kolumny, a nie zostawiane na pustyni.
SYSTEM PART: Control Room / widok architektury.
ARCHITECTURE FUNCTION: Operator widzi strukture i przeplyw; dane z tego samego zrodla co strona
serwerowa, wiec nie ma drugiej prawdy.
DEPENDENCIES/LINKS: api.ts (fetchSystemGraph), @xyflow/react (bez nowych zaleznosci).
TECH STACK: TypeScript 6 + React 19 + React Flow 12, swiadomie zamiast Rusta - domyslnego jezyka.
  (1) MUSI: rysowac interaktywna kanwe w przegladarce, z hover, wyborem, zoomem i minimapa.
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
  MiniMap,
  Position,
  ReactFlow,
  type Edge,
  type Node,
  type NodeProps,
} from '@xyflow/react'
import '@xyflow/react/dist/style.css'
import { fetchSystemGraph, type ArchitectureNode, type ArchitectureSnapshot } from './api'

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

const KIND_ICON: Record<string, string> = {
  repository: '▣',
  directory: '▤',
  module: '◆',
  file: '▸',
  dependency: '◇',
  runtime: '●',
}

const NODE_W = 280
const NODE_H = 96
const GAP_Y = 80
const GAP_X = 120

type NodeData = ArchitectureNode & { active: boolean; dimmed: boolean }

function ArchNode({ data }: NodeProps) {
  const d = data as unknown as NodeData
  const tip = [
    `id: ${d.id}`,
    `rodzaj: ${KIND_LABEL[d.kind] ?? d.kind}`,
    `rola: ${d.role ?? 'brak'}`,
  ].join('\n')

  const cls = [
    'gnode',
    `gnode--${d.kind}`,
    d.active ? 'gnode--active' : '',
    d.dimmed ? 'gnode--dimmed' : '',
  ]
    .filter(Boolean)
    .join(' ')

  return (
    <div className={cls} title={tip}>
      <Handle type="target" position={Position.Left} />
      <span className="gnode__icon" aria-hidden="true">
        {KIND_ICON[d.kind] ?? '·'}
      </span>
      <span className="gnode__text">
        <span className="gnode__name">{d.name}</span>
        <span className="gnode__kind">
          {KIND_LABEL[d.kind] ?? d.kind}
          {d.role ? ` · ${d.role}` : ''}
        </span>
      </span>
      <Handle type="source" position={Position.Right} />
    </div>
  )
}

const nodeTypes = { arch: ArchNode }

/**
 * Uklad: kolumny wedlug rodzaju, kolejnosc w kolumnie ustalana barycentrycznie.
 * Wezel laduje naprzeciw sredniej pozycji swoich sasiadow, wiec sciezki biegna
 * poziomo zamiast krzyzowac cala kanwe. Wezly bez krawedzi ida na srodek kolumny.
 */
function layout(snapshot: ArchitectureSnapshot) {
  const columns = new Map<number, string[]>()
  const colOf = new Map<string, number>()

  for (const node of snapshot.nodes) {
    const c = COLUMN[node.kind] ?? 6
    colOf.set(node.id, c)
    if (!columns.has(c)) columns.set(c, [])
    columns.get(c)!.push(node.id)
  }

  const neighbours = new Map<string, string[]>()
  const degree = new Map<string, number>()
  for (const e of snapshot.edges) {
    neighbours.set(e.from, [...(neighbours.get(e.from) ?? []), e.to])
    neighbours.set(e.to, [...(neighbours.get(e.to) ?? []), e.from])
    degree.set(e.from, (degree.get(e.from) ?? 0) + 1)
    degree.set(e.to, (degree.get(e.to) ?? 0) + 1)
  }

  const rank = new Map<string, number>()
  for (const ids of columns.values()) {
    ids.forEach((id, i) => rank.set(id, i))
  }

  // Kilka przebiegow wystarcza dla grafu tej wielkosci.
  for (let pass = 0; pass < 6; pass += 1) {
    for (const [, ids] of [...columns.entries()].sort((a, b) => a[0] - b[0])) {
      const scored = ids.map((id) => {
        const ns = neighbours.get(id) ?? []
        if (ns.length === 0) {
          // Osierocony: srodek wlasnej kolumny, zeby nie lezal na uboczu.
          return { id, score: (ids.length - 1) / 2 }
        }
        const sum = ns.reduce((acc, n) => acc + (rank.get(n) ?? 0), 0)
        return { id, score: sum / ns.length }
      })
      scored.sort((a, b) => a.score - b.score)
      scored.forEach((s, i) => rank.set(s.id, i))
      columns.set(colOf.get(scored[0]?.id ?? '') ?? 0, scored.map((s) => s.id))
    }
  }

  const pos = new Map<string, { x: number; y: number }>()
  for (const [col, ids] of columns.entries()) {
    ids.forEach((id, i) => {
      pos.set(id, { x: col * (NODE_W + GAP_X), y: i * (NODE_H + GAP_Y) })
    })
  }
  return { pos, degree }
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

  const flow = useMemo(() => {
    if (!snapshot) return { nodes: [] as Node[], edges: [] as Edge[], onPath: new Set<string>() }
    const { pos } = layout(snapshot)
    const activeId = selected?.id ?? null

    // Zywa sciezka: kazda krawedz dotykajaca wybranego wezla, w obie strony.
    const onPath = new Set<string>()
    if (activeId) {
      for (const e of snapshot.edges) {
        if (e.from === activeId || e.to === activeId) {
          onPath.add(e.from)
          onPath.add(e.to)
        }
      }
    }

    const nodes: Node[] = snapshot.nodes.map((node) => ({
      id: node.id,
      type: 'arch',
      position: pos.get(node.id) ?? { x: 0, y: 0 },
      data: {
        ...node,
        active: node.id === activeId,
        dimmed: activeId !== null && !onPath.has(node.id),
      } as unknown as Record<string, unknown>,
      draggable: true,
    }))

    const edges: Edge[] = snapshot.edges.map((edge) => {
      const lit = activeId !== null && (edge.from === activeId || edge.to === activeId)
      return {
        id: edge.id,
        source: edge.from,
        target: edge.to,
        label: edge.kind,
        animated: lit,
        className: lit ? 'edge--lit' : activeId ? 'edge--dim' : '',
        style: { strokeWidth: lit ? 3 : 1.8 },
      }
    })

    return { nodes, edges, onPath }
  }, [snapshot, selected])

  const onNodeClick = useCallback((_: unknown, node: Node) => {
    setSelected((node.data as unknown as ArchitectureNode) ?? null)
  }, [])

  if (error) {
    return (
      <section className="panel panel--error">
        <h2>Graf systemu</h2>
        <p role="alert">{error}</p>
      </section>
    )
  }

  const linked = selected ? Math.max(flow.onPath.size - 1, 0) : 0

  return (
    <section className="panel panel--graph">
      <h2>
        Graf systemu{' '}
        <span className="badge">
          {flow.nodes.length} wezlow / {flow.edges.length} krawedzi
        </span>
        {selected ? <span className="badge">sciezka: {linked} sasiadow</span> : null}
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
            // Dopasowanie do calosci potrafilo zejsc do 0.33 i wtedy wezel 280x96
            // rysowal sie nieczytelnie. Podloga 0.68 trzyma tekst
            // w rozmiarze, a operator doscrolluje reszte.
            fitViewOptions={{ padding: 0.12, minZoom: 0.68, maxZoom: 1 }}
            minZoom={0.3}
            maxZoom={2}
          >
            <Background gap={24} size={1} />
            <MiniMap pannable zoomable nodeStrokeWidth={2} />
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
              <dt>sasiedzi</dt>
              <dd>{linked}</dd>
            </dl>
            <button type="button" className="graph-details__close" onClick={() => setSelected(null)}>
              Zamknij
            </button>
          </aside>
        ) : (
          <aside className="graph-details graph-details--empty">
            <p className="dim">Kliknij wezel, zeby podswietlic jego sciezke.</p>
          </aside>
        )}
      </div>
    </section>
  )
}
