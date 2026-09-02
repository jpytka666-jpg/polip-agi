/*
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Opus 5
TIMESTAMP: 2026-09-02 10:30:00
REASON FOR CREATION: Krawedz grafu jako trasa transportowa - tor z pociagiem albo trasa lotnicza z samolotem.
MECHANICS: Trzy rodzaje trasy. TOR (polaczenie w jednym procesie): podklady, dwie szyny, po nich
jedzie sklad - lokomotywa i wagony. TRASA LOTNICZA (polaczenie przez siec, czyli dotykajace
runtime): przerywana linia korytarza powietrznego, po niej leci samolot. TRASA MORSKA (wyjscie poza system, np. przez Ghost Gate): szeroka linia
z okretem podwodnym; wskaznik przy zwrotnicy pokazuje ZANURZENIE, czyli odleglosc wezla od
korzenia grafu - miara struktury, nie glebokosci w sieci. Ruch ciagnie SVG
animateMotion, wiec jest renderem, a nie petla w JavaScripcie. Na srodku stoi zwrotnica -
dzwignia z tarcza; przestawienie zatrzymuje ruch i zmienia trase na czerwona.
UWAGA - ZAKRES, trzy rzeczy wprost:
  1. LICZBA WAGONOW NIE JEST POMIAREM RUCHU. API Darkstara zwraca strukture grafu, nie wolumen
     danych. Wagony licza sie z liczby polaczen wezla docelowego - to realna liczba z danych,
     a nie zmyslony przeplyw. Okno wagonu mowi to samo.
  2. TOR KONTRA SAMOLOT NIE POCHODZI Z POLA "WiFi/kabel" - takiego pola w danych nie ma.
     Rozroznienie robimy po rodzaju wezlow: krawedz dotykajaca runtime biegnie miedzy procesami,
     czyli przez siec (samolot); krawedz miedzy plikami zyje w jednym procesie (tor).
  3. ZWROTNICA I NOTATKA ZMIENIAJA WIDOK, NIE SYSTEM. API wystawia wylacznie GET, wiec z
     przegladarki nie da sie ani zatrzymac polaczenia, ani nic zapisac na hoscie. Notatka
     operatora zyje w pamieci przegladarki i nigdzie dalej nie idzie.
SYSTEM PART: Control Room / widok architektury.
ARCHITECTURE FUNCTION: Pozwala przesledzic jedna trase i opisac ja wlasnymi slowami, bez
dotykania czegokolwiek na hoscie.
DEPENDENCIES/LINKS: SystemGraph.tsx, @xyflow/react (BaseEdge, EdgeLabelRenderer), localStorage.
TECH STACK: TypeScript 6 + React 19 + React Flow 12, swiadomie zamiast Rusta - domyslnego jezyka.
  (1) MUSI: rysowac animowane SVG i obslugiwac mysz w warstwie DOM nad plotnem.
  (2) DLACZEGO NIE RUST: to render SVG i zdarzenia przegladarki; Rust przez WebAssembly wymagalby
      tej samej warstwy TS. Dane i decyzje zostaja po stronie serwera w Ruscie.
  (3) TRACIMY: typy wspolne z rdzeniem; kontrakt odbity w api.ts.
LOCAL WORKSPACE: D:\codex-fresh-2026-08-28\worktrees\polip-agi-darkstar-plan
GIT COMMIT: PENDING
GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
==========================================
*/

import { useEffect, useState, type ReactNode } from 'react'
import { createPortal } from 'react-dom'
import { BaseEdge, EdgeLabelRenderer, getSmoothStepPath, type EdgeProps } from '@xyflow/react'
import { MODE, odmien, travelSeconds, type TransportMode } from './transport'

export type FlowEdgeData = {
  label: string
  enabled: boolean
  lit: boolean
  dimmed: boolean
  cars: number
  /** 'rail' = w jednym procesie, 'air' = przez siec, 'sea' = wyjscie na zewnatrz. */
  mode: TransportMode
  /** Czy caly ten rodzaj ruchu jest wlaczony. Osobno od `enabled`, ktore dotyczy jednej trasy. */
  modeEnabled: boolean
  /** Zanurzenie: odleglosc wezla docelowego od korzenia grafu. Miara struktury, nie glebokosci sieci. */
  depth: number
  onToggle: (id: string) => void
}

const CAR_W = 15
const CAR_GAP = 3

function noteKey(edgeId: string, car: number) {
  return `darkstar_car_note:${edgeId}:${car}`
}

/** Okno szczegolow - ten sam ksztalt co okno wezla: srodek ekranu, staly rozmiar.
 *
 *  Tryb `hover` istnieje z jednego powodu: okno otwarte najechaniem myszy nie moze
 *  przykryc elementu, ktory je otworzyl. Przeslona lapiaca kursor wywoluje na tym
 *  elemencie `mouseleave`, okno znika, przeslona znika, kursor znow jest na elemencie
 *  i wszystko zaczyna sie od nowa - okno miga kilkadziesiat razy na sekunde.
 *  W tym trybie przeslona przepuszcza kursor, a kliniecie w tlo lapie nasluch dokumentu. */
function DetailWindow({
  title,
  onClose,
  children,
  hover = false,
  onPointerEnter,
  onPointerLeave,
}: {
  title: string
  onClose: () => void
  children: ReactNode
  hover?: boolean
  onPointerEnter?: () => void
  onPointerLeave?: () => void
}) {
  useEffect(() => {
    if (!hover) return
    // Zamykamy dopiero, gdy mysz sie RUSZY poza okno i poza swoja tabliczke.
    // Zdarzenie mouseleave tu nie dziala: okno staje na srodku ekranu i samo
    // przykrywa tabliczke, wiec przegladarka melduje wyjscie przy nieruchomym
    // kursorze - i okno miga.
    const outside = (event: MouseEvent) => {
      const target = event.target as HTMLElement | null
      if (!target?.closest('.gpop, .eswitch')) onClose()
    }
    document.addEventListener('mousemove', outside)
    document.addEventListener('mousedown', outside)
    return () => {
      document.removeEventListener('mousemove', outside)
      document.removeEventListener('mousedown', outside)
    }
  }, [hover, onClose])

  return createPortal(
    <>
      <div
        className={`gpop__backdrop${hover ? ' gpop__backdrop--ghost' : ''}`}
        onClick={hover ? undefined : onClose}
      />
      <div
        className="gpop"
        role="dialog"
        aria-label={title}
        onMouseEnter={onPointerEnter}
        onMouseLeave={onPointerLeave}
      >
        <div className="gpop__body">
          <span className="gpop__title">{title}</span>
          {children}
        </div>
      </div>
    </>,
    document.body,
  )
}

function CargoWindow({
  edgeId,
  car,
  label,
  mode,
  onClose,
}: {
  edgeId: string
  car: number
  label: string
  mode: 'rail' | 'air' | 'sea'
  onClose: () => void
}) {
  // Notatke czytamy raz, przy otwarciu okna. Okno dostaje `key` na numer wagonu,
  // wiec przy przejsciu na inny wagon React montuje je od nowa i odczyt sie powtarza.
  const [note, setNote] = useState(() => {
    try {
      return localStorage.getItem(noteKey(edgeId, car)) ?? ''
    } catch {
      return ''
    }
  })

  const save = (value: string) => {
    setNote(value)
    try {
      localStorage.setItem(noteKey(edgeId, car), value)
    } catch {
      /* prywatne okno albo zablokowane dane witryny - notatka zostaje tylko na ekranie */
    }
  }

  const unit = mode === 'sea' ? 'Przedzial' : mode === 'air' ? 'Ladownia' : 'Wagon'

  return (
    <DetailWindow title={`${unit} ${car + 1} — ${label}`} onClose={onClose}>
      <dl className="gpop__facts">
        <dt>co laczy</dt>
        <dd>{label}</dd>
        <dt>czym sie jedzie</dt>
        <dd>
          {mode === 'sea'
            ? 'lodzia podwodna — wyjscie na zewnatrz, poza nasz system'
            : mode === 'air'
              ? 'samolotem — laczy dwa osobne programy przez siec'
              : 'pociagiem — obie strony siedza w tym samym programie'}
        </dd>
        <dt>ktory z kolei</dt>
        <dd>{car + 1}</dd>
        <dt>nazwa w kodzie</dt>
        <dd>{edgeId}</dd>
      </dl>

      <label className="gpop__field">
        <span>Twoja notatka</span>
        <textarea
          value={note}
          onChange={(e) => save(e.target.value)}
          placeholder="Wpisz wlasnymi slowami, co tu jedzie."
          rows={5}
        />
      </label>

      <p className="gpop__note">
        Wagonow jest tyle, ile rzeczy podpina sie pod koniec tej trasy. To ksztalt systemu,
        a NIE zmierzony ruch — nikt tu niczego nie wazyl. Notatka zostaje w tej przegladarce:
        nie idzie na serwer ani do repozytorium.
      </p>
    </DetailWindow>
  )
}

export function FlowEdge({
  id,
  sourceX,
  sourceY,
  targetX,
  targetY,
  sourcePosition,
  targetPosition,
  data,
}: EdgeProps) {
  const d = data as unknown as FlowEdgeData
  const [openCar, setOpenCar] = useState<number | null>(null)
  const [openSwitch, setOpenSwitch] = useState(false)


  const air = d.mode === 'air'
  const sea = d.mode === 'sea'

  // Kazdy rodzaj transportu dostaje wlasny pas: trasa jest przesunieta w pionie o stala
  // z MODE[...].lane. Dzieki temu pociagi, samoloty i lodzie biegna rownolegle zamiast
  // jedno po drugim - i mozna zgasic jeden rodzaj, nie ruszajac pozostalych.
  const lane = MODE[d.mode].lane
  const [path, labelX, labelY] = getSmoothStepPath({
    sourceX,
    sourceY: sourceY + lane,
    sourcePosition,
    targetX,
    targetY: targetY + lane,
    targetPosition,
    borderRadius: sea ? 30 : air ? 24 : 10,
    offset: 26,
  })

  // Zmierzone na jasnym plotnie #dcdcd8: zielen #2b8a3e 3.2:1, blekit #0e639c 4.7:1,
  // glebia #0e5a7d 5.6:1, krew #6b0f1a 8.9:1. To elementy graficzne, prog 3:1 spelniony.
  const live = sea
    ? d.lit
      ? '#0a3d62'
      : '#0e5a7d'
    : air
      ? d.lit
        ? '#0b4f7d'
        : '#0e639c'
      : d.lit
        ? '#1f7a32'
        : '#2b8a3e'
  const rail = d.enabled ? live : '#6b0f1a'
  const cars = Math.max(1, Math.min(d.cars, 6))
  // Czas przejazdu bierze sie z rodzaju trasy i dlugosci skladu, a nie z tego, czy
  // trasa jest podswietlona. Podswietlenie tylko przyspiesza o jedna czwarta, zeby
  // wybrana sciezka rzucala sie w oczy.
  const seconds = travelSeconds(d.mode, cars) * (d.lit ? 0.75 : 1)
  const dur = `${seconds.toFixed(2)}s`
  // Ruch stoi, gdy zamknieta jest ta trasa albo caly jej rodzaj.
  const moving = d.enabled && d.modeEnabled
  const faded = d.dimmed && !d.lit

  return (
    <>
      {/* TOR: podklady pod szynami. TRASA LOTNICZA: bez podkladow, sam korytarz. */}
      {d.mode === 'rail' ? (
        <BaseEdge
          id={`${id}-ties`}
          path={path}
          style={{
            stroke: d.enabled ? '#8a7355' : '#5a4038',
            strokeWidth: 14,
            strokeDasharray: '3 9',
            opacity: faded ? 0.2 : 0.85,
          }}
        />
      ) : null}

      <BaseEdge
        id={id}
        path={path}
        style={{
          stroke: rail,
          strokeWidth: sea ? 10 : air ? 4 : 8,
          strokeLinecap: 'round',
          opacity: faded ? 0.22 : 1,
          // Korytarz powietrzny jest kreskowany zawsze; tor tylko gdy zamkniety.
          strokeDasharray: air ? '14 10' : sea ? '26 6' : d.enabled ? undefined : '12 9',
        }}
      />

      {d.mode === 'rail' ? (
        <BaseEdge
          id={`${id}-shine`}
          path={path}
          style={{
            stroke: d.enabled ? 'rgba(255,255,255,0.5)' : 'rgba(255,255,255,0.25)',
            strokeWidth: 2,
            opacity: faded ? 0.15 : 0.7,
          }}
        />
      ) : null}

      {moving ? (
        <g className="train">
          {sea ? (
            /* Okret podwodny: kadlub, kiosk i peryskop. */
            <g className="train__loco">
              <ellipse cx={0} cy={0} rx={13} ry={6} fill="#123b52" stroke="#8ecbff" strokeWidth="1.6" />
              <rect x={-3} y={-10} width={6} height={5} rx={1.5} fill="#123b52" stroke="#8ecbff" strokeWidth="1.2" />
              <line x1={0} y1={-10} x2={0} y2={-15} stroke="#8ecbff" strokeWidth="1.6" />
              <circle cx={5} cy={0} r={2} fill="#8ecbff" />
              <animateMotion dur={dur} repeatCount="indefinite" rotate="auto" path={path} />
            </g>
          ) : air ? (
            /* Samolot: kadlub ze skrzydlami, obraca sie zgodnie z kursem. */
            <g className="train__loco">
              <path
                d="M 12 0 L -2 5 L -6 5 L -3 0 L -6 -5 L -2 -5 Z"
                fill="#e8f4ff"
                stroke="#0b4f7d"
                strokeWidth="1.6"
              />
              <path d="M 1 0 L -7 -11 L -3 -11 L 5 0 Z" fill="#cfe6fb" stroke="#0b4f7d" strokeWidth="1.2" />
              <path d="M 1 0 L -7 11 L -3 11 L 5 0 Z" fill="#cfe6fb" stroke="#0b4f7d" strokeWidth="1.2" />
              <animateMotion dur={dur} repeatCount="indefinite" rotate="auto" path={path} />
            </g>
          ) : (
            <g className="train__loco">
              <rect x={-11} y={-8} width={22} height={16} rx={4} fill="#ff8c1a" stroke="#7a3b00" strokeWidth="2" />
              <path d="M 2 -4 L 9 0 L 2 4 Z" fill="#111111" />
              <animateMotion dur={dur} repeatCount="indefinite" rotate="auto" path={path} />
            </g>
          )}

          {Array.from({ length: cars }).map((_, i) => (
            <g
              key={i}
              className="train__car"
              onClick={(e) => {
                e.stopPropagation()
                setOpenCar(i)
              }}
            >
              <rect
                x={-CAR_W / 2}
                y={sea ? -4.5 : air ? -5 : -6.5}
                width={CAR_W}
                height={sea ? 9 : air ? 10 : 13}
                rx={sea ? 4.5 : 3}
                fill={sea ? '#8ecbff' : air ? '#cfe6fb' : '#ffb45c'}
                stroke={sea ? '#0a3d62' : air ? '#0b4f7d' : '#7a3b00'}
                strokeWidth="1.6"
              />
              <text x={0} y={sea ? 2.6 : air ? 2.8 : 3.4} textAnchor="middle" fontSize="8" fontWeight="700" fill="#111111">
                {i + 1}
              </text>
              <animateMotion
                dur={dur}
                begin={`${-((i + 1) * (CAR_W + CAR_GAP)) / 60}s`}
                repeatCount="indefinite"
                rotate="auto"
                path={path}
              />
            </g>
          ))}
        </g>
      ) : null}

      <EdgeLabelRenderer>
        <div
          className={`eswitch${d.enabled ? ' eswitch--on' : ''}${d.lit ? ' eswitch--lit' : ''}${
            air ? ' eswitch--air' : sea ? ' eswitch--sea' : ''
          }`}
          style={{ transform: `translate(-50%, -50%) translate(${labelX}px, ${labelY}px)` }}
          onMouseEnter={() => setOpenSwitch(true)}
        >
          {/* Zwrotnica: tarcza wskaznika, dzwignia i koziol. */}
          <button
            type="button"
            className="eswitch__lever"
            role="switch"
            aria-checked={d.enabled}
            aria-label={`Zwrotnica ${d.label}`}
            onClick={(e) => {
              e.stopPropagation()
              d.onToggle(id)
            }}
          >
            <span className="eswitch__disc" />
            <span className="eswitch__arm" />
            <span className="eswitch__base" />
          </button>
          <span className="eswitch__label">
            {sea ? '⌁' : air ? '✈' : '▤'} {d.label}
          </span>
          {d.enabled ? <span className="eswitch__cars">{cars}</span> : null}
          {sea ? <span className="eswitch__depth">-{d.depth}</span> : null}
        </div>
      </EdgeLabelRenderer>

      {openSwitch ? (
        <DetailWindow
          title={`Zwrotnica — ${d.label}`}
          hover
          onClose={() => setOpenSwitch(false)}
        >
          <dl className="gpop__facts">
            <dt>teraz</dt>
            <dd>{d.enabled ? 'otwarta — ruch idzie' : 'zamknieta — ruch stoi'}</dd>
            <dt>gdy otwarta</dt>
            <dd>
              widac trase, a po niej jedzie {cars} {odmien(cars, MODE[d.mode].unit)}
            </dd>
            <dt>jak szybko</dt>
            <dd>
              {MODE[d.mode].speedWhy}; pelny przejazd trwa {dur}, dluzszy sklad jedzie wolniej
            </dd>
            <dt>wlasny pas</dt>
            <dd>
              {MODE[d.mode].title.toLowerCase()} jada osobnym pasem, wiec mozna zatrzymac je
              wszystkie i zostawic reszte ruchu
            </dd>
            <dt>gdy zamknieta</dt>
            <dd>trasa robi sie czerwona i przerywana, nic po niej nie jedzie</dd>
            <dt>czym sie jedzie</dt>
            <dd>
              {sea
                ? 'lodzia podwodna — to wyjscie na zewnatrz, poza nasz system'
                : air
                  ? 'samolotem — laczy dwa osobne programy przez siec'
                  : 'pociagiem — obie strony siedza w tym samym programie'}
            </dd>
            {sea ? (
              <>
                <dt>jak gleboko</dt>
                <dd>
                  {d.depth} {odmien(d.depth, ['przystanek', 'przystanki', 'przystankow'])}{' '}
                  od glownego katalogu
                </dd>
              </>
            ) : null}
            <dt>nazwa w kodzie</dt>
            <dd>{id}</dd>
          </dl>
          <p className="gpop__note">
            Ten przelacznik zmienia tylko rysunek na ekranie. Przez przegladarke mozna
            Darkstara wylacznie ogladac, wiec nic tu naprawde nie wylaczysz. Czerwona trasa
            znaczy „schowana na rysunku", a nie „cos padlo".
          </p>
        </DetailWindow>
      ) : null}

      {openCar !== null ? (
        <CargoWindow
          key={openCar}
          edgeId={id}
          car={openCar}
          label={d.label}
          mode={d.mode}
          onClose={() => setOpenCar(null)}
        />
      ) : null}
    </>
  )
}
