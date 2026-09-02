<!-- darkstar-header-v1 -->
<!-- po co: 2026-09-03-windows-joined-headscale.md -->
<!-- nie wolno: tailscale up/logout na CBMS, klucz w repozytorium, nft poza ich skryptem -->
<!--
Autor: Marcin Szul
Model: Claude
Czas: 2026-09-03T00:20:00+01:00
Po co: Zapis pierwszego wezla, ktory odpowiedzial na wlasnym serwerze sterujacym Headscale,
       przy zachowaniu Tailscale SaaS jako drogi ratunkowej na CBMS.
-->

# Windows dolaczyl do wlasnego mesh — 2026-09-03

Pierwszy wezel odpowiada na **wlasnym** serwerze sterujacym. CBMS zostal na SaaS i jest
nadal lina ratunkowa.

## Osiem linii pomiaru

```
1 wezel w headscale     : desktop-udi6m9f, ID 1, user darkstar, online
2 adres w mesh          : 100.64.0.1  oraz  fd7a:115c:a1e0::1
3 ControlURL na Windows : http://192.168.2.1:8080     <- wlasny serwer, nie SaaS
4 tailscale status      : 100.64.0.1 desktop-udi6m9f darkstar windows
5 ping 192.168.2.1      : True
6 ping 8.8.8.8          : True                        <- internet caly
7 health :8080 z Windows: 200
8 CBMS                  : LAN_OK, hostname CBMS; tailscale nadal na SaaS
                          (100.71.8.70 darkstar-cbms, idle, offers exit node)
```

## Co sie zmienilo w drogach dostepu

Windows opuscil siec SaaS, wiec adres `100.71.8.70` **przestal byc z niego osiagalny**.
Cala dalsza praca na CBMS idzie przez kabel: `owner@192.168.2.1`.

Ta droga byla sprawdzona **wykonaniem, przed joinem, a nie po fakcie** — `echo LAN_OK`
wrocilo poprawnie i klucz hosta zostal zapamietany, wiec polaczenie po zerwaniu chmury
nie zatrzymalo sie na pytaniu o potwierdzenie. To byl warunek, bez ktorego join nie mial
prawa ruszyc.

W widoku SaaS Windows figuruje teraz jako `offline, last seen 1m ago`. To jest poprawne:
wezel nie zostal usuniety z tamtej sieci, tylko przestal sie do niej zglaszac.

## Czego nie zrobiono

`tailscale up` ani `logout` **nie padly na CBMS** i nie padna, dopoki wlasny mesh nie
obsluzy obu wezlow. CBMS trzyma polaczenie do SaaS jako jedyna zdalna droge na wypadek,
gdyby kabel do `192.168.2.1` przestal wystarczac.

Klucza preauth **nie ma w repozytorium** i miec nie moze. Zostal pokazany operatorowi raz;
jest wielokrotnego uzytku i wygasa po 24 godzinach.

## Warunek wyjscia — polowa

Warunek brzmial: SaaS zostaje, dopoki **dwa** wezly nie odpowiedza na nowym serwerze.
Odpowiada **jeden**. Do domkniecia brakuje CBMS, a to wymaga osobnego stanu `tailscaled`
— wlasnej jednostki systemd, wlasnego `--statedir` i wlasnego urzadzenia `tun` — czyli
uprawnien roota. Przelaczenie biezacego demona jest wykluczone: zerwaloby jedyna zdalna
droge do hosta.

## Nietkniete

Siec domowa poza jedna swiadomie przeladowana regula zapory (port 8080 z `192.168.2.0/24`,
wykonana przez operatora jego wlasnym skryptem `darkstar-firewall-apply`), dysk E, pociagi
w Sterowni. Zadna nowa siec dockerowa nie powstala. `0.0.0.0` nie zostalo uzyte jako adres
nasluchu.
