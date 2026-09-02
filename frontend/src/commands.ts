/*
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Opus 5
TIMESTAMP: 2026-09-02 02:45:00
REASON FOR CREATION: Przygotowanie diagramu na to, zeby jego funkcje dalo sie faktycznie
wykonac ("execute ready"), bez udawania, ze juz teraz da sie cokolwiek zmienic na hoscie.
Dzis Darkstar wystawia wylacznie odczyt. Zeby pozniejsze podpiecie zapisu bylo zmiana w
JEDNYM miejscu, a nie w kilkunastu miejscach w JSX, kazda akcja diagramu przechodzi tedy.
MECHANICS: Akcja to opisany obiekt, nie wywolanie funkcji rozsiane po komponentach. Kazda
ma zadeklarowany zasieg: 'view' zmienia wylacznie rysunek w przegladarce i wykonuje sie od
razu, 'host' zmienialby stan maszyny i jest ODRZUCANA, dopoki nie istnieje API zapisu.
Odrzucenie jest jawne - wraca powod, ktory mozna pokazac operatorowi, zamiast cichego nic.
Dzieki temu przycisk moze juz dzis istniec, byc opisany i przetestowany, a mimo to nie
klamac, ze cos zrobil.
SYSTEM PART: Control Room / warstwa akcji.
ARCHITECTURE FUNCTION: Jedyna brama miedzy interfejsem a swiatem. Kiedy control plane w
Ruscie dostanie sciezke zapisu, podpina sie ja w funkcji `execute` i nic poza tym plikiem
nie musi sie zmienic.
DEPENDENCIES/LINKS: SystemGraph.tsx i FlowEdge.tsx (wysylaja akcje), api.ts (przyszly zapis).
TECH STACK: TypeScript 6, swiadomie zamiast Rusta - domyslnego jezyka tego projektu.
  (1) MUSI: zyc w tym samym procesie co komponenty React i wywolywac ich funkcje stanu
      synchronicznie, w reakcji na klikniecie w przegladarce.
  (2) DLACZEGO NIE RUST: to dyspozytor zdarzen interfejsu, nie logika decyzyjna. Rust przez
      WebAssembly nie moze wywolac setState Reacta bez warstwy TS po drodze, wiec dolozylby
      granice zamiast ja usunac. Prawdziwa decyzja - czy wolno cos zmienic na hoscie - i tak
      zapadnie po stronie serwera w Ruscie, i to on bedzie ja egzekwowal.
  (3) TRACIMY: kontrola typow akcji wspolna z serwerem. Rekompensata: zasieg 'host' jest tu
      twardo odrzucany, wiec przegladarka nie ma czym skusic serwera.
LOCAL WORKSPACE: D:\codex-fresh-2026-08-28\worktrees\polip-agi-darkstar-plan
GIT COMMIT: PENDING
GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
==========================================
*/

import type { TransportMode } from './transport'

/** Co akcja rusza: sam rysunek czy stan maszyny. */
export type Scope = 'view' | 'host'

export type Command =
  | { kind: 'toggle-edge'; scope: 'view'; edgeId: string }
  | { kind: 'toggle-mode'; scope: 'view'; mode: TransportMode }
  | { kind: 'select-node'; scope: 'view'; nodeId: string | null }
  /** Zarezerwowane. Nie ma dzis sciezki zapisu - `execute` to odrzuca. */
  | { kind: 'restart-runtime'; scope: 'host'; nodeId: string }
  | { kind: 'close-egress'; scope: 'host'; edgeId: string }

export type CommandResult =
  | { ok: true; scope: Scope }
  | { ok: false; reason: string }

/** Co akcja robi - jednym zdaniem, dla operatora. */
export function describe(cmd: Command): string {
  switch (cmd.kind) {
    case 'toggle-edge':
      return 'chowa albo pokazuje te trase na rysunku'
    case 'toggle-mode':
      return 'chowa albo pokazuje caly jeden rodzaj ruchu na rysunku'
    case 'select-node':
      return 'podswietla droge przez wybrany element'
    case 'restart-runtime':
      return 'restart programu na maszynie'
    case 'close-egress':
      return 'zamkniecie wyjscia na zewnatrz'
  }
}

/** Podpiete przez widok. Tylko akcje o zasiegu 'view' maja tu prawo wstepu. */
export type ViewHandlers = {
  toggleEdge: (edgeId: string) => void
  toggleMode: (mode: TransportMode) => void
  selectNode: (nodeId: string | null) => void
}

/**
 * Jedyne wyjscie akcji z interfejsu.
 *
 * Akcje widoku wykonuja sie od razu. Akcje dotykajace maszyny sa odrzucane z powodem,
 * bo API Darkstara wystawia wylacznie odczyt - przegladarka nie ma czym ich wyslac.
 * Kiedy sciezka zapisu powstanie, dopisuje sie ja TUTAJ i nigdzie indziej.
 */
export function execute(cmd: Command, view: ViewHandlers): CommandResult {
  if (cmd.scope === 'host') {
    return {
      ok: false,
      reason: `Nie wykonano: ${describe(cmd)}. Przez przegladarke Darkstara mozna tylko ogladac - nie ma sciezki zapisu.`,
    }
  }

  switch (cmd.kind) {
    case 'toggle-edge':
      view.toggleEdge(cmd.edgeId)
      return { ok: true, scope: 'view' }
    case 'toggle-mode':
      view.toggleMode(cmd.mode)
      return { ok: true, scope: 'view' }
    case 'select-node':
      view.selectNode(cmd.nodeId)
      return { ok: true, scope: 'view' }
  }
}
