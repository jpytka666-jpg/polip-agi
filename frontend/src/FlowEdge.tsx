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
import { MODE, odmien, type TransportMode } from './transport'

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
  // Ile rzeczy podpina sie pod koniec tej trasy. Miara ksztaltu grafu, nie ruchu.
  const cars = Math.max(1, Math.min(d.cars, 6))
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
            <dt>gdy widoczna</dt>
            <dd>linia jest narysowana pelnym kolorem</dd>
            <dt>gdy ukryta</dt>
            <dd>linia robi sie czerwona i przerywana - znika z rysunku, nie z maszyny</dd>
            <dt>ile sie podpina</dt>
            <dd>
              {cars} {odmien(cars, MODE[d.mode].unit)} po stronie celu
            </dd>
            <dt>wlasna warstwa</dt>
            <dd>
              {MODE[d.mode].title.toLowerCase()} maja osobna warstwe, wiec mozna ukryc je
              wszystkie i zostawic reszte
            </dd>
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
            To filtr widoku. Chowa linie na rysunku i nic wiecej - zadne polaczenie na
            maszynie sie nie zmienia. Zeby cos naprawde odczytac, kliknij wezel i uzyj
            przycisku Wykonaj w panelu po prawej.
          </p>
        </DetailWindow>
      ) : null}

    </>
  )
}
