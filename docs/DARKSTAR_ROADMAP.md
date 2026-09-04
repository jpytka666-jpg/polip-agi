<!-- darkstar-header-v1 -->
<!-- po co: DARKSTAR_ROADMAP.md -->
<!-- nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie -->
<!-- autor: Marcin -->
<!-- powstal: 2026-09-04 -->
<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: GPT-5 Codex
TIMESTAMP: 2026-08-29 11:19:11 Europe/London
REASON FOR CREATION: Utworzenie jednego kanonicznego harmonogramu rozwoju Darkstar od checkpointu fd8099e przez Ghost Gate, Headscale/Headplane, Warlock/Kali i przyszłą infrastrukturę AIONS.
REVISION: 2026-09-03 17:49 Europe/London — sekcja stanu przepisana na pomiar żywego hosta (Claude Opus 5).
REVISION: 2026-09-04 12:43 Europe/London — dopisano zmierzony stan dual bind i drogi telefonu do /world/ (GPT-5 Codex).
REVISION: 2026-09-04 14:05 Europe/London — dopisano stan po uruchomieniu Headplane wyłącznie na pętli (GPT-5 Codex).
REVISION: 2026-09-04 14:40 Europe/London — dopisano żywe, wyłącznie odczytowe kafelki po wdrożeniu obrazu na CBMS (GPT-5 Codex).
REVISION: 2026-09-04 16:19 Europe/London — dopisano otwarcie zapory hosta dla portu 18080 z LAN i istnienie plików unitów systemd Headscale/Headplane (Claude Sonnet 5).
REVISION: 2026-09-04 17:33 Europe/London — Task 14 (14.1-14.11) zamknięte papierowo; 14.12 to bramka, nie osobne zadanie (Claude Sonnet 5).
==========================================
-->

# Darkstar Roadmap

## Źródło prawdy

- Repozytorium: jpytka666-jpg/polip-agi
- Gałąź rozwojowa (kanon od 2026-09-03): docs/darkstar-headscale-hotspot-plan
- Stary handoff: feat/darkstar-module-control — punkt przekazania, nie źródło prawdy dnia
- Punkt przekazania: fd8099e4e3c3399ae69a885fa2ed32ab0c57d2df
- Szczegółowa specyfikacja: docs/superpowers/specs/2026-08-29-darkstar-native-gateway-private-mesh-design.md
- Szczegółowy plan: docs/superpowers/plans/2026-08-29-darkstar-native-gateway-headscale-headplane-plan.md

## Legenda

- DONE — wykonane i potwierdzone testem.
- PARTIAL — część działa, ale brakuje pełnego dowodu.
- NEXT — najbliższy etap wykonawczy.
- PLANNED — zaplanowane po spełnieniu wcześniejszych bramek.
- BLOCKED — wymaga sprzętu, decyzji lub zewnętrznego warunku.
- ŻYWE — działa teraz i ma pomiar z żywego środowiska.
- NIE — nie działa, nie zostało uruchomione albo nie należy jeszcze do aktywnego zakresu.

## Stan na 2026-09-04 16:19 — ZMIERZONE

To jest najnowsza prawda dnia. Zapora hosta puszcza teraz LAN do portu 18080 — wcześniej
serwer miał już oba jawne bindy, ale reguła zapory wciąż odcinała `192.168.2.0/24` od
`18080`, mimo że `8080` (Headscale) przechodził. Operator ręcznie przeładował zaporę;
szczegółowy pomiar jest w `docs/operations/evidence/2026-09-04-lan-18080.md`, reguła to
commit `a8eb53b`.

| Obszar | Stan | Dowód zmierzony 2026-09-04 |
|---|---|---|
| Zapora hosta — port 18080 z LAN | ŻYWE | `darkstar-firewall-apply` zwrócił "loaded"; `grep -c 18080 /etc/darkstar/host-guard.nft` = 4, zgodnie z commitem `a8eb53b`; `/world/` odpowiada z kafelkami UP bez tunelu. |
| Jednostki systemd Headscale/Headplane | PARTIAL — pliki istnieją, nie zainstalowane | `deploy/systemd/darkstar-headscale.service` i `darkstar-headplane.service` są w repozytorium, ze ścieżką roboczą poprawioną na `/home/owner/polip-agi`; żaden nie jest `enable`d ani wystartowany — oba serwisy działają dziś bez systemd. |
| Headplane — Step 14.3, 14.4, 14.5 | LIVE | `ss` wyłącznie `127.0.0.1:3000`, `/admin` przez pętlę = 302, LAN `192.168.2.0/24` = 000, sekrety w `/etc/darkstar/headplane/*`, `docker ps` `healthy`. Pętla zwrotna jest docelowym stanem, nie tymczasowym. Dowody: `docs/operations/evidence/2026-09-04-headplane-private-listen.md`, `docs/operations/evidence/2026-09-04-network-done.md`. |
| Headplane — Step 14.6 | UNIT PLIK ONLY | `deploy/systemd/darkstar-headplane.service` istnieje w repozytorium, ścieżka robocza poprawiona na `/home/owner/polip-agi`. Nikt go nie `enable`ował ani nie startował na hoście — to zostaje ręczną czynnością operatora. |
| Headplane — Step 14.7–14.10 | KOD JEST | 14.7 węzeł `runtime:headplane` w grafie (`4019b4d`). 14.8 pasek statusu w Control Room (istniał już wcześniej). 14.9 link przez tunel `127.0.0.1:3001`, tylko Control Room, nigdy `frontend/public/world` (`a794fff`). 14.10 testy granic: sonda przypięta do `127.0.0.1:3000` w `main.rs`, `frontend/public/world` bez tunelu/LAN/`0.0.0.0` (`9b767aa`). |
| Headplane — Step 14.11 | DONE, opis się mylił | `deploy/headplane/verify-config` (`1789eec`) mierzy naprawdę: `docker compose config` kończy się kodem 0 niezależnie od obecności plików sekretów — to polecenie sprawdza wyłącznie składnię YAML, nie istnienie ścieżek na dysku. Poprzedni opis w README zakładał odwrotnie; poprawiony tym samym zapisem co ten wiersz. |

Task 14, kroki 14.1–14.11 (ops/infrastruktura i kod granic uprawnień w `crates/`/`frontend/`):
zamknięte, każdy z commitem i dowodem powyżej. Krok 14.12 jest bramką papierową, nie osobnym
zadaniem: instalacja unitu na hoście (14.6) i szerszy zakres UI (osobny widok `MeshPanel`,
dodatkowa nawigacja) zostają świadomie otwarte jako następny, inny kawałek pracy.

## Historia — stan na 2026-09-04 14:40 — ZMIERZONE

To był wcześniejszy pomiar dnia. Kafelki są podłączone do żywych sond, ale niczego nie
uruchamiają ani nie zatrzymują. Szczegółowy pomiar jest w
`docs/operations/evidence/2026-09-04-world-tiles-live.md`; poprawka self-probe to `3ad0df7`.

| Obszar | Stan | Dowód zmierzony 2026-09-04 |
|---|---|---|
| Darkstar | ŻYWE | Kontener z obrazem `f0b2473c580f` jest `healthy`; health odpowiada HTTP 200, a `ss` pokazuje wyłącznie dwa jawne bindy `127.0.0.1:18080` i `192.168.2.1:18080`. |
| `/world/` | ŻYWE | Landing i `world.js` odpowiadają HTTP 200; HTML rzeczywiście ładuje ten skrypt. |
| Kafelki live | ŻYWE — TYLKO ODCZYT | `world.js` wykonuje GET `/v1/world/status`; odpowiedź ma `readOnly: true` oraz `up` dla Darkstar, Headscale i Headplane. |
| Headscale | ŻYWE | `http://192.168.2.1:8080/windows` odpowiada HTTP 200; kontener nie został odtworzony w tej fali. |
| Headplane | ŻYWE — TYLKO PĘTLA | Listen pozostaje wyłącznie `127.0.0.1:3000`, `/admin` odpowiada 302, a kontener nie został odtworzony. |
| Tailscale SaaS | ŻYWE OBOK — NIE ODCIĘTE | `tailscaled.service` pozostaje `active`; M8 nie zostało wykonane. |
| Warlock | NIE — ZAKAZANY W TEJ FAZIE | Nie uruchomiono, nie skonfigurowano i nie włączono żadnego hosta ani komponentu Warlock/Kali. |

## Historia — stan na 2026-09-04 14:05 — ZMIERZONE

To był wcześniejszy pomiar dnia. Zastąpił jeszcze starsze wiersze dotyczące Headplane i wejścia
`/world/`, ale nie kasuje pomiaru sprzed uruchomienia panelu. Dowód runtime jest zapisany w
`docs/operations/evidence/2026-09-04-network-done.md`, commit `c04a161`.

| Obszar | Stan | Dowód zmierzony 2026-09-04 |
|---|---|---|
| Headplane | ŻYWE — TYLKO PĘTLA | Kontener jest `healthy` i słucha wyłącznie na `127.0.0.1:3000`; Windows używa tunelu `127.0.0.1:3001 -> 127.0.0.1:3000`, a `/admin` odpowiada HTTP 302. Panel nie jest osadzony ani wystawiony przez `/world/`. |
| `/world/` | ŻYWE | `http://192.168.2.1:18080/world/` odpowiada HTTP 200 z telefonu przez prywatną drogę. |
| Kafelki live | NIE | Powłoka działa, ale kafelki nadal nie odczytują żywych stanów usług. |
| Tailscale SaaS | ŻYWE OBOK — NIE ODCIĘTE | Komercyjny control plane pozostaje fallbackiem obok własnego Headscale; M8 nie zostało wykonane. |
| Warlock | NIE — ZAKAZANY W TEJ FAZIE | Nie uruchomiono, nie skonfigurowano i nie włączono żadnego hosta ani komponentu Warlock/Kali. |

## Historia — stan na 2026-09-04 przed Headplane — ZMIERZONE

To był wcześniejszy pomiar dnia 2026-09-04. Sekcja rozdzielała działające elementy od plików
i planów, które jeszcze niczego nie uruchamiały. Dowód dual bind i pomiar ośmiu bramek jest
w `docs/operations/evidence/2026-09-04-network-done.md`, commit `b318433`.

| Obszar | Stan | Dowód zmierzony 2026-09-04 |
|---|---|---|
| Bramka `DARKSTAR-WiFi` | ŻYWE | CBMS ma `192.168.2.1`; Windows przechodzi przez hub do bramki Darkstar. |
| Headscale | ŻYWE | Headscale v0.29.3 odpowiada na `192.168.2.1:8080`; telefon z One Agent otwiera `/windows` z HTTP 200. |
| Named tunnel `darkstar` | ŻYWE | Stan `Healthy`, trasa CIDR `192.168.2.0/24`, zespół Zero Trust `weathered-bar-3262`. |
| Darkstar dual bind | ŻYWE | Kod `26ba7e0` jest w obrazie uruchomionym na CBMS; `ss` pokazuje `127.0.0.1:18080` oraz `192.168.2.1:18080`. |
| `/world/` | ŻYWE | Telefon otrzymuje HTTP 200. To żywa powłoka strony; kafelki nie mają jeszcze żywych danych. |
| `verify-world` | ŻYWE | Commit dowodowy `b318433`; skrypt zwrócił 8/8 PASS, w tym LAN bez tokenu → 401 i brak wildcard bindu. |
| Procesy drogi do świata | ŻYWE | `cloudflared.service` jest `active`; kontener `darkstar` jest `running/healthy`. |
| Tailscale SaaS | ŻYWE OBOK | Nadal działa jako fallback obok własnego Headscale. To nie jest wykonanie M8 i niczego nie odłączono. |
| Headplane | NIE | Pliki są w `7d67562`, ale nie wykonano `compose up` i kontener nie działa. |
| Control Room ze świata | NIE | Dostęp pozostaje przez tunel SSH i pętlę `127.0.0.1`; `/world/` nie otwiera Control Room. |
| Kafelki live | NIE | Powłoka `/world/` działa, ale kafelki nie są podłączone do żywych źródeł. |
| Warlock | NIE | Host i publiczna warstwa Warlock nie istnieją; M9 pozostaje zablokowane. |
| Kontekst AIONS ze świata | NIE | Kontekst działa lokalnie, ale nie jest wystawiony przez drogę światową. |
| `nft` w tej fali | NIE — nietknięte | Nie odczytywano ani nie zmieniano reguł; dual bind, obraz i pomiary nie wymagały operacji `nft`. |

## Historia — stan na 2026-09-03 — ZMIERZONE

To była prawda dnia 2026-09-03. Każdy wiersz pochodził z komendy uruchomionej tego dnia
na żywej maszynie, nie z dokumentu. Tabela zostaje jako historia i nie jest już źródłem
prawdy bieżącej; w razie różnicy obowiązuje sekcja z 2026-09-04 powyżej.

| Obszar | Stan | Dowód zmierzony 2026-09-03 |
|---|---|---|
| Headscale | DONE | Kontener `darkstar-headscale`, obraz `headscale/headscale:v0.29.3`. `http://192.168.2.1:8080/` → HTTP 200. `headscale nodes list` → 2 węzły, 0 wygasłych. |
| Prywatny mesh + SSH | DONE | 100.64.0.1 (Windows) i 100.64.0.2 (CBMS) oba online, połączenie **direct** przez `192.168.2.1:41642`, `tailscale ping` 2–3 ms. `ssh owner@100.64.0.2 'echo MESH_SSH'` → `MESH_SSH`, rc=0. |
| Chroma 8000 + 8001 | DONE | `127.0.0.1:8000/api/v2/heartbeat` → 200, `127.0.0.1:8001/api/v2/heartbeat` → 200. `systemctl is-active tailscaled tailscaled-headscale darkstar-chroma-e-copy` → `active active active`. |
| Kontekst AIONS | DONE | `/v1/context/health` → `{"local_cbms_ok":true,"remote_e_ok":true}`. |
| Host-guard | PARTIAL | Zachowanie na żywo potwierdzone: `headscale0` przepuszcza wyłącznie tcp/22, a HTTP po mesh (`100.64.0.2:8080/8000/8001`) jest **dropowane** — timeout, nie odmowa. To dowód działania, nie dowód konfiguracji: reguł `nft` ani unitu firewalla nie oglądano, bo operator wprost zabronił dotykania `nft` w tej sesji. |
| Control Room — dostęp i trwałość | DONE (2026-09-03) | Tunel SSH po prywatnym mesh; właściciel gniazda 18080 to `ssh.exe` z `owner@100.64.0.2`; `/health` → 200 `{"service":"darkstar","api_version":"darkstar.core/v1","status":"ok"}`. Krok 6 w `aions_boot_steps.json` (`enabled=true`) woła `control_room_tunnel_run.cmd` → `Start-ControlRoomTunnel.ps1`; krok weryfikuje **właściciela gniazda**, nie sam kod 200, i jest celowo ostatni w rozruchu. |
| Control Room UI | PARTIAL | Bez zmian od 29.08. Starter Vite + React Flow istnieje; produkcyjny ekran nie jest skończony. Osadzony widok w Rust pozostaje fallbackiem. |
| Provider execute | NEXT | `DryRunProvider` jest jedyną implementacją `ModuleProvider`; `module_control_http.rs` nadal deklaruje „This v1 boundary does not execute infrastructure”. Żądania są autoryzowane, ale **nic nie wykonują**. To najbliższy etap wykonawczy. |
| cloudflared / wyjście na świat | PLANNED | Skrypt `deploy/headscale/world-tunnel` istnieje, `ORIGIN=http://192.168.2.1:8080`. Ale `cloudflared` **nie jest zainstalowany na CBMS**: `command -v` puste, unit `inactive`, brak procesu. Binarka istnieje wyłącznie po stronie Windows — po przeciwnej stronie względem źródła. |
| Warlock Bridge | PLANNED | Bez zmian. Wymaga migracji typów i przyszłego hosta. |

Poza tabelą, zmierzone i istotne:

- Komercyjny login server nadal działa obok własnego: `100.71.8.70`, `idle; offers exit node`.
  Węzeł Windows po tamtej stronie jest offline od 17 h. Odłączenie pozostaje w M8.
- Headplane nie jest uruchomiony — `docker ps` nie pokazuje takiego kontenera.
- Host: `up 2 days, 11 hours`, `boot_id dbe6ad68-eff2-4954-baa4-66c7077d1c8c`. W tej sesji
  **nie było restartu Ubuntu**, więc żaden wiersz DONE po stronie Ubuntu nie jest dowodem
  na przetrwanie reboota. Reguła 2 z sekcji „Reguły aktualizacji roadmapy” pozostaje dla
  nich niespełniona.

## Historia — stan na 2026-08-29 (nieaktualny)

Poniższa tabela jest zachowana jako zapis tego, co wiedziano 2026-08-29. **Nie jest
źródłem prawdy.** Tam, gdzie różni się od tabeli wyżej, obowiązuje tabela wyżej.


| Obszar | Stan | Dowód / brakujący dowód |
|---|---|---|
| Rust control plane | DONE | Core, policy, sessions, module contracts i serwer istnieją. |
| Control Room osadzony w Rust | PARTIAL | Graf i API istnieją; pełny realny provider i produkcyjny UI nie są skończone. |
| React/TypeScript frontend | PARTIAL | Commit fd8099e dodaje starter Vite i React Flow; ekran jest nadal starterem Vite. |
| Module control HTTP | PARTIAL | Żądania są autoryzowane, ale v1 nie wykonuje infrastruktury. |
| Testy Rust | PARTIAL | Formatowanie przechodzi; trzy testy integracyjne zwracają 401 z powodu fixture zależnego od env. |
| Darkstar systemd/Compose | DONE | darkstar.service jest aktywny na Ubuntu. |
| Dependency retry timer | DONE | Timer jest aktywny. |
| Host firewall | PARTIAL | Pliki istnieją na Ubuntu i unit jest enabled, ale inactive/dead; brak dowodu startu po reboot. |
| IPv4/IPv6 forwarding | DONE | Obie wartości runtime wynoszą 1; persistence ma być ponownie sprawdzone po reboot. |
| Tailscale exit node | TEMPORARY | Działa jako obecny kanał zarządzania i awaryjny fallback. |
| Serveo | DONE | Fallback wyłączony/inaktywny. |
| Natywny downstream do Windows | NEXT | enp1s0 istnieje; hotspot AP raportowany, lecz brak testu concurrent AP+STA. |
| Headscale | PLANNED | Nie jest zainstalowany. |
| Headplane | PLANNED | Wybrany jako UI najbliższy Tailscale; nie jest zainstalowany. |
| Warlock Bridge | PLANNED | Nowa nazwa kanoniczna zastępująca Warlock; wymaga migracji typów i przyszłego hosta. |
| Kali Bridge | BLOCKED | Czeka na przyszłą maszynę. |
| Domowa chmura Git/CI | PLANNED | Osobny etap po stabilnej sieci. |
| AIONS na PowerEdge | BLOCKED | Osobny etap po przygotowaniu i montażu sprzętu. |

## Decyzje technologiczne

### Rust 2024 + Axum/Tokio

Pozostaje właściwym wyborem dla zaufanego control plane, polityki, providerów, audytu i HTTP API. Zapewnia typowane granice i spójność z istniejącym repozytorium.

### React + TypeScript + Vite + React Flow

Jest właściwym wyborem dla złożonego interaktywnego Control Room. Checkpoint fd8099e wprowadził już ten stos. Nie wolno utrzymywać dwóch niezależnych pełnych UI; osadzony widok Rust jest fallbackiem do czasu osiągnięcia zgodności funkcjonalnej.

### NetworkManager + nftables + systemd

Są natywnymi warstwami Ubuntu odpowiedzialnymi odpowiednio za profile/interfejsy, reguły sieciowe oraz start/persistence. Darkstar steruje nimi przez wąski provider, zamiast tworzyć własny sterownik sieci.

### Headscale + Headplane

Headscale zapewnia samodzielnie hostowany control server sieci mesh. Headplane jest wybranym panelem webowym zbliżonym do Tailscale. Oba komponenty pozostają oddzielnymi, przypiętymi zależnościami open source z zachowanymi licencjami.

## Kolejność realizacji

### M0 — Ochrona checkpointu i dokumentacja

Stan: DONE

Dowód: commit b3b599f utworzył specyfikację, roadmapę i plan z zadaniami
0–16 jako potomka checkpointu fd8099e; kolejna kontrola dodała obowiązkową
bramkę 0A dla logowania SSH kluczem.

Zakres:

- zachowanie fd8099e jako dokładnego punktu startowego;
- praca w izolowanym worktree;
- zapis specyfikacji, roadmapy i planu;
- aktualizacja wcześniejszych dokumentów o rzeczywisty status;
- utrwalenie zmiany Warlock -> Warlock;
- commit i push dokumentacji.

Brama wyjścia:

- dokumenty są spójne;
- nowe pliki mają AUTHOR: M. SZUL;
- GitHub wskazuje commit dokumentacyjny jako potomka fd8099e.

### M1 — Zielona baza testowa

Stan: PLANNED

Zakres:

- naprawienie logowania SSH kluczem bez nadpisywania authorized_keys i bez
  wyłączania hasła przed udanym testem BatchMode;
- zastąpienie AppState::from_env w test fixture deterministycznym stanem z tokenem;
- dodanie testu potwierdzającego fail-closed bez tokena;
- uruchomienie fmt, clippy i pełnego workspace test;
- uruchomienie lint/build frontendu;
- usunięcie pozostałości build/cache po weryfikacji, jeśli zostały stworzone lokalnie.

Brama wyjścia:

- zero nieoczekiwanych test failures;
- brak sekretów i zależności testów od zmiennych procesu.

### M2 — Firewall jako kod i prawdziwy reboot

Stan: PLANNED, najwyższy priorytet infrastrukturalny

Zakres:

- odczyt i review aktualnych /etc/darkstar plików;
- przeniesienie audytowalnych szablonów do deploy/firewall i deploy/systemd;
- test składni nftables;
- kontrolowany manual start usługi;
- sprawdzenie aktywnej tablicy;
- prawdziwy reboot z porównaniem boot_id;
- sprawdzenie usług, API i routingu po boot.

Brama wyjścia:

- nowy boot_id;
- darkstar-firewall.service active;
- reguły nftables obecne;
- Darkstar API i kanał administracyjny dostępne;
- dowód zapisany w runbooku bez sekretów.

### M3 — Natywny link Windows -> Darkstar

Stan: PLANNED

Zakres:

- preflight Ethernet i Wi-Fi;
- instalacja iw tylko jeśli potrzebna do bezpiecznego sprawdzenia sprzętu;
- wybór Ethernet downstream jako wariantu bazowego;
- hotspot jako podmoduł i wariant mobilny/rodzinny;
- osobny profil NetworkManager;
- DHCP, DNS, NAT i forward;
- Windows cutover oraz rollback w natywnym PowerShell/netsh;
- test braku automatycznego przejścia Windows na domowe Wi-Fi.

Brama wyjścia:

- Windows ma jedną aktywną drogę domyślną przez Darkstar;
- publiczne IP Windows jest adresem egress Darkstar;
- utrata upstream Darkstar odcina Internet zamiast omijać Darkstar;
- konfiguracja wraca po reboot.

### M4 — Darkstar Network Provider

Stan: PLANNED

Zakres:

- typowane modele GatewayStatus i GatewayCommand;
- capability: network.inspect, network.start, network.stop, network.restart;
- policy gate i trusted approval;
- ograniczony provider systemd/NetworkManager;
- event i audit;
- chronione endpointy HTTP;
- testy unit, route i provider contract.

Brama wyjścia:

- przeglądarka nie ma dostępu do powłoki;
- provider nie wykonuje dowolnego argumentu;
- wszystkie mutacje mają audyt.

### M5 — Control Room React v1

Stan: PLANNED

Zakres:

- usunięcie ekranu startowego Vite;
- struktura features/api/components;
- autoryzowane API;
- System Graph i Live Run Graph;
- panel modułu, firewall i gateway;
- timeline SSE;
- stany loading/empty/error/blocked;
- testy React;
- build frontendu serwowany przez Rust;
- zachowanie osadzonego fallbacku do odbioru.

Brama wyjścia:

- pełny podstawowy zakres wcześniejszego Control Room;
- brak tokenów w URL i localStorage;
- lint, typecheck, test i build przechodzą.

### M6 — Prywatny Headscale

Stan: DONE (control server), PARTIAL (bramka wyjścia)

Dowód 2026-09-03: własny control server działa w kontenerze `headscale/headscale:v0.29.3`;
Windows i CBMS są zarejestrowane we własnym login server i widzą się bezpośrednio.
Brakujący dowód: backup/restore drill nie został wykonany, a komercyjny login server
nadal jest uruchomiony obok — więc bramka "komercyjny login server nie uczestniczy"
nie jest jeszcze zamknięta.

Zakres:

- wybór i zapis przypiętego upstream commit/release;
- zachowanie BSD-3-Clause;
- osobny moduł Compose/systemd;
- konfiguracja lokalna bez sekretów w Git;
- użytkownicy, preauth keys, ACL/policy;
- rejestracja Darkstar i Windows do własnego login server;
- backup i restore drill;
- brak publicznego API.

Brama wyjścia:

- urządzenia łączą się przez własny control server;
- komercyjny login server nie uczestniczy w teście;
- backup odtwarza stan w środowisku testowym.

### M7 — Headplane i integracja UI

Stan: PLANNED

Zakres:

- wybór i zapis przypiętego upstream commit/release;
- zachowanie MIT;
- ograniczone konto/token API;
- prywatny bind;
- stan i link w Control Room;
- późniejszy adapter dla wybranych operacji;
- test braku wycieku tokena.

Brama wyjścia:

- panel jest dostępny wyłącznie z prywatnej drogi;
- token nie dociera do przeglądarki;
- operacje Darkstar nadal przechodzą przez policy.

### M8 — Odłączenie Tailscale SaaS

Stan: PLANNED

Warunki wejścia:

- M2–M7 zakończone;
- dwie niezależne udane próby restartu;
- fizyczny dostęp do Darkstar;
- działający rollback;
- własny mesh działa z Windows.

Zakres:

- eksport bieżącego stanu;
- wyłączenie autostartu Tailscale;
- test lokalnej i zdalnej ścieżki;
- obserwacja stabilności;
- dopiero później decyzja o odinstalowaniu.

Brama wyjścia:

- żadna funkcja krytyczna nie używa control plane Tailscale SaaS;
- nie ma Serveo;
- odzyskanie dostępu jest udokumentowane.

### M9 — Warlock i Kali

Stan: BLOCKED do czasu dostępności hosta

Zakres:

- migracja SheriffBridge -> WarlockBridge z aliasem danych legacy;
- przeniesienie publicznego ingress/egress i własnego relay/DERP na Warlock;
- Kali jako osobna sandboxowana warstwa;
- Darkstar pozostaje control plane za Kali;
- test segmentacji i policy.

### M10 — Domowa chmura deweloperska

Stan: PLANNED po stabilizacji M9

Kandydaci open source podlegający osobnej analizie:

- Forgejo jako lokalny Git forge;
- runner CI uruchamiany w izolacji;
- registry artefaktów/kontenerów;
- backup do osobnego nośnika;
- synchronizacja z GitHub jako zewnętrznym mirror/remote, nie pojedyncze źródło odzyskania.

Ten etap wymaga osobnej specyfikacji i planu. Nie jest dopisywany do gateway provider.

### M11 — AIONS na serwerze PowerEdge

Stan: BLOCKED do czasu gotowości sprzętu

Zakres wysokiego poziomu:

- inwentaryzacja CPU, RAM, storage, RAID, sieci i GPU;
- bezpieczny firmware/BIOS/iDRAC baseline;
- system hosta i storage;
- konteneryzacja usług AIONS;
- lokalny model;
- backup, monitoring i plan awarii;
- włączenie do prywatnego mesh.

## Reguły aktualizacji roadmapy

1. Status DONE wymaga komendy testowej i zapisanego wyniku.
2. Reboot persistence wymaga zmiany boot_id.
3. Nie kasujemy fallbacku w tym samym kroku, w którym testujemy jego następcę.
4. Każda nowa zależność open source ma zapisane: upstream URL, licencję, wersję/commit, sposób aktualizacji i rollback.
5. Każdy nowy plik projektu ma metadane AUTHOR: M. SZUL.
6. Każdy milestone kończy się małym commitem i aktualizacją dowodu w dokumentacji.
7. Zmiany nazwy Warlock nie mogą zerwać odczytu starszych danych Warlock bez jawnej migracji.
