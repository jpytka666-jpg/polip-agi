/*
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Opus 5
TIMESTAMP: 2026-09-01 23:20:00
REASON FOR CREATION: Warstwa dostepu Control Room do API Darkstara - wylacznie odczyt (Task 10).
MECHANICS: Dwa zapytania GET: /v1/system-graph i /v1/gateway/status. Typy odwzorowuja kontrakty
z darkstar-core (ArchitectureSnapshot, GatewayStatus). Nie ma tu ani jednej funkcji zapisujacej -
sterowanie brama nie istnieje w API i nie wolno go tu udawac. Token czytany z pamieci przegladarki,
nigdy z kodu.
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

/** Token operatora zyje w pamieci przegladarki, nigdy w repozytorium. */
export function readToken(): string {
  return localStorage.getItem('darkstar_api_token') ?? ''
}

export function storeToken(token: string): void {
  localStorage.setItem('darkstar_api_token', token)
}

async function getJson<T>(path: string, token: string): Promise<T> {
  const response = await fetch(path, {
    method: 'GET',
    headers: token ? { authorization: `Bearer ${token}` } : {},
  })
  if (response.status === 401) {
    throw new Error('Brak autoryzacji - podaj token operatora.')
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
