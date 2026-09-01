<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: GPT-5 Codex
TIMESTAMP: 2026-08-29 11:19:11 Europe/London
REASON FOR CREATION: Utrwalenie zatwierdzonej architektury natywnej bramy Darkstar, prywatnej sieci Headscale/Headplane oraz migracji Warlock do Warlock.
==========================================
-->

# Darkstar Native Gateway and Private Mesh — Design Specification

## Status

Zatwierdzona podstawa do szczegółowego planu implementacji.

Punkt przekazania prac: commit fd8099e4e3c3399ae69a885fa2ed32ab0c57d2df na gałęzi feat/darkstar-module-control.

## 1. Cel

Darkstar ma tymczasowo pełnić rolę Ghost Gate i zapewnić prywatną, samodzielnie kontrolowaną drogę:

~~~text
Internet
  -> Darkstar Ubuntu / Ghost Gate
  -> Windows i prywatne urządzenia
  -> przyszły Kali
  -> AIONS
~~~

Po dołączeniu kolejnej maszyny zewnętrzna warstwa otrzymuje nazwę Warlock, zastępując wcześniejszą nazwę Warlock:

~~~text
Internet
  -> Warlock Bridge
  -> Kali Bridge
  -> Darkstar
  -> AIONS
~~~

Nazwa Warlock pozostaje wyłącznie terminem historycznym do czasu kontrolowanej migracji identyfikatorów w kodzie i danych.

## 2. Potwierdzony stan początkowy

Stan sprawdzony 2026-08-29:

- repozytorium Ubuntu /home/owner/polip-agi jest czyste i wskazuje dokładnie commit fd8099e;
- Windows i Darkstar komunikują się tymczasowo przez adres 100.71.8.70;
- zapisany klucz C:\Users\User\.ssh\darkstar_codex został odrzucony przez
  Ubuntu; jednorazowe logowanie hasłem działa, więc trwałe logowanie kluczem
  musi zostać naprawione i przetestowane przed firewallem/rebootem;
- Tailscale działa i Darkstar oferuje tymczasowy exit node;
- Darkstar ma aktywny systemd service oraz aktywny timer zależności;
- IPv4 i IPv6 forwarding mają wartość 1;
- API Darkstar było wcześniej zdrowe na 127.0.0.1:18080;
- nftables host guard istnieje lokalnie w /etc/darkstar/host-guard.nft;
- darkstar-firewall.service jest enabled, ale inactive/dead i nie ma wpisów w journal;
- firewall został zapisany 2026-08-29, a host uruchomił się 2026-08-28, więc start po prawdziwym restarcie nie został jeszcze udowodniony;
- karta Intel Wireless-AC 8260 z driverem iwlwifi zgłasza obsługę AP;
- narzędzie iw nie jest zainstalowane, dlatego jednoczesny tryb Wi-Fi client + AP nie został potwierdzony;
- przewodowy interfejs enp1s0 istnieje, lecz jest obecnie niedostępny/down;
- Headscale nie jest zainstalowany;
- commit fd8099e dodaje starter React/TypeScript/Vite z React Flow, ale App.tsx pozostaje ekranem startowym Vite;
- istniejący panel Control Room działa jako osadzony widok po stronie Rust;
- cargo fmt przechodzi;
- bazowe cargo test ma trzy istniejące błędy integracyjne: test fixture buduje AppState z nieustawionej zmiennej DARKSTAR_API_TOKEN i otrzymuje 401.

## 3. Zasady architektoniczne

1. GitHub jest źródłem prawdy dla kodu i konfiguracji możliwej do wersjonowania.
2. Sekrety, hasła Wi-Fi, klucze prywatne i tokeny nie trafiają do Git.
3. Darkstar Rust pozostaje zaufanym control plane.
4. Interfejs użytkownika nigdy nie uruchamia bezpośrednio nmcli, nft, systemctl ani Docker.
5. UI wysyła żądanie, Darkstar sprawdza sesję, capability i policy, a dopiero provider wykonuje ograniczoną operację.
6. NetworkManager zarządza profilami i urządzeniami sieciowymi.
7. nftables zarządza filtrowaniem, forward i NAT.
8. systemd zarządza startem po bootowaniu i kolejnością usług.
9. Headscale jest prywatnym control serverem sieci mesh.
10. Headplane jest panelem administracyjnym Headscale, a nie drugim silnikiem polityki.
11. Istniejący Tailscale SaaS pozostaje wyłącznie kanałem awaryjnym do czasu pełnego odbioru drogi natywnej.
12. Serveo pozostaje wyłączone.
13. Usunięcie drogi awaryjnej następuje dopiero po teście restartu i udokumentowanym rollbacku.

## 4. Docelowy przepływ sieci w pierwszym etapie

Preferowana, najbardziej przewidywalna topologia:

~~~text
Router domowy / Internet
  -> Wi-Fi wlp2s0 Darkstar (upstream)
  -> nftables + forwarding Darkstar
  -> Ethernet enp1s0 Darkstar (downstream)
  -> Windows
~~~

Ten wariant nie wymaga, aby jedna karta radiowa jednocześnie odbierała i nadawała Wi-Fi.

Wariant hotspot:

~~~text
Router domowy / Internet
  -> upstream Darkstar
  -> profil AP NetworkManager
  -> prywatny SSID Darkstar
  -> Windows i urządzenia rodzinne
~~~

Jeżeli upstream i AP mają działać na tej samej karcie Intel 8260, wdrożenie może przejść dalej wyłącznie po sprawdzeniu valid interface combinations. Samo WIFI-PROPERTIES.AP=yes potwierdza tryb AP, ale nie potwierdza jednoczesnego AP + managed station.

Jeżeli karta nie wspiera stabilnej współbieżności, obowiązuje jeden z dwóch wariantów:

- Wi-Fi upstream + Ethernet downstream;
- Wi-Fi upstream + druga, jawnie wspierana karta Wi-Fi dla hotspotu.

Nie wolno opisywać jednoczesnego trybu jednej karty jako działającego bez testu utraty pakietów, DHCP, DNS i restartu.

## 5. Moduły

### 5.1 Darkstar Host Guard

Odpowiedzialności:

- walidacja reguł nftables przed ich zastosowaniem;
- domyślna polityka drop dla wejścia;
- pozwolenie na loopback, established/related oraz wymagany ICMP/ICMPv6;
- jawne reguły dla interfejsu zarządzającego i prywatnego downstream;
- forwarding tylko między wskazanym upstream i downstream;
- NAT tylko na wybranym upstream;
- atomowe zastosowanie reguł;
- start systemd przed usługami zależnymi.

### 5.2 Darkstar Native Gateway

Odpowiedzialności:

- wykrycie interfejsów i ich możliwości;
- utworzenie wersjonowalnego szablonu profilu NetworkManager bez sekretów;
- instalacja lokalnego profilu z sekretem poza repozytorium;
- DHCP/DNS dla prywatnego segmentu przez natywny tryb shared NetworkManager albo jawnie wybrany mechanizm;
- kontrolowany start, stop, status i diagnostyka;
- bezpieczny rollback do poprzedniej konfiguracji.

### 5.3 Darkstar Network Provider

Rust provider obsługuje wyłącznie zdefiniowane operacje:

- inspect;
- preflight;
- start;
- stop;
- restart;
- verify.

Provider nie przyjmuje dowolnej komendy shell. Parametry są typowane, nazwy interfejsów są walidowane, a operacje przechodzą przez istniejący ModuleProvider i policy gate.

### 5.4 Headscale

Headscale działa jako osobna usługa o przypiętej wersji lub commit SHA.

Zasady:

- repozytorium upstream i licencja BSD-3-Clause pozostają zachowane;
- własne zmiany mają osobną historię i metadane projektu;
- baza i klucze znajdują się w lokalnych wolumenach poza obrazem;
- API i panel nie są otwarte bezpośrednio na publiczny Internet;
- rejestracja urządzeń jest audytowalna;
- ACL/policy są przechowywane i testowane jako kod;
- publiczne serwery DERP nie są uznawane za docelową niezależną infrastrukturę.

### 5.5 Headplane

Headplane jest wybranym interfejsem, ponieważ funkcjonalnie i wizualnie jest najbliżej panelu Tailscale.

Zasady:

- zachowanie licencji MIT i informacji upstream;
- przypięta wersja lub commit SHA;
- dostęp tylko z prywatnego segmentu lub przez uwierzytelniony Darkstar;
- token Headscale nie trafia do przeglądarki ani repozytorium;
- UI nie omija Darkstar policy dla operacji należących do Darkstar;
- integracja może być linkiem/iframe tylko po analizie nagłówków i izolacji, a docelowo preferuje adapter API i wspólną nawigację.

### 5.6 Control Room React

Checkpoint fd8099e jest świadomym przejściem do osobnego frontendu React/TypeScript/Vite.

React jest właściwy dla:

- interaktywnego grafu React Flow;
- paneli, inspektorów, formularzy i stanu live;
- przyszłego połączenia doświadczeń Darkstar i Headplane.

Rust pozostaje właściwy dla:

- autoryzacji;
- sesji;
- polityki;
- providerów;
- audytu;
- operacji systemowych.

Osadzony widok Rust pozostaje kompatybilnym fallbackiem, dopóki React nie osiągnie pełnego zakresu testów i odbioru.

## 6. Prywatna sieć Headscale

Pierwszy zakres obejmuje:

- serwer Headscale na Darkstar;
- Headplane na Darkstar;
- prywatny namespace użytkowników/urządzeń;
- co najmniej Darkstar i Windows jako dwa urządzenia testowe;
- klient skonfigurowany wyłącznie na własny login server;
- test bez logowania do komercyjnego control plane;
- test bez publicznego wystawiania API;
- udokumentowany eksport konfiguracji i backup bazy.

Pełna niezależność dla urządzeń poza domem wymaga późniejszej warstwy Warlock:

- własnego publicznego wejścia lub kontrolowanego port-forward;
- własnego TLS i DNS;
- własnego DERP albo innej jawnie przyjętej ścieżki relay;
- monitoringu i rotacji kluczy.

Do czasu jej wdrożenia sieć lokalna może być niezależna, ale zdalna łączność przez NAT nie może być deklarowana jako w pełni niezależna.

## 7. Bezpieczeństwo

- Brak dowolnych poleceń shell z HTTP.
- Brak sekretów w URL, logach, obrazach i repozytorium.
- Brak bindu paneli administracyjnych do 0.0.0.0 bez reguł sieciowych i uwierzytelnienia.
- Każda mutacja sieci ma request_id, principal_id, module_id, wynik i wpis audytowy.
- Start/stop/restart wymaga capability oraz zaufanej zgody.
- Niepowodzenie providerów jest odróżnione od odmowy policy.
- Rollback jest przygotowany przed zmianą trasy lub firewalla.
- Tailscale SaaS nie jest wyłączany w tym samym kroku, w którym uruchamiana jest pierwsza wersja nowej drogi.
- Prawdziwy reboot jest wykonywany wyłącznie, gdy operator ma dostęp fizyczny lub drugą sprawdzoną drogę zarządzającą.

## 8. Test restartu, który stanowi dowód

Test persistence musi:

1. zapisać stary /proc/sys/kernel/random/boot_id;
2. zapisać bieżący czas bootowania;
3. potwierdzić działanie kanału awaryjnego;
4. wydać rzeczywiste sudo systemctl reboot;
5. zaobserwować niedostępność hosta;
6. poczekać na ponowną dostępność;
7. odczytać nowy boot_id;
8. wykazać, że nowy boot_id różni się od starego;
9. sprawdzić is-enabled i is-active darkstar-firewall.service;
10. sprawdzić tablicę nftables, forwarding, Darkstar API, timer i routing klienta.

Samo wykonanie helpera, ponowne połączenie SSH albo status enabled nie jest dowodem restartu.

## 9. Migracja Warlock do Warlock

Nowa nazwa kanoniczna to Warlock Bridge.

Migracja kodu będzie osobnym, testowanym krokiem:

- wprowadzenie WarlockBridge w typach Rust;
- zgodność deserializacji legacy warlock_bridge przez okres migracyjny;
- zmiana node_id z warlock-bridge na warlock-bridge z aliasem odczytu;
- aktualizacja testów, dokumentacji, grafu i przykładowych danych;
- brak nagłego zerwania zapisanych zdarzeń lub konfiguracji.

## 10. Poza zakresem pierwszej implementacji

- publiczne wystawienie AIONS;
- pełna platforma GitHub replacement;
- instalacja i migracja AIONS na Dell PowerEdge;
- model Qwen i obsługa GPU;
- pełny Warlock i Kali na oddzielnych hostach;
- usunięcie historycznych identyfikatorów przed migracją danych;
- kopiowanie kodu Tailscale bez zachowania licencji;
- obietnica anonimowości lub zmiany geolokalizacji.

Te elementy pozostają w roadmapie jako osobne etapy.

## 11. Kryteria odbioru architektury

Pierwszy etap jest ukończony, gdy:

1. bazowy zestaw testów jest zielony;
2. konfiguracja firewalla jest w Git i ma zgodne nagłówki;
3. prawdziwy reboot ma dowód zmiany boot_id;
4. firewall, Darkstar i wymagane timery wracają po restarcie;
5. Windows ma tylko jedną aktywną drogę domyślną przez Darkstar podczas testu;
6. wyłączenie upstream Darkstar powoduje kontrolowany brak Internetu na Windows, bez automatycznego przejścia na domowe Wi-Fi;
7. hotspot lub Ethernet downstream działa zgodnie z wynikiem preflight;
8. Headscale i Headplane działają lokalnie bez komercyjnego control plane;
9. operacje administracyjne są dostępne tylko z prywatnej drogi;
10. Control Room pokazuje stan gateway, mesh i firewall;
11. Tailscale SaaS pozostaje tylko fallbackiem albo zostaje wyłączony dopiero po osobnym odbiorze cutover;
12. każdy nowy plik projektu ma wymagany nagłówek AUTHOR: M. SZUL.
