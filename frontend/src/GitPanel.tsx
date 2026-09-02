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
import { fetchGitRail, type GitCommandResult, type GitRailSnapshot } from './api'

type RailState = 'dirty' | 'ahead' | 'synced' | 'unknown'

function outputOf(result: GitCommandResult): string {
  return result.stdout.trimEnd() || result.stderr.trimEnd() || 'brak wyniku'
}

function stateOf(rail: GitRailSnapshot): RailState {
  if (rail.status.exit_code !== 0) return 'unknown'

  const statusLines = rail.status.stdout.trim().split(/\r?\n/)
  if (statusLines.slice(1).some((line) => line.trim())) return 'dirty'

  const summary = statusLines[0] ?? ''
  const ahead = Number(summary.match(/\[ahead (\d+)\]/)?.[1] ?? 0)
  const behind = Number(summary.match(/\[behind (\d+)\]/)?.[1] ?? 0)
  if (ahead > 0) return 'ahead'
  if (rail.upstream.exit_code === 0 && behind === 0) return 'synced'
  return 'unknown'
}

const STATE_LABEL: Record<RailState, string> = {
  dirty: 'DIRTY · lokalne zmiany',
  ahead: 'AHEAD · lokalny przód',
  synced: 'SYNCED · zgodny z upstream',
  unknown: 'UNKNOWN · brak pewnego porównania',
}

const COMMANDS: Array<{ key: keyof Pick<GitRailSnapshot, 'status' | 'log' | 'head' | 'upstream'>; label: string }> = [
  { key: 'status', label: 'status -sb' },
  { key: 'log', label: 'log -15 --oneline --decorate' },
  { key: 'head', label: 'rev-parse HEAD' },
  { key: 'upstream', label: 'rev-parse @{u}' },
]

export function GitPanel() {
  const [rail, setRail] = useState<GitRailSnapshot | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)

  const refresh = useCallback(async () => {
    setLoading(true)
    try {
      setRail(await fetchGitRail())
      setError(null)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Nie udalo sie odczytac lokalnego WORKTREE.')
    } finally {
      setLoading(false)
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

  const state = useMemo(() => (rail ? stateOf(rail) : 'unknown'), [rail])

  return (
    <section className={`panel git-panel git-panel--${state}`} aria-labelledby="git-panel-title">
      <h2 id="git-panel-title">
        Git rail <span className="git-panel__state">{STATE_LABEL[state]}</span>
      </h2>
      <div className="git-panel__toolbar">
        <code>git -C {rail?.worktree ?? 'Windows WORKTREE'}</code>
        <button type="button" onClick={() => void refresh()} disabled={loading}>
          {loading ? 'Czytam…' : 'Odswiez'}
        </button>
      </div>
      {error ? <p className="git-panel__error" role="alert">{error}</p> : null}
      <div className="git-panel__commands">
        {COMMANDS.map(({ key, label }) => (
          <div className={`git-command git-command--${key}`} key={key}>
            <code>git -C {rail?.worktree ?? '...'} {label}</code>
            <pre>{rail ? outputOf(rail[key]) : 'czekam na lokalny WORKTREE…'}</pre>
          </div>
        ))}
      </div>
    </section>
  )
}
