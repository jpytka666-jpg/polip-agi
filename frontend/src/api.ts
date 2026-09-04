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

// Jawne rozszerzenie, tak samo jak w main.tsx: bez niego loader ESM Node-a nie znajdzie
// modulu i testy `node --test` nie ruszaja. Vite i tsc (allowImportingTsExtensions) to znosza.
import { authorizationHeaders } from './operatorPin.ts'

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

export type WorldServiceState = 'up' | 'down'

export interface WorldServiceStatus {
  state: WorldServiceState
  probe: 'http' | 'tcp'
  target: string
}

export interface WorldStatus {
  readOnly: true
  services: {
    darkstar: WorldServiceStatus
    headscale: WorldServiceStatus
    headplane: WorldServiceStatus
  }
}

export interface HeadplanePanelView {
  state: WorldServiceState | 'unknown'
  label: 'UP' | 'DOWN' | 'BRAK ODCZYTU'
  listen: string
}

/** Widok panelu zachowuje jawny stan nieznany; brak pomiaru nie staje sie awaria uslugi. */
export function headplanePanelView(status: WorldServiceStatus | undefined): HeadplanePanelView {
  if (!status) {
    return { state: 'unknown', label: 'BRAK ODCZYTU', listen: '127.0.0.1:3000' }
  }
  return {
    state: status.state,
    label: status.state === 'up' ? 'UP' : 'DOWN',
    listen: status.target,
  }
}

/**
 * Informacja o dostepie z sieci prywatnej - NIGDY link. Nasluch petli zwrotnej jest
 * celowym, docelowym stanem (Task 14, Step 14.9), wiec panel nie proponuje adresu
 * 192.168.2.1:3000 - takiego nasluchu nie ma i nie bedzie. Jedyna droga to tunel SSH
 * z zaufanej maszyny, tak samo jak Control Room.
 */
export function headplaneAccessNote(listen: string): string {
  return listen.startsWith('127.0.0.1')
    ? 'zamkniete dla sieci prywatnej - tylko przez tunel z zaufanej maszyny'
    : 'stan dostepu z sieci prywatnej nieznany'
}

/**
 * Adres tunelu SSH Windows -> CBMS dla Headplane. Dziala WYLACZNIE tutaj, w Control
 * Room - tunel istnieje jako lokalny port na maszynie operatora, nie na CBMS. Ten
 * link NIGDY nie trafia do frontend/public/world: telefon i inne urzadzenia w sieci
 * prywatnej nie maja tego tunelu, wiec adres byłby dla nich martwy.
 */
const CONTROL_ROOM_HEADPLANE_TUNNEL_URL = 'http://127.0.0.1:3001/admin'

export interface HeadplaneTunnelLink {
  href: string
  label: string
}

/**
 * Link przez tunel, TYLKO gdy sonda zmierzyla usluge jako `up`. Gdy tunel albo sama
 * usluga nie odpowiada, zwraca `null` - kafelek ma wtedy pokazac "zamkniety", NIGDY
 * podpowiadac adresu 192.168.2.1:3000 jako zastepstwa (takiego naslucha nie ma).
 */
export function headplaneTunnelLink(
  state: WorldServiceState | 'unknown',
): HeadplaneTunnelLink | null {
  if (state !== 'up') return null
  return { href: CONTROL_ROOM_HEADPLANE_TUNNEL_URL, label: 'Otworz przez tunel' }
}

/** Publiczna sonda stanu: jeden GET, bez PIN-u, cookies i ciala zapytania. */
export async function fetchWorldStatus(): Promise<WorldStatus> {
  const response = await fetch('/v1/world/status', {
    method: 'GET',
    headers: { accept: 'application/json' },
    credentials: 'omit',
    cache: 'no-store',
  })
  if (!response.ok) {
    throw new Error(`Blad odczytu stanu uslug (${response.status})`)
  }
  return (await response.json()) as WorldStatus
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

export interface GitCommit {
  hash: string
  parents: string[]
  refs: string[]
  subject: string
  author: string
  date: string
}

/** Odwzorowanie odpowiedzi GET /v1/git/overview z darkstar-server. */
export interface GitOverview {
  /** null przy detached HEAD - to normalny stan repozytorium, nie blad. */
  branch: string | null
  head: string
  dirty: boolean
  ahead: number
  behind: number
  hasUpstream: boolean
  upstream: string | null
  commits: GitCommit[]
}

/**
 * Odczyt stanu repozytorium z darkstar-server.
 *
 * `null` oznacza "ten serwer nie ma takiego widoku" - starszy obraz kontenera odpowiada 404.
 * Zwracamy wtedy pusty widok zamiast wywalac surowy blad na operatora. 401 i 503 to co innego:
 * endpoint istnieje i ma nam cos do powiedzenia, wiec leca dalej jako bledy.
 *
 * Naglowek budowany jest tutaj, a nie przez authorizationHeaders() - ta funkcja przepuszcza
 * wylacznie napis o dlugosci OPERATOR_PIN_LENGTH, a ten odczyt ma dotrzec do serwera takze
 * przy napisie niepelnym; odpowiedzia jest wtedy uczciwe 401 z Rusta.
 *
 * Pusty napis oznacza BRAK naglowka, a nie naglowek pusty. To rozroznienie jest istotne:
 * serwer wpuszcza petle zwrotna tylko wtedy, gdy zapytanie wlasnego naglowka nie przynioslo.
 * "Bearer " bez wartosci bylby naglowkiem bledym i skonczylby sie 401.
 */
export async function fetchGitOverview(pin: string): Promise<GitOverview | null> {
  const response = await fetch('/v1/git/overview', {
    method: 'GET',
    headers: pin ? { authorization: `Bearer ${pin}` } : {},
  })
  if (response.status === 404) {
    return null
  }
  if (response.status === 401) {
    throw new Error('Brak autoryzacji - podaj PIN operatora.')
  }
  if (response.status === 503) {
    throw new Error('Git nieodczytywalny - repozytorium nie jest podmontowane.')
  }
  if (!response.ok) {
    throw new Error(`Blad ${response.status}`)
  }
  return (await response.json()) as GitOverview
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
