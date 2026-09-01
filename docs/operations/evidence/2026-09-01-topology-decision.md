<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Opus 5
TIMESTAMP: 2026-09-01 07:35:00
REASON FOR CREATION: Decyzja o topologii downstream Darkstar na podstawie odczytow z CBMS (Task 4, Step 4.4-4.6).
==========================================
-->

# Darkstar — decyzja o topologii downstream

Data: 2026-09-01, host CBMS (`owner@100.71.8.70`), boot `dbe6ad68-eff2-4954-baa4-66c7077d1c8c`.
Zrodla: `2026-09-01-network-preflight.md` oraz odczyt `ip -4 addr` / `ip -4 route` / `iw list`.
Wszystko tylko do odczytu. Zadna zmiana na hoscie, brama nie uruchamiana.

## 1. Uplink — Wi-Fi, potwierdzony

`wlp2s0` (iwlwifi, firmware `36.c8e8e144.0`) jest **jedyna droga do internetu**:

~~~text
inet 192.168.1.48/24 brd 192.168.1.255 scope global noprefixroute wlp2s0
default via 192.168.1.1 dev wlp2s0 proto dhcp src 192.168.1.48 metric 600
~~~

Droga zarzadzania `tailscale0` = `100.71.8.70/32` biegnie po tym samym uplinku.

## 2. Ethernet downstream — ZABLOKOWANY, brak kabla

`enp1s0` nie pojawia sie w `ip -4 addr` (brak adresu). Preflight z tego samego dnia:

~~~text
downstream_operstate=down
downstream_carrier=0
downstream_link_detected=no
~~~

Interfejs istnieje, ale nie ma sygnalu na kablu. Step 4.5 planu wymaga potwierdzonego carrier
przed wyborem baseline, wiec **baseline ethernet jest zablokowany na brakujacym kablu**, nie na
brakujacym sprzecie ani na bledzie konfiguracji.

## 3. Hotspot / wspolbieznosc AP+STA — NIE RUSZAC

`iw list` zglasza dwie grupy kombinacji:

~~~text
* #{ managed } <= 1, #{ P2P-client, P2P-GO } <= 1, #{ P2P-device } <= 1,
  total <= 3, #channels <= 2
* #{ managed } <= 1, #{ AP, P2P-client, P2P-GO } <= 1, #{ P2P-device } <= 1,
  total <= 3, #channels <= 1
~~~

Grupa zawierajaca `AP` ma **`#channels <= 1`**. AP i klient musialyby dzielic jeden kanal, wiec
hotspot bylby przywiazany do kanalu narzuconego przez siec nadrzedna, a jego zmiana zrywalaby
oba polaczenia naraz — w tym `tailscale0`, czyli jedyna droge zarzadzania. Plan dopuszcza
same-radio AP+STA wylacznie po testach zdolnosci i stabilnosci (Step 4.4, Step 4.6 pozycja 3).
**Decyzja: nie ruszac.** Tryb hotspot pozostaje niewlaczony.

## 4. Kolizja z sieciami Dockera — NIE

| Siec | Adresacja | Kolizja z `10.44.0.0/24` |
|---|---|---|
| `docker0` | `172.17.0.0/16` (state DOWN, NO-CARRIER) | **nie** |
| `br-e0a0946cdec3` | `172.18.0.0/16` (state UP) | **nie** |
| uplink `wlp2s0` | `192.168.1.0/24` | **nie** |
| mesh `tailscale0` | `100.71.8.70/32` | **nie** |

`DARKSTAR_DOWNSTREAM_CIDR=10.44.0.1/24` i `DARKSTAR_DOWNSTREAM_SUBNET=10.44.0.0/24` z
`deploy/network/darkstar-gateway.env.example` sa **wolne** — zadna aktywna siec na hoscie nie
zachodzi na `10.44.0.0/24`. Kontraktu nie trzeba zmieniac.

## 5. Decyzja

**CZEKAC NA KABEL.**

Wybor baseline pozostaje nierozstrzygniety do czasu wpiecia kabla Ethernet miedzy `enp1s0` na
CBMS a portem Ethernet na Windows i powtorzenia preflightu. Preferencja z planu (pozycja 1:
`wlp2s0` upstream + `enp1s0` downstream) jest sprzetowo mozliwa i wolna od kolizji adresowych —
brakuje wylacznie fizycznego polaczenia.

Nastepny krok po wpieciu kabla: powtorzyc `deploy/network/darkstar-network-preflight` i
potwierdzic `downstream_carrier=1` oraz `Link detected: yes`. Dopiero wtedy Step 4.6 ma
przeslanki do zamkniecia.

Zadne haslo nie zostalo zapisane. `darkstar-gateway-apply` nie byl wywolany w zadnym trybie.
