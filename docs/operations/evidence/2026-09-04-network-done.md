<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: GPT-5 Codex
TIMESTAMP: 2026-09-04 02:15:01 Europe/London
REASON FOR CREATION: Rozdzielenie gotowego kodu i audytu sieci od nadal niewykonanej podmiany starego obrazu Darkstar na CBMS.
==========================================
-->

# Droga do świata — bramka przed podmianą obrazu, 2026-09-04

## Werdykt

Kod jest gotowy i sprawdzony lokalnie. **CBMS nie jest jeszcze wdrożony do końca**:
uruchomiony kontener nadal pochodzi ze starego obrazu i ma tylko listener
`127.0.0.1:18080`. Ten plik nie udaje dowodu wdrożenia. Stan `DONE` wolno nadać dopiero,
gdy pomiar po podmianie obrazu pokaże oba listenery, `/world/` oraz odmowę `401` z adresu
bramy bez tokenu.

## Stan live przekazany przez operatora

Te fakty pochodzą z pomiarów wykonanych przed tą kontynuacją; w tej sesji nie łączono się
z CBMS i nie zmieniano sieci hosta.

```
Headscale                    192.168.2.1:8080 — działa
telefon + One Agent          /windows — działa
Chrome                       /windows — działa
darkstar-server              ss: tylko 127.0.0.1:18080
darkstar-server przez LAN    jeszcze brak 192.168.2.1:18080
Headplane                    pliki są; usługa nie jest uruchomiona
```

Nie wykonano `docker build`, `docker compose up`, restartu, operacji `nft`, zmian
Tailscale/cloudflared ani zmian profilu `DARKSTAR-WiFi`.

## Stan repozytorium

```
31247c2  frontend/public/world — landing
4b6efca  serwowanie /world/ przez darkstar-server
7d67562  przypięte pliki Headplane; bez uruchomienia
26ba7e0  dwa jawne bindy: loopback + adres bramy
30aa527  test granicy loopback-auth dla peer 192.168.2.1
39a6459  odczytowy deploy/network/verify-world
```

Gałąź: `docs/darkstar-headscale-hotspot-plan`.

## `bind_addresses()` — osiem linii źródła

```rust
fn bind_addresses(primary: SocketAddr) -> Vec<SocketAddr> {
    let loopback = SocketAddr::from(([127, 0, 0, 1], primary.port()));
    if primary == loopback {
        return vec![loopback];
    }

    vec![loopback, primary]
}
```

Dla `192.168.2.1:18080` funkcja zwraca kolejno `127.0.0.1:18080` oraz
`192.168.2.1:18080`. Kod odrzuca adres nieokreślony, więc nie używa `0.0.0.0` ani `::`.

## Audyt loopback-auth

Wynik: **PASS w kodzie i teście; live pozostaje do zmierzenia po podmianie obrazu**.

1. Oba listenery przekazują rzeczywisty adres peer przez
   `into_make_service_with_connect_info::<SocketAddr>()`.
2. `allow_loopback` sprawdza `address.ip().is_loopback()`.
3. Token jest dopisywany tylko przy prawdziwym loopback, braku własnego nagłówka
   `Authorization` i skonfigurowanym tokenie.
4. Peer `192.168.2.1` nie jest loopback i bez jawnego tokenu dostaje `401`.
5. Brak `ConnectInfo`, brak tokenu lub błędny własny token pozostają fail-closed.

Świeży test:

```
cargo test -p darkstar-server --test loopback_http
6 passed; 0 failed
```

Test `gateway_address_does_not_receive_the_loopback_token` przypina dokładnie przypadek
`192.168.2.1`. Dodatkowo krok 7 skryptu `verify-world` wymusza ten adres źródłowy przez
`curl --interface 192.168.2.1`, nie wysyła `Authorization` i wymaga `401`. Odpowiedź `200`
jest błędem bezpieczeństwa.

## `verify-world`: osiem odczytowych sprawdzeń

1. Headscale HTTP na `192.168.2.1:8080`.
2. Listener Headscale dokładnie na `192.168.2.1:8080`.
3. Health Darkstar po loopback.
4. Listener Darkstar na `127.0.0.1:18080`.
5. Drugi listener Darkstar na `192.168.2.1:18080`.
6. Landing `http://192.168.2.1:18080/world/`.
7. `401` z chronionego API przez `192.168.2.1` bez tokenu.
8. Brak wildcard listenera na portach 8080 i 18080.

Kontrole lokalne:

```
sh -n deploy/network/verify-world                         PASS
symulacja zdrowego ss/curl                               8/8 PASS
symulacja błędu: chronione API bez tokenu zwraca 200     1/8 FAIL, krok 7
```

Skrypt niczego nie uruchamia, nie zatrzymuje i nie zapisuje na CBMS. Do czasu podmiany
obrazu kroki 5, 6 i 7 mają uczciwie kończyć się `FAIL`, ponieważ listener LAN nie istnieje.

## Bramka zamknięcia

Po ręcznej podmianie obrazu wynik wolno uznać za końcowy tylko wtedy, gdy:

```
ss      -> 127.0.0.1:18080 i 192.168.2.1:18080; brak wildcard
health  -> http://127.0.0.1:18080/health = 200
world   -> http://192.168.2.1:18080/world/ = 200
auth    -> http://192.168.2.1:18080/v1/gateway/status bez tokenu = 401
verify  -> deploy/network/verify-world = 8/8 PASS
```

Jeśli polecenie operatorskie zatrzyma się na `sudo` albo Docker odmówi dostępu, wdrożenia
nie ma. Należy pokazać dokładny błąd operatorowi, bez obchodzenia uprawnień i bez zmiany
sieci hosta.
