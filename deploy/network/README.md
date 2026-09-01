<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Opus 5
TIMESTAMP: 2026-09-01 05:44:48
REASON FOR CREATION: Instrukcja uruchomienia tylko-do-odczytu preflightu sieciowego Darkstar (Task 4).
==========================================
-->

# Darkstar network preflight

Rozpoznanie sprzetu przed wyborem topologii downstream. Skrypt **niczego nie zmienia**:
nie aktywuje, nie rozlacza i nie modyfikuje polaczen, nie dotyka firewalla, nie instaluje pakietow.

## Uruchomienie

~~~bash
./deploy/network/darkstar-network-preflight
~~~

Nadpisanie domyslnych interfejsow:

~~~bash
DARKSTAR_UPSTREAM_IFACE=wlp2s0 DARKSTAR_DOWNSTREAM_IFACE=enp1s0 \
  ./deploy/network/darkstar-network-preflight
~~~

## Wyjscie

Sekcje `# --- nazwa` oraz linie `klucz=wartosc`. Brakujace narzedzie lub brak uprawnien daje
`unavailable:...` zamiast bledu — skrypt konczy sie kodem 0 nawet na niekompletnym hoscie,
zeby dowod dalo sie zebrac w jednym przebiegu.

Zbierane fakty: wersje NetworkManager/nft/systemd/ip/iw, sterownik i firmware `wlp2s0`,
zdolnosc AP, stan i carrier `enp1s0`, trasy domyslne IPv4/IPv6, aktywne profile
NetworkManager, wartosci forwardingu, obecnosc tabeli `inet darkstar_host_guard`.

## Kontrola jakosci

~~~bash
sh -n deploy/network/darkstar-network-preflight
shellcheck deploy/network/darkstar-network-preflight
~~~

## Zakres

Odpowiada za Step 4.2 i Step 4.3 planu
`docs/superpowers/plans/2026-08-29-darkstar-native-gateway-headscale-headplane-plan.md`.
Wynik uruchomienia na hoscie nalezy zapisac w
`docs/operations/evidence/2026-08-29-network-preflight.md` razem z decyzja ze Step 4.6.

Uruchomienie na hoscie jest zablokowane do czasu udowodnienia trwalosci firewalla po
prawdziwym reboocie (Task 3), ktory wymaga fizycznego dostepu do maszyny CBMS.
