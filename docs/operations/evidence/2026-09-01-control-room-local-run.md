<!-- darkstar-header-v1 -->
<!-- po co: 2026-09-01-control-room-local-run.md -->
<!-- nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie -->
<!-- autor: Marcin -->
<!-- powstal: 2026-09-01 -->
<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Opus 5
TIMESTAMP: 2026-09-02 00:20:00
REASON FOR CREATION: Evidence pierwszego lokalnego uruchomienia Control Room z prawdziwym API (Task 10).
==========================================
-->

# Control Room — pierwsze uruchomienie lokalne

Data: 2026-09-01, Windows. Wszystko na petli zwrotnej, nic nie wystawione na siec.

| Element | Wynik |
|---|---|
| `darkstar-server` przed startem | nic nie nasluchiwalo na 8080/18080/5173 |
| start serwera | `cargo run -p darkstar-server`, `DARKSTAR_HOST=127.0.0.1`, `DARKSTAR_PORT=18081` |
| nasluch | `127.0.0.1:18081` — **wylacznie petla zwrotna**, nie `0.0.0.0` |
| `GET /health` | `200` `{"service":"darkstar","api_version":"darkstar.core/v1","status":"ok"}` |
| `GET /v1/gateway/status` z tokenem | `503` |
| Vite dev | `http://127.0.0.1:5173/`, `host: '127.0.0.1'`, proxy `/v1` → `127.0.0.1:18081` |
| panel bez tokenu | „Brak autoryzacji - podaj token operatora." (oba widoki) |
| panel z tokenem | graf: **17 wezlow / 15 krawedzi**; brama: „Brama nieodczytywalna - host nie odpowiada." |

## Dlaczego brama pokazuje blad, a nie `wlp2s0 / 192.168.2.1`

`503` jest **poprawna odpowiedzia**, nie usterka. Serwer zostal uruchomiony na Windows, a
`GatewayProvider` czyta stan przez `nmcli` i `ip` — polecenia istniejace wylacznie na Ubuntu.
Wykonawca zwraca `CommandFailed`, granica HTTP zamienia to na `503 gateway_unreadable`, a panel
pokazuje komunikat zamiast zmyslonych danych. Fail-closed zadzialal na calej dlugosci lancucha.

Zeby zobaczyc `wlp2s0` i `192.168.2.1`, `darkstar-server` musi dzialac **na CBMS**. Tego nie
robiono — na hoscie nie uruchomiono niczego.

## Co zostalo potwierdzone

- Starter Vite zniknal; strona to Control Room z dwoma panelami i stopka o trybie odczytu.
- Graf systemu rysuje sie z prawdziwego `GET /v1/system-graph` (17 wezlow, m.in. `polip-agi`,
  `capability_gate.rs`, `AIONS / Ionis`, `WPC Engine`, `Ghost Gate`).
- Brak tokenu daje czytelna odmowe w obu panelach, a nie pusty ekran.
- Zaden element interfejsu nie ma przycisku sterujacego brama.

Zrzut ekranu przechowany poza repozytorium (katalog roboczy sesji). Token uzyty w probie byl
lokalna wartoscia deweloperska i **nie znajduje sie w repozytorium**.

## Sprzatanie

Playwright zapisal zrzut do katalogu systemowego Windows; plik zostal stamtad **przeniesiony**,
katalog systemowy jest czysty.
