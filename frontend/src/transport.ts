/*
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Opus 5
TIMESTAMP: 2026-09-02 02:40:00
REASON FOR CREATION: Jedno miejsce, w ktorym opisane sa trzy srodki transportu na grafie -
pociag, samolot i lodz podwodna. Wczesniej pas, predkosc i nazewnictwo byly rozsypane po
FlowEdge.tsx, wiec kazda zmiana wymagala szukania po JSX.
MECHANICS: Dla kazdego srodka trzyma trzy rzeczy. (1) PAS - przesuniecie prostopadle w
pikselach, dzieki ktoremu trasy roznych rodzajow biegna rownolegle, a nie jedna po drugiej;
pas pozwala tez zgasic caly jeden rodzaj ruchu, bo kazdy ma wlasna wysokosc. (2) PREDKOSC -
czas przejazdu calej trasy. Kolejnosc jest odwrotna do potocznej intuicji i to jest celowe:
polaczenie w jednym programie (pociag) jest NAJSZYBSZE, przez siec (samolot) wolniejsze,
a wyjscie poza system (lodz) najwolniejsze - bo tak wyglada rzeczywisty koszt tych trzech
rodzajow polaczen. (3) SLOWNICTWO - nazwy jednostek w polskiej odmianie.
SYSTEM PART: Control Room / widok architektury.
ARCHITECTURE FUNCTION: Warstwa opisu metafory transportowej. Nie dotyka danych ani API -
zamienia rodzaj krawedzi na sposob jej narysowania i animowania.
DEPENDENCIES/LINKS: FlowEdge.tsx (rysowanie i animacja), SystemGraph.tsx (pasek blokad).
TECH STACK: TypeScript 6, swiadomie zamiast Rusta - domyslnego jezyka tego projektu.
  (1) MUSI: dostarczyc wartosci, ktore konsumuje kod rysujacy SVG w przegladarce - liczby
      trafiaja wprost do atrybutow animateMotion i do JSX, w tym samym module co reszta widoku.
  (2) DLACZEGO NIE RUST: to jest tablica stalych dla warstwy DOM. Rust przez WebAssembly
      musialby te wartosci i tak wyeksportowac do JS, doklada granice FFI i osobny krok
      budowania, a nie daje w zamian nic - nie ma tu obliczen, wspolbieznosci ani I/O.
      Rust pozostaje jezykiem control plane, gdzie zapadaja decyzje.
  (3) TRACIMY: wspolne typy z rdzeniem w Ruscie. Rekompensata: plik nie opisuje zadnego
      kontraktu serwera, tylko sposob rysowania, wiec nie ma czego rozjechac.
LOCAL WORKSPACE: D:\codex-fresh-2026-08-28\worktrees\polip-agi-darkstar-plan
GIT COMMIT: PENDING
GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
==========================================
*/

export type TransportMode = 'rail' | 'air' | 'sea'

export const MODES: TransportMode[] = ['rail', 'air', 'sea']

type ModeSpec = {
  /** Nazwa dla czlowieka, uzywana w pasku blokad. */
  title: string
  /** Znak wiodacy - ten sam co na tabliczce zwrotnicy. */
  glyph: string
  /** Przesuniecie pasa w pikselach. Kazdy rodzaj ma wlasna wysokosc, wiec trasy sie nie nakladaja. */
  lane: number
  /** Czas przejazdu calej trasy przy jednej jednostce ladunku, w sekundach. */
  baseSeconds: number
  /** Odmiana nazwy jednostki: 1 / 2-4 / 5+. */
  unit: [string, string, string]
  /** Jednym zdaniem: co ten rodzaj polaczenia znaczy naprawde. */
  meaning: string
  /** Jednym zdaniem: dlaczego jedzie akurat z taka predkoscia. */
  speedWhy: string
}

export const MODE: Record<TransportMode, ModeSpec> = {
  rail: {
    title: 'Pociagi',
    glyph: '\u25a4',
    lane: 0,
    baseSeconds: 3,
    unit: ['wagon', 'wagony', 'wagonow'],
    meaning: 'obie strony siedza w tym samym programie',
    speedWhy: 'najszybciej - dane nie opuszczaja jednego programu',
  },
  air: {
    title: 'Samoloty',
    glyph: '\u2708',
    lane: -14,
    baseSeconds: 6,
    unit: ['samolot', 'samoloty', 'samolotow'],
    meaning: 'laczy dwa osobne programy przez siec',
    speedWhy: 'wolniej niz pociag - po drodze jest siec',
  },
  sea: {
    title: 'Lodzie',
    glyph: '\u2301',
    lane: 14,
    baseSeconds: 10,
    unit: ['lodz', 'lodzie', 'lodzi'],
    meaning: 'wyjscie na zewnatrz, poza nasz system',
    speedWhy: 'najwolniej - to droga poza nasz system',
  },
}

/** Polska odmiana po liczbie: 1 wagon, 2 wagony, 5 wagonow. */
export function odmien(n: number, forms: [string, string, string]) {
  const last = n % 10
  const two = n % 100
  if (n === 1) return forms[0]
  if (last >= 2 && last <= 4 && (two < 12 || two > 14)) return forms[1]
  return forms[2]
}

/**
 * Czas przejazdu calej trasy. Dluzszy sklad jedzie wolniej - to zalozenie metafory,
 * a NIE zmierzony czas przesylu. Liczba jednostek pochodzi z ksztaltu grafu.
 */
export function travelSeconds(mode: TransportMode, cars: number) {
  return MODE[mode].baseSeconds * (1 + (Math.max(cars, 1) - 1) * 0.1)
}
