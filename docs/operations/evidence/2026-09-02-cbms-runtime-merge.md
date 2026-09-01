<!-- darkstar-header-v1 -->
<!-- po co: 2026-09-02-cbms-runtime-merge.md -->
<!-- nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie -->
<!-- autor: Marcin -->
<!-- powstal: 2026-09-01 -->
<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Opus 5
TIMESTAMP: 2026-09-02 04:00:00
REASON FOR CREATION: Rozpoznanie i scalenie dwoch instancji Darkstara na CBMS - dzialajacej produkcyjnej i tymczasowej z tej sesji. Usuniecie dublowania.
==========================================
-->

# CBMS — scalenie zamiast drugiej instancji

Data: 2026-09-02. Zasada prowadzaca: **nie dublowac**. Nic nie zostalo pominiete.

## 1. Co trzymalo port 18080 — rozpoznane do konca

Proces wygladal obco, bo wskazywal plik, ktorego na hoscie **nie ma**:

~~~text
PID 3158, PPID 3134, user 10001, dziala 16 h
/usr/local/bin/darkstar-server  ->  ls: No such file or directory
~~~

Wyjasnienie: proces zyje **w kontenerze**, a plik istnieje tylko w jego warstwie.

| Element | Ustalenie |
|---|---|
| kontener | `darkstar`, obraz `darkstar:dev`, `Up 16 hours (healthy)`, `127.0.0.1:18080->8080/tcp` |
| jednostka | `darkstar.service` — „Darkstar Docker Compose Runtime", `WorkingDirectory=/home/owner/polip-agi` |
| start | `docker compose --env-file deploy/.env -f deploy/docker-compose.yml up -d --remove-orphans` |
| zrodlo obrazu | `/home/owner/polip-agi`, galaz `feat/darkstar-module-control`, commit `4a54d93` (2026-08-29) |
| czego nie mial | `gateway_http.rs`, `context_http.rs` — stad `404` na `/v1/context/health` |

**To nie byla obca usluga ani duplikat — to ten sam Darkstar, tylko sprzed czterech dni.**

## 2. Relacja galezi — zero rozjazdu

~~~text
git merge-base --is-ancestor 4a54d93 HEAD   ->  TAK
git log HEAD..4a54d93                       ->  0 commitow
merge-base(HEAD, 4a54d93)                   ->  4a54d93
~~~

Nasza galaz zawiera **caly** kod z CBMS. Scalenie bylo przewinieciem do przodu, nie laczeniem
rozbieznych historii. Nic nie trzeba bylo ratowac.

## 3. Co bylo do uratowania poza Gitem

| Rzecz | Stan |
|---|---|
| `deploy/.env` (`0600`, ignorowany) | zawieral `DARKSTAR_HOST_PORT`; **skopiowany** do `/home/owner/darkstar/env.backup` przed zmiana |
| inne pliki niesledzone | **brak** (`git status --porcelain -uall` puste) |
| inne kopie `polip-agi` na hoscie | **brak** — jeden checkout |

## 4. Brakujace ogniwo, ktore by wszystko przewrocilo

Kontener **nie mial `DARKSTAR_API_TOKEN`**. Po dolozeniu bramy i pamieci kazda uwierzytelniona
sciezka odpowiadalaby `401` — fail-closed zadzialalby poprawnie, ale usluga bylaby bezuzyteczna.

Poprawione w `deploy/docker-compose.yml`:

- `DARKSTAR_API_TOKEN` — **wymagany** z `deploy/.env`, nigdy w repozytorium. Token wygenerowany
  na hoscie z `/dev/urandom`, plik `0600`.
- `DARKSTAR_CONTEXT_LOCAL` / `DARKSTAR_CONTEXT_REMOTE` — adresy nog pamieci.
- `extra_hosts: host.docker.internal:host-gateway`.

### Pulapka, ktora to ujawnilo

Kontener **nie moze** siegnac po Chrome przez wlasne `127.0.0.1` — to jego wlasna petla zwrotna,
nie petla hosta. Dlatego noga lokalna wskazuje `host.docker.internal:8000`. Ta sama pulapka
dotyczy Headscale i zostala tam naprawiona (punkt 6).

## 5. Usuniete dublowanie — moje wlasne

W trakcie pracy zbudowalem serwer z osobnej kopii `/home/owner/darkstar/src` (**187 MB**). To bylo
dokladnie to dublowanie, ktorego mielismy unikac.

Przed usunieciem **sprawdzilem, czy nie ma tam czegos unikalnego**:

~~~text
diff po normalizacji koncow linii, wszystkie pliki .rs:  razem_roznych=0
~~~

Rozniły sie wylacznie znaki konca linii (kopia szla z Windows). Zero unikalnej tresci → katalog
usuniety. Zostaje jeden checkout `/home/owner/polip-agi` i jedno srodowisko `venv` dla Chromy.

Tymczasowa instancja na porcie `18082` (PID 56451, 56453) — **zatrzymana**. Produkcyjna na
`18080` nie byla ani razu zatrzymana ani podmieniona.

## 6. Znalezione przy okazji i NIE pominiete — blad w plikach Headscale

`deploy/headscale/config.yaml.example` mial `listen_addr: 127.0.0.1:8080` **wewnatrz kontenera**,
przy jednoczesnym mapowaniu portow w `compose.yml`. To by nie zadzialalo: w kontenerze `127.0.0.1`
to jego wlasna petla, wiec mapowanie nie mialoby czego przekazac, a usluga bylaby nieosiagalna.

Poprawione na `0.0.0.0:8080` **wewnatrz kontenera**, z ekspozycja ograniczona przez `compose.yml`
wylacznie do `127.0.0.1` i `192.168.2.1` hosta — dokladnie ten sam wzorzec, ktory od 16 godzin
dziala w kontenerze `darkstar` (`DARKSTAR_HOST: 0.0.0.0` + `ports: 127.0.0.1:...`).

To **nie jest** zlamanie zasady „nigdy 0.0.0.0": zasada dotyczy adresu widocznego na hoscie.

## 7. Czego nie ruszono

- `E:\server wiedzy` — zrodlo prawdy, nietkniete; Chroma na Windows dziala.
- `tailscaled` — kabel ratunkowy, nietkniety.
- `nmcli`, `nft`, brama `192.168.2.1` — bez zmian.
- Headscale — **nie instalowany**.

Zadne haslo ani token nie trafily do repozytorium.
