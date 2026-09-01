<!-- darkstar-header-v1 -->
<!-- po co: 2026-09-01-ee-hub-ap-static-windows-online.md -->
<!-- nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie -->
<!-- autor: Marcin -->
<!-- powstal: 2026-09-01 -->
<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Opus 5
TIMESTAMP: 2026-09-01 19:45:00
REASON FOR CREATION: Zapis dzialajacej topologii Darkstar - hub EE jako punkt dostepowy, Windows online przez brame Ubuntu ze statycznym adresem. Sluzy takze jako backup ustawien do odtworzenia.
==========================================
-->

# Darkstar — hub EE jako punkt dostepowy, Windows online

Data: 2026-09-01. Host CBMS (`owner@100.71.8.70`). Odczyt na zywo, zero zmian.

## Topologia, ktora dziala

~~~text
Internet
   |
   | Vodafone, 192.168.1.1
   |
[wlp2s0]  192.168.1.48/24        <- uplink, profil "Vodafone480B38 1"
   |
CBMS (Ubuntu, brama + NAT + zapora darkstar_host_guard)
   |
[enp1s0]  192.168.2.1/24 shared  <- profil "DARKSTAR-WiFi", dnsmasq 192.168.2.10-254
   |
   | kabel do gniazda LAN (zoltego) huba EE
   |
Hub EE   192.168.2.2, DHCP WYLACZONY, tryb punktu dostepowego, SSID EE-57GMTG
   |
   ) Wi-Fi (
   |
Windows  192.168.2.50/24 STATYCZNIE, brama 192.168.2.1, DNS 8.8.8.8
~~~

## Potwierdzenia z hosta

| Pomiar | Wynik |
|---|---|
| `ping 192.168.2.2` | 1/1, 0% loss, rtt 0.604 ms |
| `ip neigh dev enp1s0` | `192.168.2.2 lladdr a0:2d:db:d7:a9:20 DELAY` — hub odpowiada |
| `ip neigh dev enp1s0` | `192.168.2.50 lladdr d4:25:8b:d3:79:5d REACHABLE` — **Windows widoczny przez hub** |
| `default` | `via 192.168.1.1 dev wlp2s0` — wyjscie nadal uplinkiem |
| profile aktywne | `DARKSTAR-WiFi:enp1s0`, `Vodafone480B38 1:wlp2s0`, `tailscale0` |

Ze strony Windows potwierdzone: `ping 192.168.2.1` OK, `ping 192.168.2.2` OK, `ping 8.8.8.8` OK,
`google.com` otwiera sie. **NAT przez Darkstar dziala.**

## Ograniczenie: adres nadawany automatycznie NIE przechodzi przez Wi-Fi huba

Klient bezprzewodowy nie dostaje adresu z `dnsmasq` na `enp1s0` — dostawal `169.254.x.x`, czyli
adres awaryjny. Most na poziomie ramek dziala **wylacznie z adresem ustawionym recznie**.

Skutek praktyczny: kazdy nowy klient bezprzewodowy wymaga recznego ustawienia. Do rozstrzygniecia
osobno; nie blokuje obecnej pracy.

## Backup ustawien — do odtworzenia

**Windows, karta bezprzewodowa „Wi-Fi 2"** (siec `EE-57GMTG`):

~~~text
IPv4        192.168.2.50
maska       255.255.255.0
brama       192.168.2.1
DNS         8.8.8.8
~~~

**Hub EE:** adres `192.168.2.2`, serwer adresow **wylaczony**, tryb punktu dostepowego,
SSID `EE-57GMTG`, kabel w gniezdzie **LAN** (zoltym), nie w gniezdzie zewnetrznym.
Sprzet `a0:2d:db:d7:a9:20`, `SerialNumber ":114404:2525118982"`.

**Ubuntu, profil `DARKSTAR-WiFi`** (`95a89b38-ee71-484e-98c2-a6d02e8b92b3`):

~~~text
connection.type            802-3-ethernet     na enp1s0
connection.autoconnect     yes
ipv4.method                shared
ipv4.addresses             192.168.2.1/24
ipv6.method                disabled
~~~

**Zapora** `deploy/firewall/host-guard.nft`: `darkstar_downstream_ipv4 = { 192.168.2.0/24 }`,
instalacja `/etc/darkstar/host-guard.nft`, jednostka `darkstar-firewall.service` `enabled`.
Kopia poprzedniej wersji: `/var/backups/darkstar-firewall/host-guard.nft.20260901`.

## Uwaga — pozostalosc do sprzatniecia

Na `enp1s0` **znow widnieje** adres `192.168.1.253/24` i trasa `192.168.1.254 dev enp1s0`, mimo ze
oba zostaly usuniete i potwierdzone jako usuniete (`2026-09-01-ee-hub-appinitscreen-dead.md`).
Wrocily poza ta sesja — **przyczyna nieustalona**, nie zgadywana. Nie szkodza: caly ruch do
`192.168.1.0/24` nadal idzie przez `wlp2s0`, a trasa dotyczy jednego adresu. Do zdjecia przy
najblizszej okazji:

~~~bash
sudo ip route del 192.168.1.254/32 dev enp1s0
sudo ip addr del 192.168.1.253/24 dev enp1s0
~~~

Zadne haslo nie zostalo zapisane w tym pliku ani w repozytorium.
