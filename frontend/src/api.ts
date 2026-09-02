/*
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Opus 5
TIMESTAMP: 2026-09-01 23:20:00
REASON FOR CREATION: Warstwa dostepu Control Room do API Darkstara - wylacznie odczyt (Task 10).
MECHANICS: Dwa zapytania GET: /v1/system-graph i /v1/gateway/status. Typy odwzorowuja kontrakty
z darkstar-core (ArchitectureSnapshot, GatewayStatus). Nie ma tu ani jednej funkcji zapisujacej -
sterowanie brama nie istnieje w API i nie wolno go tu udawac. PIN przychodzi wylacznie z pamieci
biezacej karty i nigdy z kodu ani localStorage.
SYSTEM PART: Control Room / warstwa danych.
ARCHITECTURE FUNCTION: Jedyne miejsce, w ktorym interfejs dotyka sieci. Reszta widokow dostaje
gotowe dane, wiec nie da sie przypadkiem wywolac czegos zmieniajacego z komponentu.
DEPENDENCIES/LINKS: darkstar-server /v1/system-graph, /v1/gateway/status.
TECH STACK: TypeScript 6, swiadomie zamiast Rusta - domyslnego jezyka tego projektu.
  (1) MUSI: dzialac w przegladarce operatora, odswiezac widok i renderowac graf.
  (2) DLACZEGO NIE RUST: przegladarka wykonuje JavaScript; Rust przez WebAssembly nadal
      potrzebowalby warstwy TS do DOM i React Flow, wiec dolozylby narzedzi bez zysku.
      Zaufany control plane ZOSTAJE w Ruscie - przegladarka niczego nie wykonuje w systemie.
  (3) TRACIMY: kontrole typow wspolna z rdzeniem. Rekompensata: typy tutaj sa recznym
      odbiciem kontraktow z darkstar-core, a bledna adresacja i tak jest odrzucana po stronie Rusta.
LOCAL WORKSPACE: D:\codex-fresh-2026-08-28\worktrees\polip-agi-darkstar-plan
GIT COMMIT: PENDING
GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
==========================================
*/

import { authorizationHeaders } from './operatorPin'

export type GatewayHealth = 'offline' | 'starting' | 'ready' | 'degraded' | 'failed'
export type GatewayMode = 'ethernet' | 'hotspot'

export interface GatewayStatus {
  mode: GatewayMode
  health: GatewayHealth
  upstream_interface: string
  downstream_interface: string
  downstream_cidr: string
  downstream_subnet: string
  connection_profile: string
  connected_clients: number
  last_verified_unix_ms: number
}

export interface ArchitectureNode {
  id: string
  kind: string
  name: string
  system?: string | null
  role?: string | null
  language?: string | null
}

export interface ArchitectureEdge {
  id: string
  from: string
  to: string
  kind: string
}

export interface ArchitectureSnapshot {
  nodes: ArchitectureNode[]
  edges: ArchitectureEdge[]
}

async function getJson<T>(path: string, pin: string): Promise<T> {
  const response = await fetch(path, {
    method: 'GET',
    headers: authorizationHeaders(pin),
  })
  if (response.status === 401) {
    throw new Error('Brak autoryzacji - podaj PIN operatora.')
  }
  if (response.status === 503) {
    throw new Error('Brama nieodczytywalna - host nie odpowiada.')
  }
  if (!response.ok) {
    throw new Error(`Blad ${response.status}`)
  }
  return (await response.json()) as T
}

export function fetchGatewayStatus(token: string): Promise<GatewayStatus> {
  return getJson<GatewayStatus>('/v1/gateway/status', token)
}

export function fetchSystemGraph(token: string): Promise<ArchitectureSnapshot> {
  return getJson<ArchitectureSnapshot>('/v1/system-graph', token)
}

export interface ContextHealth {
  local_cbms_ok: boolean
  remote_e_ok: boolean
}

export interface ContextCollection {
  id: string
  name: string
  dimension: number | null
}

export interface ContextListing {
  served_by: 'local_cbms' | 'remote_e'
  collections: ContextCollection[]
}

export function fetchContextHealth(token: string): Promise<ContextHealth> {
  return getJson<ContextHealth>('/v1/context/health', token)
}

export function searchContext(
  token: string,
  query: string,
  limit = 10,
): Promise<ContextListing> {
  const params = new URLSearchParams({ q: query, limit: String(limit) })
  return getJson<ContextListing>(`/v1/context/search?${params.toString()}`, token)
}

/** Wynik recznego wywolania z panelu operatora. Blad NIE jest chowany pod sukces. */
export interface ReadCommandResult {
  ok: boolean
  status: number
  /** Ile trwalo, w milisekundach. */
  ms: number
  /** Odpowiedz serwera. JSON, jesli sie sparsowal; inaczej surowy tekst. */
  body: unknown
  /** Ostrzezenie, gdy przyszlo cos innego niz JSON - np. strona HTML zamiast API. */
  warning: string | null
}

/**
 * Wykonuje pojedyncza komende ODCZYTU z panelu operatora.
 *
 * Zawsze GET. Zwraca prawdziwy kod odpowiedzi - 401 i 503 wracaja jako `ok: false`
 * razem z trescia, ktora przyslal serwer. Nic tu nie udaje sukcesu.
 */
export async function runReadCommand(path: string, pin: string): Promise<ReadCommandResult> {
  const started = performance.now()
  const response = await fetch(path, {
    method: 'GET',
    headers: authorizationHeaders(pin),
  })
  const text = await response.text()
  let body: unknown = text
  let warning: string | null = null
  try {
    body = JSON.parse(text)
  } catch {
    // Kod 200 z trescia, ktora nie jest JSON-em, to najczesciej strona HTML
    // podstawiona przez posrednika - nie odpowiedz API. Sukces sie tu nie udaje.
    warning = text.trimStart().startsWith('<')
      ? 'To nie odpowiedz API, tylko strona HTML - zapytanie nie doszlo do serwera Darkstar.'
      : 'Odpowiedz nie jest JSON-em.'
  }
  return {
    ok: response.ok && warning === null,
    status: response.status,
    ms: Math.round(performance.now() - started),
    body,
    warning,
  }
}

export interface GitCommandResult {
  stdout: string
  stderr: string
  exit_code: number
}

export interface GitRailSnapshot {
  worktree: string
  status: GitCommandResult
  log: GitCommandResult
  head: GitCommandResult
  upstream: GitCommandResult
}

/** Odczyt lokalnego Windows WORKTREE przez middleware Vite, nigdy przez kontener CBMS. */
export function fetchGitRail(): Promise<GitRailSnapshot> {
  return getJson<GitRailSnapshot>('/__darkstar/git', '')
}

/** Aktualizuje wylacznie referencje origin. Nie wykonuje checkout, merge ani reset. */
export async function fetchGitOrigin(): Promise<GitRailSnapshot> {
  const response = await fetch('/__darkstar/git/fetch', { method: 'POST' })
  if (!response.ok) {
    const body = await response.json().catch(() => null) as { error?: string } | null
    throw new Error(body?.error ?? `Fetch nie powiodl sie (${response.status}).`)
  }
  return (await response.json()) as GitRailSnapshot
}
