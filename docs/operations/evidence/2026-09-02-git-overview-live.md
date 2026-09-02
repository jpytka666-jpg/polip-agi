<!-- darkstar-header-v1 -->
<!-- po co: 2026-09-02-git-overview-live.md -->
<!-- nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie -->
<!-- autor: Marcin Szul -->
<!-- powstal: 2026-09-02 -->
<!--
Autor: Marcin Szul
Model: Claude (ta sesja)
Czas: 2026-09-02T22:16:00+01:00
Po co: Zapis pomiaru, ze Control Room pod adresem produkcyjnym czyta stan repozytorium
       z darkstar-server zamiast z posrednika deweloperskiego, ktory tam nie istnieje.
-->

# Git overview na zywo — 2026-09-02

Sterownia pod `http://127.0.0.1:18080/` przestala pytac o `/__darkstar/git`. Ten adres
istnieje wylacznie w trybie deweloperskim Vite; pod adresem produkcyjnym odpowiada 404,
wiec panel swiecil pustka. Zrodlem danych jest teraz `GET /v1/git/overview` z serwera.

## Co bylo zepsute

Panel czytal posrednika, ktorego pod adresem produkcyjnym nie ma. Zielone testy niczego
tu nie chronily — sprawdzaly warstwe, ktora dzialala, a nie adres, pod ktory panel siegal.

## Pomiar — osiem linii, maszyna CBMS

Po `docker compose up -d --force-recreate` na obrazie `066713829e07`:

```
1 health            : 200
2 strona Sterowni    : 200
3 brama z PIN        : 200
4 git bez PIN        : 401
5 git z PIN          : 200
6 stary adres 404    : 404
7 stan programu      : healthy obraz=066713829e07
8 ile zmian w widoku : 10
```

Linia 6 jest tu najwazniejsza: martwy adres nadal odpowiada 404 i to jest poprawne.
Znaczenie ma to, ze panel juz go nie wola.

## Pomiar — czego przegladarka faktycznie zazadala

Wejscie na `http://127.0.0.1:18080/` bez PIN-u, pelna lista zapytan innych niz statyczne:

```
GET /v1/gateway/status          => 401
GET /v1/context/health          => 401
GET /v1/context/search?q=...    => 401
GET /v1/system-graph            => 401
```

**Ani jednego zapytania do `/__darkstar/git`.** Panel Gita bez PIN-u nie pyta wcale —
prosi o PIN zamiast strzelac w serwer zapytaniem, ktore i tak moze skonczyc sie 401.
To jest dowod na "zero 404" mocniejszy niz zrzut ekranu: nie ma zadnego zapytania,
ktore mogloby 404 zwrocic.

## Pelna odpowiedz serwera, odczytana na zywo

```
galaz:          docs/darkstar-headscale-hotspot-plan
HEAD:           8bdfbb124f17     (przed zmiana panelu)
brudne:         False
upstream:       True  origin/docs/darkstar-headscale-hotspot-plan
do przodu/tylu: 0 0
ile zmian:      10      <- wejscie mialo 11, limit trzyma serwer
```

## Testy

```
cargo test -p darkstar-server            53 passed, 0 failed
cargo test --workspace (w budowie obrazu) 54 passed, 0 failed
node --test tests/gitOverview.test.ts      2 passed, 0 failed
npx tsc -b                                 exit 0
npx eslint src/GitPanel.tsx src/App.tsx src/api.ts   czysto
```

## Zmiany

| Numer | Opis |
|---|---|
| `acc63fa` | `feat(darkstar): git overview on darkstar-server` |
| `8bdfbb1` | `fix(darkstar): mount the git worktree into the container` |
| `5aec460` | `feat(darkstar): read the git overview from the server in Control Room` |

## Trzy braki, ktore wyszly z czytania przepisu, nie z testow

1. Obraz produkcyjny stoi na `debian:bookworm-slim` i **nie mial gita**.
2. Worktree nalezy do operatora hosta, a proces biegnie jako uid 10001 — git odmawia
   czytania cudzego katalogu (`dubious ownership`).
3. Montaz jest `:ro`, a `git status` domyslnie odswieza indeks, czyli **pisze**.
   Flaga `--no-optional-locks` to wylacza.

Kazdy z tych trzech dawalby ten sam objaw: 503 na sciezce, ktora przechodzi 4/4 testy.

## Czego tu nie ma

Zrzutu ekranu z narysowanym grafem po wpisaniu PIN-u. PIN-u operatora nie wpisuje —
to poswiadczenie i wpisanie go nalezy do operatora. Panel z grafem wymaga tego jednego
gestu; wszystko przed nim i za nim jest zmierzone powyzej.

## Nietkniete

Siec, `nft`, dysk E, pociagi w Sterowni, Headscale. Sprawdzone po fakcie:
`DARKSTAR-WiFi` na `enp1s0` aktywne, `192.168.2.1/24` na miejscu, wyjscie do swiata
odpowiada. Zadnej komendy zmieniajacej siec w tej sesji nie bylo.
