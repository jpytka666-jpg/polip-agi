<!-- darkstar-header-v1 -->
<!-- po co: README.md -->
<!-- nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie -->
<!-- autor: Marcin -->
<!-- powstal: 2026-09-04 -->
<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Opus 5
TIMESTAMP: 2026-09-04 01:00:00
REASON FOR CREATION: Zapis ustalen z przegladu upstreamu Headplane i zasad wdrozenia panelu (Task 14, Step 14.1 i 14.2).
MECHANICS: Opis przypietej wersji, macierzy zgodnosci z Headscale, zmierzonego portu, obslugi sekretow
           oraz tego, co ma sprawdzac test prywatnego nasluchu z kroku 14.3.
SYSTEM PART: deploy/headplane - panel administracyjny prywatnego mesh.
ARCHITECTURE FUNCTION: Headplane jest oknem na Headscale, nie wladza nad nim.
DEPENDENCIES/LINKS: version.lock, compose.yml, config.yaml.example, deploy/headscale/, darkstar_host_guard.
TECH STACK: Markdown - konwencja katalogu deploy/.
LOCAL WORKSPACE: D:\codex-fresh-2026-08-28\worktrees\polip-agi-darkstar-plan
GIT COMMIT: PENDING
GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
==========================================
-->

# Headplane — przypiecie wersji i zasady nasluchu

**Nic z tego katalogu nie jest wdrozone.** Zaden kontener nie zostal uruchomiony, zaden port nie
zostal zajety. To sa cztery pliki w repozytorium: co wolno zainstalowac, na jakim adresie i pod
jakimi warunkami. Uruchomienie to dopiero kroki 14.5 i 14.6 planu.

Headplane to panel WWW dla Headscale. Headscale trzyma mesh; Headplane tylko go pokazuje i pozwala
klikac. **Panel jest wymienny, mesh nie.** Gdyby panel trzeba bylo wyrzucic, siec dziala dalej.

## Co zostalo przypiete — Step 14.2

| Pole | Wartosc |
|---|---|
| Upstream | `https://github.com/tale/headplane` |
| Licencja | MIT, Copyright (c) 2024 Aarnav Tale — sprawdzone w pliku `LICENSE` pod tagiem |
| Tag wydania | `v0.7.1`, opublikowany 2026-08-28 |
| Commit | `30c842ed8dbf011e78596e3a474133ec4069ba79` |
| Obiekt tagu | `dde4395a6bb7a07c38896002781389282f2e0d85`, podpis ssh-ed25519 zweryfikowany |
| Obraz | `ghcr.io/tale/headplane:0.7.1` |
| Digest obrazu | `sha256:394483fd2336ce6b15df360c3e118ddfd45eb8140fe21df9f2611077f5d79d22` |
| Digest amd64 | `sha256:25cc988fc123c1125b7029a351143f7f1d30eb0d4c1891673e1b70c061dd8593` |

Dwie rzeczy warte zapamietania:

1. **Tag obrazu nie ma litery `v`.** Zmierzone przez API rejestru: `:0.7.1` odpowiada `200`,
   `:v0.7.1` odpowiada `404`. Tag gita ma `v`, tag obrazu nie ma.
2. **Obraz udowodnil swoje pochodzenie.** Etykieta `org.opencontainers.image.revision` w konfiguracji
   obrazu to dokladnie `30c842ed...`, czyli commit tagu `v0.7.1`. Obraz i kod to ta sama rzecz.

`compose.yml` wskazuje obraz przez digest, nie przez tag. Gdyby ktos przepial tag `0.7.1` na inna
zawartosc, docker odmowi startu. **Nigdzie nie ma slowa `latest`.**

## Zgodnosc z Headscale v0.29.3 — Step 14.1

Na hoscie CBMS dziala Headscale `v0.29.3` (kontener `darkstar-headscale`, panel odpowiada `200`).
Pytanie brzmialo: czy Headplane 0.7.1 z nim wspolpracuje. Fakty odczytane z upstreamu pod tagiem
`v0.7.1`, w kolejnosci waznosci:

| Fakt | Zrodlo |
|---|---|
| Minimum to Headscale **0.27.0** | `docs/install/docker.md` |
| Testy integracyjne upstreamu chodza na `0.27.0, 0.27.1, 0.28.0, 0.29.0, 0.29.1` | `tests/integration/setup/env.ts`, stala `HS_VERSIONS` |
| Wlasny `compose.yaml` upstreamu uzywa Headscale `0.29.2` | `compose.yaml` |
| Mozliwosci Headscale wykrywane sa przez `/version` i porownanie „nie starsze niz" | `app/server/headscale/api/capabilities.ts` |
| Przegladarkowe SSH jest zepsute od `0.29.0-beta` do `0.29.1`; wymagane `0.28.x` albo `0.29.2+` | `docs/features/ssh.md` |

**Wniosek:** para Headplane 0.7.1 + Headscale 0.29.3 jest zgodna wedlug regul, ktore upstream sam
zapisal. 0.29.3 jest powyzej minimum, jest nowsze niz wersja z wlasnego compose upstreamu, i jest
juz po poprawce psujacej SSH.

**NIEZWERYFIKOWANE:** upstream nigdzie nie pisze wprost „0.29.3". Jego macierz testow konczy sie na
`0.29.1`. To nie jest sprzecznosc — to znaczy, ze zgodnosc jest wywnioskowana z regul, a nie
potwierdzona przebiegiem. Potwierdzeniem bedzie dopiero **Step 14.6**: panel polaczony z API
Headscale, lista wezlow widoczna w UI. Do tego czasu w `version.lock` stoi
`headscale_0_29_3_status = zgodne-wedlug-regul-upstreamu / niepotwierdzone-przebiegiem`.

## Model uwierzytelniania

Headplane 0.7.1 ma trzy drogi wejscia. Zadna nie jest wlaczona automatycznie.

| Droga | Na czym polega | Kiedy uzyc |
|---|---|---|
| Klucz API Headscale | Operator wkleja w formularzu klucz z `headscale apikeys create` | na pierwsze uruchomienie |
| OIDC | Zewnetrzny dostawca tozsamosci, role z roszczen | dopiero gdy taki dostawca stanie w sieci Darkstar |
| `server.proxy_auth` | Uwierzytelnia posrednik przed panelem, panel ufa naglowkom z podanych podsieci | tylko z zaufanym posrednikiem |

Pierwszy zalogowany uzytkownik dostaje role wlasciciela. Panel ma wlasny model rol
(`owner`, `admin`, `network_admin`, `it_admin`, `auditor`, `viewer`, `member`) — to rola **w panelu**,
nie w polityce mesh. Polityke nadal rozstrzyga `deploy/headscale/policy.hujson`.

## Uprawnienia — do plikow i do API

Wynik przegladu, zestawiony z tym, co dostaje nasz kontener:

| Czego wymaga upstream | Po co | Nasza decyzja |
|---|---|---|
| Odczyt `/etc/headplane/config.yaml` | wlasna konfiguracja | montaz **tylko do odczytu** |
| Zapis w `/var/lib/headplane` | baza sesji i cache panelu | wolumen `headplane_data` |
| Klucz API Headscale | operacje po stronie serwera | plik `0600` roota, przez docker secret |
| Odczyt i **zapis** `config.yaml` Headscale | edycja DNS i ustawien mesh z UI | montaz **tylko do odczytu** — panel pokazuje, nie zmienia |
| Dostep do `/var/run/docker.sock` | restart Headscale po zmianie DNS | **ODMOWA.** Gniazdo dockera to rownowaznik roota na bramie |

Odmowa gniazda dockera kosztuje: z UI nie da sie edytowac DNS ani ustawien Headscale. Tak ma byc.
Krok 14.7 planu mowi wprost, ze Headplane nie jest organem polityki, a panel osiagalny z sieci
klienckiej nie moze miec uchwytu, ktorym restartuje kontenery na bramie.

Obraz jest **distroless** i startuje jako `uid 0` — odczytane z konfiguracji obrazu (pole `User`),
nie zgadniete. Nie ma w nim powloki: `/bin/sh` i `/bin/bash` to atrapy odsylajace do obrazu debug.
Dlatego pliki sekretow moga byc `root:root 0600` i beda czytelne, a `compose.yml` dokrecca reszte:
`read_only`, `cap_drop: ALL`, `no-new-privileges`.

## Sekrety

Dwa pliki. **Zadnego z nich nie ma w repozytorium i nigdy nie bedzie.** Powstaja na hoscie CBMS:

| Plik | Zawartosc | Prawa |
|---|---|---|
| `/etc/darkstar/headplane/cookie_secret` | 32 znaki losowe, sekret sesji WWW | `root:root 0600` |
| `/etc/darkstar/headplane/headscale_api_key` | klucz z `headscale apikeys create` | `root:root 0600` |

Klucz API powstaje na hoscie poleceniem
`docker exec darkstar-headscale headscale apikeys create --expiration 90d`.

`compose.yml` podaje oba jako docker secrets, wiec w kontenerze pojawiaja sie pod `/run/secrets/`,
a konfiguracja wskazuje na nie polami `cookie_secret_path` i `headscale.api_key_path`. Wartosc
sekretu nie wchodzi do obrazu, nie wchodzi do repozytorium i nie jest wysylana do przegladarki.
`config.yaml.example` zawiera wylacznie atrapy.

`info_secret` celowo nie jest ustawiony — bez niego endpoint `/api/info`, ktory wystawia wersje
Headplane i Headscale, pozostaje wylaczony.

## Port — zmierzony, nie wybrany

Pomiar na hoscie CBMS, 2026-09-04 00:33 UTC:
`ssh -o BatchMode=yes owner@100.64.0.2 'ss -lnt'`

| Adres i port | Kto | Wniosek |
|---|---|---|
| `192.168.2.1:8080` | Headscale | zajete — **nie ruszac** |
| `127.0.0.1:18080` | kontener `darkstar` | zajete — nie ruszac |
| `127.0.0.1:8000`, `:8001`, `:11434`, `:18789`, `:20241`, `:20242`, `:39099`, `:631` | inne uslugi hosta | zajete |
| `:3000` | **brak wpisu** | **wolne — bierzemy** |

Zapytanie celowane `ss -lnt | grep -E ':(3000|3001|8080)'` zwrocilo wylacznie wiersz Headscale.
Port `3000` to jednoczesnie domyslny port Headplane, wiec nie trzeba niczego naginac.

## Adresy nasluchu — wiazace

| Zasada | Wartosc |
|---|---|
| Dozwolone | `127.0.0.1` (domyslne) oraz `192.168.2.1` |
| **Zakazane** | `0.0.0.0`, `::`, dowolny adres z `192.168.1.0/24`, `10.44.0.0/24` |
| Zapora | dostep wylacznie z `192.168.2.0/24`; `darkstar_host_guard` juz ma ten zakres |

Kontener stoi w **sieci hosta**, dokladnie z tego samego powodu co Headscale: ten komputer jest
brama domowa, a wlasna siec dockerowa dolozylaby mostek i reguly przekierowania do zywej maszyny
routujacej. Skoro tak, o ekspozycji decyduje wylacznie adres nasluchu — i dlatego jest on wymuszony
w dwoch miejscach naraz:

- w `config.yaml` polem `server.host`,
- w `compose.yml` zmienna `HEADPLANE_SERVER__HOST`, ktora ma **pierwszenstwo** nad plikiem
  konfiguracyjnym (upstream, `docs/configuration/index.md`).

Zeby wystawic panel publicznie, trzeba by zmienic obie wartosci i jeszcze przepuscic ruch przez
zapore. Jedna pomylka nie wystarczy.

## Test prywatnego nasluchu — czego wymaga Step 14.3

Testu jeszcze nie ma; ten rozdzial mowi, co ma robic. Zasada jest jedna: **test ma sie czerwienic
sam z siebie, zanim ktokolwiek zdazy zauwazyc problem** — a wiec musi byc uruchamiany przed i po
kazdym wdrozeniu panelu.

Test ma **przejsc**, gdy wszystkie ponizsze sa prawda, i **polec**, gdy pada choc jedna:

| Sprawdzenie | Czerwone gdy |
|---|---|
| Nasluch | `ss -lnt` na hoscie pokazuje `:3000` na `0.0.0.0`, `::`, `192.168.1.x` albo na jakimkolwiek adresie spoza `{127.0.0.1, 192.168.2.1}` |
| Zrodlo prawdy w plikach | w `compose.yml` albo w `config.yaml` znajduje sie `0.0.0.0` lub `::` przy adresie panelu |
| Segment nadrzedny | polaczenie na `3000` z adresu z `192.168.1.0/24` (siec Vodafone) **nawiazuje sie**; oczekiwane jest odrzucenie albo cisza |
| Interfejs `wlp2s0` | port `3000` jest osiagalny na adresie IP przypisanym do `wlp2s0` |
| Zapora | `darkstar_host_guard` nie istnieje albo polityka `input` przestala byc `drop` |
| Przypiecie | obraz kontenera nie zgadza sie z digestem z `version.lock`, albo gdziekolwiek pojawia sie `latest` |
| Wyciek sekretu | klucz API Headscale albo sekret ciasteczka wystepuje w odpowiedzi HTTP panelu, w dostarczonym JS albo w logach kontenera |

Dwie pulapki, ktore test musi ominac:

1. **Sprawdzenie od strony hosta nie wystarczy.** `ss` pokaze adres nasluchu, ale nie odpowie na
   pytanie, czy zapora faktycznie odcina segment nadrzedny. Potrzebna jest proba polaczenia
   z drugiej strony — z maszyny w `192.168.1.0/24` — i **brak** odpowiedzi jest wynikiem pozytywnym.
2. **Kontener jest distroless.** Nie ma w nim `ss`, `netstat`, `wget` ani powloki. Test musi patrzec
   z hosta (`ss`, `docker inspect`, `curl`), a nie przez `docker exec` — inaczej sam sie wywroci
   i nikt nie odrozni awarii testu od awarii zabezpieczenia.

## Czego nie ruszac

- Kontenera `darkstar-headscale`, portu `8080` i `deploy/headscale/` — panel jest dodatkiem,
  mesh dziala i ma dzialac dalej.
- `deploy/docker-compose.yml`, `deploy/.env*`, portu `18080` — inny obszar pracy.
- `wlp2s0`, profilu Vodafone, `DARKSTAR-WiFi`, `192.168.2.1` — dzialajaca brama.
- `tailscaled` — wciaz jedyna droga ratunkowa do hosta.
- `/etc/darkstar/host-guard.nft` recznie — zmiany tylko przez wersjonowany plik.

## Co zostaje do zrobienia w Task 14

Kroki 14.1 i 14.2 sa zamkniete. Stan pozostalych, zmierzony na zywym hoscie:

- **14.3 — PASS (recznie zmierzone, skrypt testu nadal nie istnieje).** Operator zmierzyl:
  `ss` pokazuje wylacznie `127.0.0.1:3000`, `/admin` przez petle = `302`, LAN (`192.168.2.0/24`)
  = `000` - potwierdzone po tym samym przeladowaniu zapory, ktore otworzylo port 18080, wiec
  ten pomiar jest tez kontrola, ze 3000 nie zostal przy okazji otwarty. Dowod i dokladne
  pokrycie siedmiu wierszy checklisty: `docs/operations/evidence/2026-09-04-headplane-private-listen.md`.
  Automatyczny skrypt opisany w rozdziale powyzej wciaz nie istnieje - to pomiar reczny.
- **14.4 i 14.5 — DONE.** Sekrety sa na hoscie w `/etc/darkstar/headplane/*`, `config.yaml` jest
  zlozony, panel jest live na `127.0.0.1:3000`: `docker ps` pokazuje `darkstar-headplane` jako
  `Up ... (healthy)`, `cookie_secret` ma 32 bajty z prawami `0600` i wlascicielem `root:root`,
  a `127.0.0.1:3000/admin` odpowiada `302`. Dowod: `docs/operations/evidence/2026-09-04-network-done.md`,
  sekcja "Headplane — petla live 2026-09-04T13:28:40+01:00". Krok 14.5 nie oznacza LAN bind -
  petla zwrotna JEST docelowym stanem, tak jak Control Room. `192.168.2.1:3000` ma zostac `000`;
  to jest warunek PASS dla 14.3, nie zaleglosc do domkniecia. `192.168.2.1:3000` i `0.0.0.0` sa
  poza zakresem tego zadania.
- **14.6 — CZESCIOWO.** `deploy/systemd/darkstar-headplane.service` istnieje (wzorowany na
  `darkstar-headscale.service`), ale nie jest zainstalowany ani `enable`d na hoscie — panel dziala
  dzis bez niego. Rozszerzenie `verify` o sprawdzenia sensowne dopiero na dzialajacym kontenerze
  pozostaje nadal otwarte.
- **14.7 — DONE.** `runtime:headplane` w `crates/darkstar-core/src/system_graph.rs`, jedna
  krawedz `admin_view_of -> runtime:headscale`. Commit `4019b4d`.
- **14.8 — DONE (istnialo juz wczesniej).** Pasek statusu Headplane w `SystemGraph.tsx`
  (`headplanePanelView`) czytal `/v1/world/status` jeszcze przed ta sesja - sprawdzone, nie
  dopisane od nowa.
- **14.9 — DONE.** `headplaneAccessNote()` i `headplaneTunnelLink()` w `frontend/src/api.ts`:
  link `http://127.0.0.1:3001/admin` istnieje WYLACZNIE w Control Room i tylko gdy sonda
  zmierzyla `up`; w kazdym innym stanie kafelek pokazuje "zamkniety", nigdy
  `192.168.2.1:3000`. `frontend/public/world` nietkniety. Commit `a794fff`.
- **14.10 — DONE.** Testy granic uprawnien: sonda Headplane przypieta do `127.0.0.1:3000` w
  `main.rs` (nigdy `0.0.0.0`), a `frontend/public/world/{index.html,world.js}` nie zawieraja
  ani tunelu, ani adresu LAN, ani `0.0.0.0`. Commit `9b767aa`.
- **14.11 — DONE, opis ponizej byl bledny.** `deploy/headplane/verify-config` (nowy skrypt)
  uruchamia `docker compose -f compose.yml config` na Windows, bez SSH do CBMS. Zmierzone
  naprawde: `config` konczy sie kodem `0` NIEZALEZNIE od tego, czy pliki sekretow istnieja -
  to polecenie sprawdza wylacznie skladnie YAML, nigdy istnienia sciezek na dysku. To
  sprawdzenie robi dopiero `up`/`create`, ktorego skrypt celowo nie wola. Commit `1789eec`.
- **14.12 — ten wpis jest bramka.** Wszystkie powyzsze siedem krokow (14.4-14.11) ma commit i
  dowod. Zostaja swiadomie OTWARTE, bo naleza do innego zadania niz operacyjne domkniecie
  Task 14: 14.6 instalacja unitu na hoscie (plik istnieje, nikt go nie `enable`owal - operator
  robi to recznie), oraz zakres poza `deploy/`/`crates/darkstar-core`/`frontend`: wezel w
  Control Room jako osobny widok `MeshPanel`, dodatkowa nawigacja i testy granic uprawnien
  we frontendzie wykraczajace poza status i link opisane w 14.9/14.10.

`docker compose -f deploy/headplane/compose.yml config` sprawdza WYLACZNIE skladnie -
poprawnosc YAML, interpolacje zmiennych, rozwiazanie `secrets:`/`volumes:` do ich definicji.
NIE sprawdza, czy pliki pod `/etc/darkstar/headplane/*` albo `deploy/headplane/config.yaml`
naprawde istnieja na dysku - te sciezki staja sie prawdziwym problemem dopiero przy `up` albo
`create`, ktorych ten dokument nigdy nie zaleca uruchamiac z tego katalogu bez fizycznego
dostepu do CBMS. Pliki `/etc/darkstar/headplane/*` powstaja dopiero na hoscie, w kroku 14.4,
i nie naleza do repozytorium.
