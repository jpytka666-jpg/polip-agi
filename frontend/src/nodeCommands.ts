/*
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Opus 5
TIMESTAMP: 2026-09-02 03:10:00
REASON FOR CREATION: Graf ma byc pulpitem operatora, a nie rysunkiem. Klikniecie wezla musi
albo wykonac prawdziwe wywolanie, albo powiedziec wprost BRAK KOMENDY. Ten plik to jedyne
miejsce, gdzie zapisane jest, ktory z 17 wezlow ma po drugiej stronie istniejacy endpoint.
MECHANICS: Mapa `id wezla -> komenda odczytu`. Wpisane sa WYLACZNIE trasy, ktore naprawde
istnieja w darkstar-server (sprawdzone w router(): http.rs, gateway_http.rs, context_http.rs).
Wezel bez wpisu nie dostaje przycisku - dostaje napis BRAK KOMENDY. Zadna komenda nie zmienia
stanu hosta: wszystkie sa metoda GET.
  ZWERYFIKOWANE TRASY ODCZYTU (router w darkstar-server):
    GET /health, GET /ready, GET /v1/system-graph,
    GET /v1/gateway/status, GET /v1/context/health, GET /v1/context/search
  SWIADOMIE POMINIETE:
    POST /v1/modules/{id}/actions - to start/stop/restart, czyli ZAPIS, nie odczyt.
    POST /v1/sessions, PUT .../memory/{key} - rowniez zapis.
    GET /v1/runs/{run_id}/events - wymaga run_id, ktorego graf nie zna.
  BRAK PO STRONIE SERWERA:
    Nie ma GET /v1/modules. ModuleRegistry (darkstar-core/src/module_registry.rs) trzyma
    `capabilities` kazdego modulu, ale NIE JEST wystawiony po HTTP. Dlatego panel pokazuje
    to, co faktycznie przychodzi w /v1/system-graph - role, system i jezyk - a przy polu
    capability stoi uczciwe "niewystawione po HTTP", a nie zmyslona lista.
SYSTEM PART: Control Room / warstwa akcji.
ARCHITECTURE FUNCTION: Granica miedzy rysunkiem a API. Kiedy serwer dostanie nowa trase
odczytu, dopisuje sie ja tutaj i wezel od razu ozywa.
DEPENDENCIES/LINKS: api.ts (runReadCommand), SystemGraph.tsx (panel operatora),
darkstar-server: http.rs, gateway_http.rs, context_http.rs.
TECH STACK: TypeScript 6, swiadomie zamiast Rusta - domyslnego jezyka tego projektu.
  (1) MUSI: byc odczytany przez komponent Reacta w chwili klikniecia i zlozyc adres, pod
      ktory pojdzie fetch z przegladarki.
  (2) DLACZEGO NIE RUST: to tablica adresow dla warstwy DOM. Rust przez WebAssembly musialby
      ja i tak wyeksportowac do JS, dokladajac granice FFI i osobny krok budowania, bez zadnej
      korzysci - nie ma tu obliczen ani I/O. Prawdziwa decyzja, czy wolno cos wykonac, zapada
      po stronie serwera w Ruscie i to on jej pilnuje (authenticated() + capability gate).
  (3) TRACIMY: wspolna kontrole typow z routerem. Rekompensata: kazdy wpis ma obok nazwe pliku
      serwera, w ktorym trase zadeklarowano, wiec rozjazd widac przy czytaniu.
LOCAL WORKSPACE: D:\codex-fresh-2026-08-28\worktrees\polip-agi-darkstar-plan
GIT COMMIT: PENDING
GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
==========================================
*/

export type NodeCommand = {
  /** Adres, pod ktory pojdzie zapytanie. Zawsze odczyt. */
  path: string
  /** Co operator dostanie - jednym zdaniem, bez zargonu. */
  what: string
  /** Plik serwera, w ktorym ta trasa jest zadeklarowana. Do sprawdzenia przy rozjezdzie. */
  declaredIn: string
}

/**
 * Wezly, ktore maja po drugiej stronie zywy endpoint odczytu.
 * Wszystko poza ta mapa dostaje w panelu napis BRAK KOMENDY.
 */
export const NODE_COMMANDS: Record<string, NodeCommand> = {
  'repo:polip': {
    path: '/v1/system-graph',
    what: 'caly spis elementow i polaczen, prosto z serwera',
    declaredIn: 'darkstar-server/src/http.rs',
  },
  'file:http': {
    path: '/health',
    what: 'czy serwer odpowiada i w ktorej jest wersji',
    declaredIn: 'darkstar-server/src/http.rs',
  },
  'runtime:session': {
    path: '/ready',
    what: 'czy serwer jest gotowy i czy ma ustawiony token',
    declaredIn: 'darkstar-server/src/http.rs',
  },
  'runtime:ghost-gate': {
    path: '/v1/gateway/status',
    what: 'stan bramy sieciowej - tylko odczyt, nic sie nie przestawia',
    declaredIn: 'darkstar-server/src/gateway_http.rs',
  },
  'runtime:aions-server-wiedzy': {
    path: '/v1/context/health',
    what: 'czy pamiec odpowiada - osobno noga lokalna i zdalna',
    declaredIn: 'darkstar-server/src/context_http.rs',
  },
}

export function commandFor(nodeId: string): NodeCommand | null {
  return NODE_COMMANDS[nodeId] ?? null
}
