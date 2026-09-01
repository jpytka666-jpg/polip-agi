<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Opus 5
TIMESTAMP: 2026-09-02 04:40:00
REASON FOR CREATION: Zmierzony stan pojedynczego runtime Darkstar na CBMS po scaleniu. Same fakty.
==========================================
-->

# CBMS — stan po scaleniu, jeden runtime

Data: 2026-09-02. Pomiar, nie opis.

| # | Pomiar | Wynik |
|---|---|---|
| 1 | `docker ps` | `darkstar  Up 16 hours (healthy)  127.0.0.1:18080->8080/tcp` |
| 2 | `GET 127.0.0.1:18080/health` | `200` `{"service":"darkstar","api_version":"darkstar.core/v1","status":"ok"}` |
| 3 | `GET /v1/context/health` (token z `deploy/.env`) | **`404`** |
| 4 | Chroma venv | `heartbeat v2 = 200`, **36 kolekcji** |
| 5 | Windows `E:` na `:8000` | **ZYJE**, `200` (proces `chroma` PID 5080) |
| 6 | `git -C /home/owner/polip-agi log -1` | `ca2e1cf` na `docs/darkstar-headscale-hotspot-plan` |
| 7 | `tailscaled` | `active` |
| 8 | `/var/lib/darkstar/chroma` | **371 plikow, 174 MB** (scp zakonczony) |
| 9 | `/home/owner/darkstar/src` | **USUNIETY** — koniec dublowania |

## Dlaczego punkt 3 to 404

Checkout jest na `ca2e1cf` i **ma** `context_http.rs`, ale **kontener chodzi na obrazie sprzed
16 godzin**, zbudowanym z `4a54d93`. Sciezka `/v1/context/health` w tamtym obrazie nie istnieje.
Token jest poprawny i nie jest przyczyna — brak tokenu dalby `401`, nie `404`.

## Dlaczego przebudowa nie przeszla

`cargo test --workspace --locked` w kontenerze buduajacym:

~~~text
Could not resolve host: index.crates.io   ->  kod 101
~~~

Potwierdzone osobno: `docker run debian` → `getent hosts deb.debian.org` = **DNS_FAIL**.

Przyczyna w zywej zaporze: `chain forward` ma `policy drop` i przepuszcza wylacznie
`ip saddr @darkstar_downstream_ipv4` (czyli `192.168.2.0/24`) z `enp1s0` na `wlp2s0`.
Siec Dockera `172.17.0.0/16` nie ma zadnej reguly. Host guard aktywowano 2026-08-29 18:51,
a obraz powstal tego samego dnia ~06:15 — **przed** zapora. To pierwsza budowa od jej wlaczenia.

## Stan runtime — bez dwuznacznosci

Wybrano **dokonczenie**, nie rollback: kontener caly czas stoi na dzialajacym starym obrazie
(`healthy`), wiec nie bylo czego cofac. Przebudowa wznowiona z `--network=host` — to omija
odciecie **bez zmiany zapory**. Zapora nie byla i nie jest modyfikowana.

## Czego nie zrobiono

- Chroma **nie jest pusta** — 36 kolekcji, 371 plikow; sync z E: zakonczony i zmierzony.
- Headscale — pliki leza w `deploy/headscale/`, **nie instalowany**.
- `tailscaled` nietkniety. Zrodlo na `E:` nietkniete.
