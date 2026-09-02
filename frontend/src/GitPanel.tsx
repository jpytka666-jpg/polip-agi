/*
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: GPT-5 Codex
TIMESTAMP: 2026-09-02 03:20:00 Europe/London
REASON FOR CREATION: Pokazanie operatorowi stanu Git z lokalnego Windows WORKTREE, a nie z kontenera CBMS.
==========================================
*/

import { useCallback, useEffect, useMemo, useState } from 'react'
import { fetchGitOrigin, fetchGitRail, type GitRailSnapshot } from './api'

type RailState = 'dirty' | 'local' | 'synced' | 'unknown'

type GitCommit = {
  sha: string
  parents: string[]
  decorations: string
  subject: string
}

type GitRailView = {
  branch: string
  head: string
  dirty: boolean
  ahead: number
  behind: number
  hasUpstream: boolean
  commits: GitCommit[]
}

type GraphRow = {
  commit: GitCommit
  lane: number
  row: number
}

type GraphEdge = {
  from: GraphRow
  to: GraphRow
}

const ROW_HEIGHT = 48
const LANE_GAP = 18
const LANE_COLORS = ['#7cc4ff', '#bd93f9', '#ff8c69', '#50c878', '#d7ba7d']

function parseCommits(output: string): GitCommit[] {
  return output
    .split('\x1e')
    .map((record) => record.trim())
    .filter(Boolean)
    .map((record) => {
      const [sha = '', parents = '', decorations = '', ...subject] = record.split('\x1f')
      return {
        sha,
        parents: parents.split(' ').filter(Boolean),
        decorations,
        subject: subject.join('\x1f') || '(bez tematu)',
      }
    })
    .filter((commit) => /^[0-9a-f]{40}$/i.test(commit.sha))
}

function viewOf(rail: GitRailSnapshot): GitRailView {
  const statusLines = rail.status.stdout.trim().split(/\r?\n/)
  const summary = statusLines[0] ?? ''
  const branchPart = summary.replace(/^##\s*/, '').split('...')[0]?.trim()

  return {
    branch: branchPart || 'nieznany',
    head: rail.head.stdout.trim(),
    dirty: statusLines.slice(1).some((line) => line.trim()),
    ahead: Number(summary.match(/\bahead (\d+)/)?.[1] ?? 0),
    behind: Number(summary.match(/\bbehind (\d+)/)?.[1] ?? 0),
    hasUpstream: rail.upstream.exit_code === 0 && Boolean(rail.upstream.stdout.trim()),
    commits: parseCommits(rail.log.stdout),
  }
}

function stateOf(rail: GitRailSnapshot, view: GitRailView): RailState {
  if (rail.status.exit_code !== 0 || rail.head.exit_code !== 0) return 'unknown'
  if (view.dirty) return 'dirty'
  if (!view.hasUpstream || view.ahead > 0) return 'local'
  return 'synced'
}

function graphOf(commits: GitCommit[], dirty: boolean) {
  const visible = new Set(commits.map((commit) => commit.sha))
  const lanes: Array<string | null> = []
  const rows: GraphRow[] = []
  let maxLane = 0

  commits.forEach((commit, index) => {
    let lane = lanes.indexOf(commit.sha)
    if (lane < 0) {
      lane = lanes.findIndex((active) => active === null)
      if (lane < 0) lane = lanes.length
      lanes[lane] = commit.sha
    }

    rows.push({ commit, lane, row: index + (dirty ? 1 : 0) })
    maxLane = Math.max(maxLane, lane)

    const parents = commit.parents.filter((parent) => visible.has(parent))
    lanes[lane] = parents[0] ?? null
    parents.slice(1).forEach((parent) => {
      if (lanes.includes(parent)) return
      let parentLane = lanes.findIndex((active) => active === null)
      if (parentLane < 0) parentLane = lanes.length
      lanes[parentLane] = parent
      maxLane = Math.max(maxLane, parentLane)
    })

    const remaining = new Set(commits.slice(index + 1).map((next) => next.sha))
    lanes.forEach((active, activeLane) => {
      if (active && !remaining.has(active)) lanes[activeLane] = null
    })
  })

  const rowBySha = new Map(rows.map((row) => [row.commit.sha, row]))
  const edges: GraphEdge[] = rows.flatMap((row) =>
    row.commit.parents
      .map((parent) => rowBySha.get(parent))
      .filter((parent): parent is GraphRow => parent !== undefined)
      .map((parent) => ({ from: row, to: parent })),
  )

  return {
    rows,
    edges,
    graphWidth: Math.max(44, (maxLane + 1) * LANE_GAP + 24),
    height: Math.max(ROW_HEIGHT, (rows.length + (dirty ? 1 : 0)) * ROW_HEIGHT),
  }
}

function refsOf(decorations: string): string[] {
  return decorations
    .split(',')
    .flatMap((raw) => {
      const ref = raw.trim()
      return ref.startsWith('HEAD -> ') ? ['HEAD', ref.slice('HEAD -> '.length)] : [ref]
    })
    .filter(Boolean)
}

const STATE_LABEL: Record<RailState, string> = {
  dirty: 'dirty',
  local: 'brak origin',
  synced: 'na origin',
  unknown: 'brak danych',
}

export function GitPanel() {
  const [rail, setRail] = useState<GitRailSnapshot | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [action, setAction] = useState<'refresh' | 'fetch' | null>(null)

  const refresh = useCallback(async () => {
    setAction('refresh')
    try {
      setRail(await fetchGitRail())
      setError(null)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Nie udalo sie odczytac lokalnego WORKTREE.')
    } finally {
      setAction(null)
    }
  }, [])

  const fetchOrigin = useCallback(async () => {
    setAction('fetch')
    try {
      setRail(await fetchGitOrigin())
      setError(null)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'git fetch origin nie powiodl sie.')
    } finally {
      setAction(null)
    }
  }, [])

  useEffect(() => {
    let cancelled = false
    fetchGitRail()
      .then((next) => {
        if (!cancelled) {
          setRail(next)
          setError(null)
        }
      })
      .catch((err) => {
        if (!cancelled) {
          setError(err instanceof Error ? err.message : 'Nie udalo sie odczytac lokalnego WORKTREE.')
        }
      })
    return () => {
      cancelled = true
    }
  }, [])

  const view = useMemo(() => (rail ? viewOf(rail) : null), [rail])
  const state = useMemo(() => (rail && view ? stateOf(rail, view) : 'unknown'), [rail, view])
  const graph = useMemo(() => graphOf(view?.commits ?? [], view?.dirty ?? false), [view])
  const relation = view?.hasUpstream
    ? `ahead ${view.ahead} / behind ${view.behind}`
    : 'brak origin'
  const headRow = graph.rows.find((row) => row.commit.sha === view?.head)

  return (
    <section className={`panel git-panel git-panel--${state}`} aria-labelledby="git-panel-title">
      <div className="git-panel__heading">
        <div>
          <h2 id="git-panel-title">Git · graf commitow</h2>
          <p title={view?.branch}>{view?.branch ?? 'lokalny worktree'}</p>
        </div>
        <span className="git-panel__state">{STATE_LABEL[state]}</span>
      </div>
      {error ? <p className="git-panel__error" role="alert">{error}</p> : null}
      <div className="git-graph" aria-label="Drzewo commitow z galeziami i merge">
        {view?.commits.length ? (
          <div className="git-graph__canvas" style={{ height: graph.height }}>
            <svg
              className="git-graph__rails"
              width={graph.graphWidth}
              height={graph.height}
              viewBox={`0 0 ${graph.graphWidth} ${graph.height}`}
              aria-hidden="true"
            >
              {graph.edges.map(({ from, to }) => {
                const x1 = 14 + from.lane * LANE_GAP
                const y1 = from.row * ROW_HEIGHT + ROW_HEIGHT / 2
                const x2 = 14 + to.lane * LANE_GAP
                const y2 = to.row * ROW_HEIGHT + ROW_HEIGHT / 2
                const bend = Math.min(18, Math.max(8, (y2 - y1) / 2))
                return (
                  <path
                    key={`${from.commit.sha}-${to.commit.sha}`}
                    d={`M ${x1} ${y1} C ${x1} ${y1 + bend}, ${x2} ${y2 - bend}, ${x2} ${y2}`}
                    stroke={LANE_COLORS[to.lane % LANE_COLORS.length]}
                  />
                )
              })}
              {view.dirty && headRow ? (
                <>
                  <path
                    className="git-graph__dirty-edge"
                    d={`M 14 ${ROW_HEIGHT / 2} C 14 ${ROW_HEIGHT}, ${14 + headRow.lane * LANE_GAP} ${headRow.row * ROW_HEIGHT}, ${14 + headRow.lane * LANE_GAP} ${headRow.row * ROW_HEIGHT + ROW_HEIGHT / 2}`}
                  />
                  <rect
                    className="git-graph__dirty-node"
                    x="9"
                    y={ROW_HEIGHT / 2 - 5}
                    width="10"
                    height="10"
                    transform={`rotate(45 14 ${ROW_HEIGHT / 2})`}
                  />
                </>
              ) : null}
              {graph.rows.map((row) => (
                <circle
                  key={row.commit.sha}
                  className={[
                    'git-graph__node',
                    row.commit.sha === view.head ? 'git-graph__node--head' : '',
                    row.commit.parents.length > 1 ? 'git-graph__node--merge' : '',
                  ].filter(Boolean).join(' ')}
                  cx={14 + row.lane * LANE_GAP}
                  cy={row.row * ROW_HEIGHT + ROW_HEIGHT / 2}
                  r={row.commit.parents.length > 1 ? 5.5 : 4.5}
                  fill={LANE_COLORS[row.lane % LANE_COLORS.length]}
                />
              ))}
            </svg>
            {view.dirty ? (
              <div className="git-graph__row git-graph__row--dirty" style={{ top: 0, paddingLeft: graph.graphWidth }}>
                <code>DIRTY</code>
                <span>zmiany na dysku</span>
              </div>
            ) : null}
            {graph.rows.map((row) => {
              const refs = refsOf(row.commit.decorations)
              return (
                <div
                  className="git-graph__row"
                  key={row.commit.sha}
                  style={{ top: row.row * ROW_HEIGHT, paddingLeft: graph.graphWidth }}
                >
                  <code>{row.commit.sha.slice(0, 7)}</code>
                  <div className="git-graph__label">
                    {refs.length || row.commit.parents.length > 1 ? (
                      <div className="git-graph__refs" title={refs.join(', ')}>
                        {row.commit.parents.length > 1 ? <span>merge</span> : null}
                        {refs.slice(0, 2).map((ref) => <span key={ref}>{ref}</span>)}
                      </div>
                    ) : null}
                    <span title={row.commit.subject}>{row.commit.subject}</span>
                  </div>
                </div>
              )
            })}
          </div>
        ) : (
          <p className="git-graph__empty">Czekam na graf lokalnego worktree…</p>
        )}
      </div>
      <div className="git-panel__bar">
        <button type="button" onClick={() => void refresh()} disabled={action !== null}>
          {action === 'refresh' ? 'Czytam…' : 'Odswiez'}
        </button>
        <button type="button" onClick={() => void fetchOrigin()} disabled={action !== null}>
          {action === 'fetch' ? 'Fetch…' : 'Fetch'}
        </button>
        <span>{view ? relation : 'czekam…'}</span>
      </div>
    </section>
  )
}
