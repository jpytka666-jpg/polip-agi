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

import { createContext, useCallback, useContext, useEffect, useMemo, useState } from 'react'
import { createPortal } from 'react-dom'
import {
  Background,
  Controls,
  Handle,
  MiniMap,
  Position,
  ReactFlow,
  type Edge,
  type MiniMapNodeProps,
  type Node,
  type NodeProps,
} from '@xyflow/react'
import '@xyflow/react/dist/style.css'
import { FlowEdge } from './FlowEdge'
import {
  fetchSystemGraph,
  runReadCommand,
  type ArchitectureNode,
  type ArchitectureSnapshot,
  type ReadCommandResult,
} from './api'
import { execute, type Command } from './commands'
import { commandFor } from './nodeCommands'
import { MODE, MODES, type TransportMode } from './transport'

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

/** To samo co KIND_LABEL, tylko powiedziane tak, zeby nie trzeba bylo znac zargonu. */
const KIND_PLAIN: Record<string, string> = {
  repository: 'caly projekt — najwyzsze pudlo, w ktorym siedzi reszta',
  directory: 'folder — pudlo grupujace pliki',
  module: 'kawalek programu odpowiedzialny za jedna rzecz',
  file: 'pojedynczy plik z kodem',
  dependency: 'cudza biblioteka, ktorej uzywamy',
  runtime: 'program, ktory naprawde chodzi i pracuje',
}

/** Kolory rodzajow na miniaturze - te same, co kropka przy nazwie na karcie. */
const MINIMAP_COLOR: Record<string, string> = {
  repository: '#b8860b',
  directory: '#8a6d3b',
  module: '#2f7d32',
  file: '#1e5aa8',
  dependency: '#6b6b6b',
  runtime: '#0b3d4a',
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
const NODE_H = 92
// Wieksze korytarze: przewody prowadzone pod katem prostym potrzebuja miejsca
// miedzy kolumnami, inaczej nakladaja sie na siebie.
const GAP_Y = 46
const GAP_X = 200

type NodeData = ArchitectureNode & { active: boolean; dimmed: boolean }

type MiniMapInteraction = {
  showPreview: (nodeId: string | null) => void
  selectNode: (nodeId: string) => void
}

const MiniMapInteractionContext = createContext<MiniMapInteraction>({
  showPreview: () => undefined,
  selectNode: () => undefined,
})

/**
 * Wlasny prostokat minimapy dodaje jedynie semantyczny hover. Nawigacja, pan i zoom
 * nadal naleza do React Flow, wiec nie powstaje drugi silnik grafu.
 */
function InteractiveMiniMapNode({
  id,
  x,
  y,
  width,
  height,
  style,
  color,
  strokeColor,
  strokeWidth,
  className,
  borderRadius,
  selected,
}: MiniMapNodeProps) {
  const { showPreview, selectNode } = useContext(MiniMapInteractionContext)
  const fill = color ?? (
    typeof style?.background === 'string'
      ? style.background
      : typeof style?.backgroundColor === 'string'
        ? style.backgroundColor
        : undefined
  )

  return (
    <rect
      className={[
        'react-flow__minimap-node',
        selected ? 'selected' : '',
        className,
      ].filter(Boolean).join(' ')}
      data-node-id={id}
      x={x}
      y={y}
      rx={borderRadius}
      ry={borderRadius}
      width={width}
      height={height}
      style={{ fill, stroke: strokeColor, strokeWidth }}
      role="button"
      tabIndex={0}
      aria-label={`Pokaz szczegoly wezla ${id}`}
      onMouseEnter={() => showPreview(id)}
      onMouseLeave={() => showPreview(null)}
      onPointerDown={(event) => event.stopPropagation()}
      onClick={(event) => {
        event.stopPropagation()
        selectNode(id)
      }}
      onKeyDown={(event) => {
        if (event.key === 'Enter' || event.key === ' ') {
          event.preventDefault()
          selectNode(id)
        }
      }}
    />
  )
}

function ArchNode({ data }: NodeProps) {
  const d = data as unknown as NodeData
  // Popup renderujemy PRZEZ PORTAL do body. Wewnatrz plotna position: fixed liczy
  // sie wzgledem transformowanego rodzica, wiec okno wedrowaloby razem z widokiem
  // i skalowalo sie z zoomem. Poza plotnem siedzi na srodku ekranu i ma staly rozmiar.
  const [open, setOpen] = useState(false)

  const cls = [
    'gnode',
    `gnode--${d.kind}`,
    d.active ? 'gnode--active' : '',
    d.dimmed ? 'gnode--dimmed' : '',
  ]
    .filter(Boolean)
    .join(' ')

  return (
    <div className={cls} onMouseEnter={() => setOpen(true)} onMouseLeave={() => setOpen(false)}>
      <Handle type="target" position={Position.Left} />
      <span className="gnode__icon" aria-hidden="true">
        {KIND_ICON[d.kind] ?? '·'}
      </span>
      <span className="gnode__text">
        <span className="gnode__name">{d.name}</span>
        <span className="gnode__kind">{KIND_LABEL[d.kind] ?? d.kind}</span>
        {d.role ? <span className="gnode__role">{d.role}</span> : null}
      </span>

      {/* Okno szczegolow: zawsze na srodku ekranu, staly rozmiar, mysz moze na nie wejsc. */}
      {open
        ? createPortal(
            <>
              {/* Klikniecie w tlo zamyka okno - tak jak zjechanie z niego myszka. */}
              <div className="gpop__backdrop" onClick={() => setOpen(false)} />
              <div
                className="gpop"
                role="tooltip"
                onMouseEnter={() => setOpen(true)}
                onMouseLeave={() => setOpen(false)}
              >
              <div className="gpop__body">
                <span className="gpop__title">{d.name}</span>
                <dl className="gpop__facts">
                  <dt>co to jest</dt>
                  <dd>{KIND_PLAIN[d.kind] ?? KIND_LABEL[d.kind] ?? d.kind}</dd>
                  <dt>do czego sluzy</dt>
                  <dd>{d.role ?? 'nie opisano'}</dd>
                  <dt>czesc czego</dt>
                  <dd>{d.system ?? 'nie przypisano'}</dd>
                  <dt>w czym napisane</dt>
                  <dd>{d.language === 'mixed' ? 'kilka roznych jezykow' : (d.language ?? 'nie dotyczy')}</dd>
                  <dt>nazwa w kodzie</dt>
                  <dd>{d.id}</dd>
                </dl>
                  <p className="gpop__note">
                    Mozna zaznaczyc i skopiowac. Okno znika, gdy odsuniesz myszke
                    albo klikniesz w szare tlo.
                  </p>
                </div>
              </div>
            </>,
            document.body,
          )
        : null}

      <Handle type="source" position={Position.Right} />
    </div>
  )
}

const nodeTypes = { arch: ArchNode }
const edgeTypes = { flow: FlowEdge }

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
  // Wylaczone nitki zyja TYLKO w widoku - API nie ma zadnej sciezki zapisu.
  const [muted, setMuted] = useState<Set<string>>(new Set())

  // Zgaszone rodzaje ruchu. Osobno od `muted`: tam chowa sie pojedyncza trase,
  // tu caly rodzaj naraz - wszystkie lodzie albo wszystkie samoloty.
  const [hiddenModes, setHiddenModes] = useState<Set<TransportMode>>(new Set())

  // Wynik ostatniego recznego wywolania z panelu. Trzymany osobno dla kazdego wezla,
  // zeby przejscie na inny wezel nie pokazywalo cudzej odpowiedzi.
  const [result, setResult] = useState<{ nodeId: string; data: ReadCommandResult } | null>(null)
  const [running, setRunning] = useState(false)
  const [runError, setRunError] = useState<string | null>(null)
  const [minimapPreviewId, setMinimapPreviewId] = useState<string | null>(null)

  const toggleEdge = useCallback((id: string) => {
    setMuted((prev) => {
      const next = new Set(prev)
      if (next.has(id)) next.delete(id)
      else next.add(id)
      return next
    })
  }, [])

  const toggleMode = useCallback((mode: TransportMode) => {
    setHiddenModes((prev) => {
      const next = new Set(prev)
      if (next.has(mode)) next.delete(mode)
      else next.add(mode)
      return next
    })
  }, [])

  const selectNode = useCallback((nodeId: string | null) => {
    if (nodeId === null) setSelected(null)
  }, [])

  // Kazda akcja diagramu wychodzi tedy - takze te, ktorych dzis wykonac nie wolno.
  // Odrzucenie wraca z powodem i ladnie widac je w konsoli zamiast cichego nic.
  const run = useCallback(
    (cmd: Command) => {
      const result = execute(cmd, { toggleEdge, toggleMode, selectNode })
      if (!result.ok) console.warn(result.reason)
      return result
    },
    [toggleEdge, toggleMode, selectNode],
  )

  const runNodeCommand = useCallback(
    async (node: ArchitectureNode) => {
      const cmd = commandFor(node.id)
      if (!cmd) return
      setRunning(true)
      setRunError(null)
      try {
        const data = await runReadCommand(cmd.path, token)
        setResult({ nodeId: node.id, data })
      } catch (err) {
        setResult(null)
        setRunError(err instanceof Error ? err.message : 'Nie udalo sie polaczyc z serwerem.')
      } finally {
        setRunning(false)
      }
    },
    [token],
  )

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
      // Wymiary podane wprost. React Flow potrafi je zmierzyc po renderze, ale
      // minimapa rysuje wylacznie wezly, ktore znaja swoj rozmiar - bez tego
      // pokazywala pusta ramke z sama maska widoku.
      width: NODE_W,
      height: NODE_H,
    }))

    // Stopien wezla docelowego - realna liczba z danych, uzywana jako liczba wagonow.
    // To NIE jest zmierzony wolumen ruchu; API takiego pola nie ma i nie udajemy, ze ma.
    const degree = new Map<string, number>()
    for (const e of snapshot.edges) {
      degree.set(e.to, (degree.get(e.to) ?? 0) + 1)
      degree.set(e.from, (degree.get(e.from) ?? 0) + 1)
    }
    const kindOf = new Map(snapshot.nodes.map((n) => [n.id, n.kind]))

    // Zanurzenie = odleglosc od korzenia (repozytorium) liczona wszerz.
    // To miara struktury grafu, NIE glebokosc w jakiejkolwiek sieci.
    const adj = new Map<string, string[]>()
    for (const e of snapshot.edges) {
      adj.set(e.from, [...(adj.get(e.from) ?? []), e.to])
      adj.set(e.to, [...(adj.get(e.to) ?? []), e.from])
    }
    const root = snapshot.nodes.find((n) => n.kind === 'repository')?.id ?? snapshot.nodes[0]?.id
    const depthOf = new Map<string, number>()
    if (root) {
      depthOf.set(root, 0)
      const queue = [root]
      while (queue.length) {
        const cur = queue.shift()!
        for (const nx of adj.get(cur) ?? []) {
          if (!depthOf.has(nx)) {
            depthOf.set(nx, (depthOf.get(cur) ?? 0) + 1)
            queue.push(nx)
          }
        }
      }
    }

    const edges: Edge[] = snapshot.edges.map((edge) => {
      const lit = activeId !== null && (edge.from === activeId || edge.to === activeId)
      // Krawedz dotykajaca runtime laczy osobne procesy, czyli biegnie przez siec -
      // rysujemy ja jako trase lotnicza. Reszta zyje w jednym procesie: tor.
      // Trzy rodzaje trasy, kazdy wyprowadzony z danych, nie zgadniety:
      //  sea  - relacja wyjscia poza system (egress) albo wezel bramy wyjsciowej,
      //  air  - dotyka runtime, czyli laczy osobne procesy przez siec,
      //  rail - reszta, czyli polaczenie w jednym procesie.
      const touchesGate =
        edge.kind.includes('egress') ||
        edge.from.includes('ghost-gate') ||
        edge.to.includes('ghost-gate')
      const touchesRuntime =
        kindOf.get(edge.from) === 'runtime' || kindOf.get(edge.to) === 'runtime'
      const mode: 'rail' | 'air' | 'sea' = touchesGate ? 'sea' : touchesRuntime ? 'air' : 'rail'
      return {
        id: edge.id,
        source: edge.from,
        target: edge.to,
        type: 'flow',
        data: {
          label: edge.kind,
          enabled: !muted.has(edge.id),
          lit,
          dimmed: activeId !== null,
          cars: Math.max(1, Math.min(degree.get(edge.to) ?? 1, 6)),
          mode,
          modeEnabled: !hiddenModes.has(mode),
          depth: depthOf.get(edge.to) ?? depthOf.get(edge.from) ?? 0,
          onToggle: (edgeId: string) => run({ kind: 'toggle-edge', scope: 'view', edgeId }),
        },
      }
    })

    return { nodes, edges, onPath }
  }, [snapshot, selected, muted, hiddenModes, run])

  const selectArchitectureNode = useCallback((node: ArchitectureNode) => {
    setSelected(node)
    setMinimapPreviewId(null)
    setResult(null)
    setRunError(null)
  }, [])

  const onNodeClick = useCallback((_: unknown, node: Node) => {
    selectArchitectureNode(node.data as unknown as ArchitectureNode)
  }, [selectArchitectureNode])

  const onMiniMapNodeClick = useCallback((nodeId: string) => {
    const node = snapshot?.nodes.find((candidate) => candidate.id === nodeId)
    if (node) selectArchitectureNode(node)
  }, [selectArchitectureNode, snapshot])

  const minimapInteraction = useMemo<MiniMapInteraction>(() => ({
    showPreview: setMinimapPreviewId,
    selectNode: onMiniMapNodeClick,
  }), [onMiniMapNodeClick])

  if (error) {
    return (
      <section className="panel panel--error">
        <h2>Graf systemu</h2>
        <p role="alert">{error}</p>
      </section>
    )
  }

  const linked = selected ? Math.max(flow.onPath.size - 1, 0) : 0
  const minimapPreview = snapshot?.nodes.find((node) => node.id === minimapPreviewId) ?? null
  const minimapCommand = minimapPreview ? commandFor(minimapPreview.id) : null
  const minimapState = minimapPreview
    ? selected?.id === minimapPreview.id
      ? 'wybrany'
      : flow.onPath.has(minimapPreview.id)
        ? 'na aktywnej sciezce'
        : 'gotowy do wyboru'
    : null

  return (
    <section className="panel panel--graph">
      <h2>
        Graf systemu{' '}
        <span className="badge">
          {flow.nodes.length} wezlow / {flow.edges.length} krawedzi
        </span>
        {selected ? <span className="badge">sciezka: {linked} sasiadow</span> : null}
        {muted.size ? <span className="badge badge--muted">ukryte nitki: {muted.size}</span> : null}
        {!selected ? (
          <span className="graph-hint">
            klik wezla = sciezka · przelacznik na przewodzie ukrywa go tylko na plotnie
          </span>
        ) : null}
      </h2>

      <div className="lane-bar" role="group" aria-label="Rodzaje ruchu">
        {MODES.map((mode) => {
          const on = !hiddenModes.has(mode)
          const spec = MODE[mode]
          return (
            <button
              key={mode}
              type="button"
              className={`lane-bar__btn lane-bar__btn--${mode}${on ? ' lane-bar__btn--on' : ''}`}
              aria-pressed={on}
              title={`${spec.meaning}; ${spec.speedWhy}`}
              onClick={() => run({ kind: 'toggle-mode', scope: 'view', mode })}
            >
              <span className="lane-bar__glyph">{spec.glyph}</span>
              <span className="lane-bar__name">{spec.title}</span>
              <span className="lane-bar__state">{on ? 'widac' : 'ukryte'}</span>
            </button>
          )
        })}
        <span className="lane-bar__hint">
          Filtry widoku. Chowaja linie na rysunku - niczego nie wylaczaja na maszynie.
        </span>
      </div>

      <div className={`graph-layout${selected ? ' graph-layout--split' : ''}`}>
        <div className="graph-canvas">
          <ReactFlow
            nodes={flow.nodes}
            edges={flow.edges}
            nodeTypes={nodeTypes}
            edgeTypes={edgeTypes}
            onNodeClick={onNodeClick}
            onPaneClick={() => setSelected(null)}
            fitView
            fitViewOptions={{ padding: 0.12, minZoom: 0.12, maxZoom: 0.85 }}
            // Oddalac wolno do woli.
            minZoom={0.12}
            maxZoom={2.5}
          >
            <Background gap={24} size={1} />
            <MiniMapInteractionContext.Provider value={minimapInteraction}>
              <MiniMap
                pannable
                zoomable
                zoomStep={2}
                onMouseLeave={() => setMinimapPreviewId(null)}
                nodeComponent={InteractiveMiniMapNode}
                nodeStrokeWidth={2}
                ariaLabel="Nawigator grafu: przeciagaj widok, przyblizaj kolkiem, najedz po podglad albo kliknij po szczegoly"
                // Ten sam kolor co kropka rodzaju na karcie, wiec miniatura czyta sie
                // tak samo jak plotno.
                nodeColor={(node) =>
                  MINIMAP_COLOR[(node.data as unknown as ArchitectureNode).kind] ?? '#6b6b6b'
                }
              />
            </MiniMapInteractionContext.Provider>
            <Controls showInteractive={false} />
          </ReactFlow>
          {minimapPreview ? (
            <aside className="minimap-preview" aria-label={`Podglad ${minimapPreview.name}`}>
              <div className="minimap-preview__head">
                <strong>{minimapPreview.name}</strong>
                <span>{KIND_LABEL[minimapPreview.kind] ?? minimapPreview.kind}</span>
              </div>
              <p>{minimapPreview.role ?? 'Rola nieopisana w snapshotcie.'}</p>
              <dl>
                <dt>system</dt>
                <dd>{minimapPreview.system ?? 'nie przypisano'}</dd>
                <dt>stan</dt>
                <dd>{minimapState}</dd>
                <dt>funkcja</dt>
                <dd>{minimapCommand?.what ?? 'brak komendy GET'}</dd>
              </dl>
              <small>Kliknij mini-wezel, aby przypiac pelne szczegoly.</small>
            </aside>
          ) : null}
        </div>

        {selected ? (
          (() => {
            const cmd = commandFor(selected.id)
            const shown = result?.nodeId === selected.id ? result.data : null
            return (
              <aside className="graph-details">
                <h3>{selected.name}</h3>
                <dl className="facts">
                  <dt>rodzaj</dt>
                  <dd>{KIND_LABEL[selected.kind] ?? selected.kind}</dd>
                  <dt>rola</dt>
                  <dd>{selected.role ?? '—'}</dd>
                  <dt>capability</dt>
                  <dd className="facts__gap">niewystawione po HTTP (brak GET /v1/modules)</dd>
                  <dt>system</dt>
                  <dd>{selected.system ?? '—'}</dd>
                  <dt>jezyk</dt>
                  <dd>{selected.language ?? '—'}</dd>
                  <dt>sasiedzi</dt>
                  <dd>{linked}</dd>
                  <dt>nazwa w kodzie</dt>
                  <dd>{selected.id}</dd>
                </dl>

                {cmd ? (
                  <div className="exec">
                    <div className="exec__head">
                      <code className="exec__path">GET {cmd.path}</code>
                      <button
                        type="button"
                        className="exec__run"
                        disabled={running}
                        onClick={() => void runNodeCommand(selected)}
                      >
                        {running ? 'Czekam…' : 'Wykonaj'}
                      </button>
                    </div>
                    <p className="exec__what">{cmd.what}</p>

                    {runError ? <p className="exec__error">{runError}</p> : null}

                    {shown ? (
                      <>
                        <p
                          className={`exec__status${shown.ok ? ' exec__status--ok' : ' exec__status--bad'}`}
                        >
                          {shown.ok ? 'Odpowiedzial' : 'Blad'} {shown.status} · {shown.ms} ms
                        </p>
                        {shown.warning ? <p className="exec__error">{shown.warning}</p> : null}
                        <pre className="exec__json">
                          {typeof shown.body === 'string'
                            ? shown.body
                            : JSON.stringify(shown.body, null, 2)}
                        </pre>
                      </>
                    ) : null}
                  </div>
                ) : (
                  <p className="exec__none">
                    BRAK KOMENDY — serwer nie wystawia dla tego elementu zadnego wywolania
                    odczytu.
                  </p>
                )}

                <button
                  type="button"
                  className="graph-details__close"
                  onClick={() => setSelected(null)}
                >
                  Zamknij
                </button>
              </aside>
            )
          })()
        ) : null}
      </div>
    </section>
  )
}
