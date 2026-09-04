<!-- darkstar-header-v1 -->
<!-- po co: 2026-09-04-world-ingress-phone.md -->
<!-- nie wolno: nft -f, tailscale down/logout na SaaS, rotowac connectora, gasic named cloudflared -->
<!--
Autor: Marcin Szul
Model: Claude Opus 5
Czas: 2026-09-04T01:15:00+01:00
Po co: Zapis pomiaru, ktory pokazal ze droga WARP -> named tunnel -> LAN JUZ DZIALA,
       a jedyna przyczyna ciszy w Safari byl brak numeru portu w adresie.
-->

# Telefon do domu przez WARP — 2026-09-04

Droga stala (WARP -> named tunnel -> 192.168.2.0/24) **jest zestawiona i przenosi ruch**.
Nie brakuje w niej zadnego elementu. Cisza w Safari nie byla awaria tunelu.

## Osiem linii pomiaru

```
1 named tunnel : darkstar 2e8159cc-5e2b-4949-a067-98f31495be46, status healthy,
                 4 polaczenia lhr19/lhr13/lhr13/lhr21, client 2026.8.3, remote_config=true
2 konfiguracja : version=1, config={"warp-routing":{"enabled":true}}, source=cloudflare
3 connector    : "Updated to new configuration config={\"warp-routing\":{\"enabled\":true}}
                 version=1" — 2026-09-03T23:07:26Z. Konfiguracja DOSZLA do connectora.
4 trasa        : 192.168.2.0/24 -> tunnel darkstar, vnet default
                 (c5c02128-cea4-49b8-8d1d-7830b3d119d4), comment "LAN Darkstar", count=1
5 split tunnel : profil "Onboarding Device profile: 04/09/2026", precedence 1000,
                 match identity.email in {jpytka666@gmail.com} — 15 wpisow exclude,
                 192.168.0.0/16 NIE ma na liscie. Profil default nadal go ma, ale
                 NIE obowiazuje tego urzadzenia.
6 PRZYCZYNA    : ERR "unable to dial tcp to origin 192.168.2.1:80: connect: connection
                 refused" ... originService=warp-routing, 2026-09-03T23:46:14Z-23:46:16Z,
                 na wszystkich czterech connIndex. Ruch z telefonu DOSZEDL do CBMS.
                 Docelowy port to 80, nie 8080.
7 porty CBMS   : nasluch na 192.168.2.1 to :53 i :8080. Na :80 NIC.
                 curl 192.168.2.1:8080 -> 200 | curl 192.168.2.1:80 -> 000
8 nft          : NIEZWERYFIKOWANE. `nft list ruleset` wymaga roota, operator zabronil
                 dotykac nft. Zapora i tak jest wykluczona jako przyczyna: "connection
                 refused" pochodzi ze stosu TCP hosta, a nie z odrzucenia w drodze —
                 pakiet dotarl do konca lancucha.
```

## Co to znaczy

Kazde ogniwo dziala: telefon -> WARP -> Cloudflare -> named tunnel -> connector na CBMS
-> siec 192.168.2.0/24. Connector zameldowal proba polaczenia z `192.168.2.1:80` i dostal
odmowe od wlasnego jadra, bo pod tym portem nic nie stoi. Headscale sluchа na `:8080`.

Safari po wpisaniu `192.168.2.1` bez portu idzie na `:80`. Stad "server can't be found".

## Krok, ktory to zamyka

Na telefonie z wlaczonym One Agent otworzyc **z portem**:

```
http://192.168.2.1:8080
```

`hs.darkstar.internal` to osobna sprawa i nie jest tu obiecywana: w koncie nie ma
wewnetrznej strefy DNS, a zapytanie o `dns_settings/views` wraca bledem uwierzytelnienia
(klucz nie ma uprawnienia DNS View). Nazwa zacznie dzialac dopiero po zalozeniu tej strefy.

## Quick tunnel — dowod poboczny, NIE produkcja

```
cloudflared tunnel --url http://192.168.2.1:8080
https://snapshot-registered-ships-rapidly.trycloudflare.com
```
Telefon na 5G otworzyl przez ten adres ekran Headscale `/windows`. To potwierdza, ze
origin `192.168.2.1:8080` jest zdrowy i oddaje tresc na zewnatrz. Adres jest tymczasowy,
bez gwarancji dostepnosci, i nie jest droga docelowa. Nie zostal ubity w tej sesji.

## Zespol i konto

```
Zero Trust team    : weathered-bar-3262
Account            : f82049cc382b9096eb7500108de91971
Urzadzenie         : iPhone enrolled, One Agent Connected
Named connector    : token-file /etc/cloudflared/token, brak config.yml — sterowanie zdalne
Quick connector    : 2fe2f9d6-9567-48d3-a954-7656a372f02a (osobny proces, sesyjny)
```

## Nietkniete

`nft` bez zmian i bez odczytu. Tailscale SaaS bez zmian. Connector named nie zatrzymany
i nie rotowany. Quick tunnel nie ubity. Chroma nie wystawiona na `0.0.0.0`. Port 18080
nie wystawiony na swiat. `config.yml` pod named tunelem nie zalozony. Zadnej domeny
nie kupiono. Zaden nowy unit nie zostal wlaczony.
