<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: GPT-5 Codex
TIMESTAMP: 2026-08-29 11:19:11 Europe/London
REASON FOR CREATION: Utworzenie jednego kanonicznego harmonogramu rozwoju Darkstar od checkpointu fd8099e przez Ghost Gate, Headscale/Headplane, Warlock/Kali i przyszłą infrastrukturę AIONS.
==========================================
-->

# Darkstar Roadmap

## Źródło prawdy

- Repozytorium: jpytka666-jpg/polip-agi
- Gałąź rozwojowa: feat/darkstar-module-control
- Punkt przekazania: fd8099e4e3c3399ae69a885fa2ed32ab0c57d2df
- Szczegółowa specyfikacja: docs/superpowers/specs/2026-08-29-darkstar-native-gateway-private-mesh-design.md
- Szczegółowy plan: docs/superpowers/plans/2026-08-29-darkstar-native-gateway-headscale-headplane-plan.md

## Legenda

- DONE — wykonane i potwierdzone testem.
- PARTIAL — część działa, ale brakuje pełnego dowodu.
- NEXT — najbliższy etap wykonawczy.
- PLANNED — zaplanowane po spełnieniu wcześniejszych bramek.
- BLOCKED — wymaga sprzętu, decyzji lub zewnętrznego warunku.

## Stan na 2026-08-29

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

Stan: PLANNED

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
