<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Opus 5
TIMESTAMP: 2026-09-01 08:10:00
REASON FOR CREATION: Udokumentowanie zastanej, dzialajacej bramy downstream DARKSTAR-WiFi na 192.168.2.0/24 i skutkow, jakie mialby dla niej apply z planu.
==========================================
-->

# Zastana brama downstream — DARKSTAR-WiFi 192.168.2.0/24

Data: 2026-09-01, host CBMS (`owner@100.71.8.70`), boot `dbe6ad68-...`.
Wszystkie polecenia tylko do odczytu. Nic nie zmieniono, `nmcli modify/up/down` nie uzyto,
`darkstar-firewall-apply` nie uruchomiono.

## 1. Co juz stoi

Na gniezdzie kablowym `enp1s0` dziala profil **`DARKSTAR-WiFi`**, i to nie od dzisiaj —
`autoconnect: yes`, wiec wstaje sam po starcie maszyny.

| Ustawienie | Wartosc |
|---|---|
| `connection.id` | `DARKSTAR-WiFi` |
| `connection.uuid` | `95a89b38-ee71-484e-98c2-a6d02e8b92b3` |
| `connection.type` | `802-3-ethernet` na `enp1s0` |
| `connection.autoconnect` | `yes` |
| `ipv4.method` | `shared` |
| `ipv4.addresses` | `192.168.2.1/24` |
| `ipv4.never-default` | `no` |
| `ipv6.method` | `auto` |

Zywe uslugi po tej stronie: DNS na `192.168.2.1:53` i DHCP na `0.0.0.0:67`. Trasa
`192.168.2.0/24 dev enp1s0 metric 100`. Uplink nietkniety: `default via 192.168.1.1 dev wlp2s0`.

Nazwa `DARKSTAR-WiFi` myli — profil jest **przewodowy**, nie bezprzewodowy. Obok stoi nieaktywny
profil `DARKSTAR_WIRED` (`8e5c27b1-...`), ktory prawdopodobnie byl wczesniejszym podejsciem.

## 2. Czy to JEST brama, ktorej chcielismy

**Tak, co do ksztaltu.** Uklad Ubuntu → kabel → urzadzenie po drugiej stronie jest dokladnie ten,
ktory plan opisuje jako pozycje 1 (`wlp2s0` upstream + `enp1s0` downstream). Mechanizm tez ten sam:
`ipv4.method=shared`, czyli NetworkManager sam podaje adres bramy, rozdaje adresy i odpowiada za
nazwy — bez osobnego dnsmasq. Rozni sie **wylacznie adresacja**: `192.168.2.0/24` zamiast
zaplanowanego `10.44.0.0/24`.

Nic w repozytorium tej bramy nie opisuje. Przeszukanie `docs`, `deploy` i `crates` nie znajduje
ani `192.168.2`, ani `DARKSTAR-WiFi` — istnieje wylacznie na hoscie, poza kontrola wersji.

## 3. Czy 10.44 jest zbedne

**Jako nowa siec — tak.** Druga brama na tym samym kablu nie jest potrzebna; `10.44.0.0/24` bylo
wartoscia domyslna z planu, nie wymaganiem. Adresacja `192.168.2.0/24` nie koliduje z niczym na
hoscie (`172.17.0.0/16` docker0, `172.18.0.0/16` br-e0a0946cdec3, `192.168.1.0/24` uplink,
`100.71.8.70/32` mesh).

**Ale `10.44.0.0/24` nie jest martwym zapisem** — jest wpisane w dzialajaca zapore:

~~~text
table inet darkstar_host_guard {
    set darkstar_downstream_ipv4 { elements = { 10.44.0.0/24 } }
    chain forward { policy drop;
        iifname @downstream_ifaces oifname @upstream_ifaces
            ip saddr @darkstar_downstream_ipv4 accept }
    chain postrouting { type nat hook postrouting;
        oifname @upstream_ifaces ip saddr @darkstar_downstream_ipv4 masquerade }
}
~~~

Z odczytu regul wynika, ze **klient z `192.168.2.x` dostanie adres i nazwy, ale jego ruch do
internetu nie ma reguly przepuszczajacej** — lancuch `forward` ma polityke `drop` i akceptuje
wylacznie zrodla z `10.44.0.0/24`, a NAT maskuje tylko te sama pule. To wniosek z tresci regul.
**NIEZWERYFIKOWANE empirycznie** — po drugiej stronie kabla nic sie jeszcze nie odzywalo
(`ip neigh dev enp1s0` puste), wiec nie bylo czym tego przetestowac.

## 4. Czy apply z planu zniszczy dzialajace

**Tak, przejmie interfejs.** Suchy przebieg `DARKSTAR_DRY_RUN=1` pokazal doslownie:

~~~text
DRY_RUN would_run: nmcli connection import type ethernet file /tmp/darkstar-downstream.*.nmconnection
DRY_RUN would_run: nmcli connection up darkstar-downstream
DRY_RUN would_run: /usr/local/sbin/darkstar-firewall-apply
~~~

`nmcli connection up darkstar-downstream` na `enp1s0` zdezaktywuje `DARKSTAR-WiFi`, bo jeden
interfejs trzyma jedno aktywne polaczenie. Profil `DARKSTAR-WiFi` **nie zostalby skasowany** —
zostalby w NetworkManager z `autoconnect: yes` — ale siec `192.168.2.0/24` przestalaby istniec, a
cokolwiek ma z niej adres, straciloby laczosc do czasu odnowienia. Skrypt `darkstar-gateway-stop`
tez by tego nie cofnal: on tylko wylacza `darkstar-downstream`, nie przywraca poprzedniego profilu.

## 5. Decyzja

**ZOSTAW `192.168.2.0/24`. Plan dopasowac do tego, co juz stoi.**

Dzialajaca brama jest warta wiecej niz zgodnosc z domyslna wartoscia z dokumentu. Do wyrownania
zostaja trzy rzeczy, wszystkie po stronie repozytorium, zadna na hoscie:

1. `deploy/firewall/host-guard.nft` — `darkstar_downstream_ipv4` na `192.168.2.0/24`.
2. `deploy/network/darkstar-gateway.env.example` — `DARKSTAR_DOWNSTREAM_CIDR=192.168.2.1/24`,
   `DARKSTAR_DOWNSTREAM_SUBNET=192.168.2.0/24`, `DARKSTAR_CONNECTION_NAME=DARKSTAR-WiFi`.
3. Plan (Task 5 „Interfaces", Step 5.2, Step 5.6, Task 6) — zamiana `10.44.0.1` / `10.44.0.0/24`
   na adresacje faktyczna.

Do czasu punktu 1 brama rozdaje adresy, ale ruch na zewnatrz jest wedlug regul blokowany.
Zmiana zapory jest **zmiana kodu, nie odczytem** — nie wykonano jej w tym przebiegu.

## 6. Czego potrzebuje hub po drugiej stronie kabla

- adres z DHCP wystawianego przez `192.168.2.1` (pula NetworkManager `shared`, zwykle
  `192.168.2.10`–`192.168.2.254`);
- brama domyslna `192.168.2.1`;
- serwer nazw `192.168.2.1`;
- port WAN huba w trybie klienta DHCP, nie w trybie wlasnego routera na tej samej puli;
- **warunek konieczny na dzis:** wpis `192.168.2.0/24` w zaporze, inaczej hub dostanie adres, ale
  nie wyjdzie na zewnatrz.

Zadne haslo nie zostalo zapisane w repozytorium ani w tym pliku.
