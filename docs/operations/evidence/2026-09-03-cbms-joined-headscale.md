<!-- darkstar-header-v1 -->
<!-- po co: 2026-09-03-cbms-joined-headscale.md -->
<!-- nie wolno: tailscale down/logout na SaaS, nft -f, gasic tailscaled.service -->
<!--
Autor: Marcin Szul
Model: Claude
Czas: 2026-09-03T06:00:00+01:00
Po co: Zapis chwili, w ktorej oba wezly odpowiadaja na wlasnym serwerze sterujacym, oraz
       jednej przeszkody, przez ktora prywatny mesh NIE moze jeszcze zastapic SaaS.
-->

# CBMS dolaczyl do wlasnego mesh — 2026-09-03

Oba wezly sa w prywatnej sieci. Warunek wyjscia, postawiony na poczatku tej pracy -
"SaaS zostaje, dopoki dwa wezly nie odpowiedza na NOWYM serwerze" - jest **spelniony**.

## Osiem linii pomiaru

```
1 wezly w headscale   : 2 - desktop-udi6m9f 100.64.0.1, darkstar-cbms 100.64.0.2, oba online
2 demon mesh          : tailscaled-headscale.service  active / enabled
3 demon SaaS          : tailscaled.service            active  <- NIETKNIETY
4 gniazda             : /run/tailscale/... oraz /run/tailscale-headscale/... - osobne
5 ping 192.168.2.1    : OK   (z CBMS i z Windows)
6 ping 8.8.8.8        : OK   <- internet caly
7 pamiec              : {"local_cbms_ok":true,"remote_e_ok":true}
8 mesh z Windows      : ping 100.64.0.2 OK, ale SSH 100.64.0.2:22 ODRZUCONE
```

## Dwa demony obok siebie, nie zamiast siebie

```
SaaS  : /usr/sbin/tailscaled --state=/var/lib/tailscale/... --socket=/run/tailscale/...
        --port=41641      widzi: 100.71.8.70 darkstar-cbms, 100.96.213.103 desktop (offline)
mesh  : --statedir=/var/lib/tailscale-headscale --socket=/run/tailscale-headscale/...
        --tun=headscale0 --port=41642
        widzi: 100.64.0.2 darkstar-cbms, 100.64.0.1 desktop-udi6m9f
```

Cztery rozdzielone zasoby - stan, gniazdo, urzadzenie, port - zadzialaly dokladnie tak,
jak mialy. Zaden z demonow nie wie o drugim i awaria jednego nie rusza drugiego.
`tailscale down` ani `logout` nie padly na zadnym.

Wezel Windows figuruje w widoku SaaS jako `offline, last seen 4h ago`. To poprawne: nie
zostal usuniety, tylko przestal sie tam meldowac, bo jego serwer sterujacy to teraz
`http://192.168.2.1:8080`.

## PRZESZKODA: mesh niesie pakiety, ale zapora nie ufa nowemu interfejsowi

Zmierzone z Windows `192.168.2.50`:

```
ping 100.64.0.2      : True     <- pakiety PLYNA przez prywatny mesh
SSH  100.64.0.2:22   : False    <- odrzucone
```

Przyczyna jest w wersjonowanym `host-guard.nft` i jest jednoznaczna:

```
set management_ifaces {
    elements = { "tailscale0" }          <- tylko STARY interfejs
}
iifname @management_ifaces accept comment "temporary Tailscale management"
ip protocol icmp accept comment "ICMPv4"
```

Nowy interfejs nazywa sie `headscale0` i ma adres `100.64.0.2/32`, ale **nie ma go w
zbiorze zaufanych**. Przechodzi wiec wylacznie ICMP, dopuszczony osobna, ogolna regula -
stad ping dziala, a TCP nie.

To jest dokladnie ta klasa bledu, ktora latwo przeoczyc: sieciowo wszystko wyglada
poprawnie, wezly widza sie nawzajem, a mimo to nie da sie po niej pracowac.

## Dlaczego SaaS ma NA RAZIE zostac, mimo spelnionego warunku

Warunek liczbowy jest spelniony - dwa wezly odpowiadaja. Ale prywatny mesh **nie moze
jeszcze zastapic SaaS jako droga zarzadzania**, bo po nim nie przechodzi SSH. Wylaczenie
`tailscaled.service` teraz zostawiloby jako jedyne wejscie kabel `192.168.2.1`.

Domkniecie wymaga dopisania `headscale0` do `management_ifaces` w `host-guard.nft` i
swiadomego przeladowania. Regula **nie zostala dopisana** - w tej rundzie zakres byl
wylacznie dokumentacyjny, a zapory nie rusza sie bez wyraznego polecenia.

## Nietkniete

`tailscale down`, `tailscale logout`, `tailscaled.service`, `nft -f`, dysk E, pociagi
w Sterowni. Klucza preauth nie ma w repozytorium. Pamiec dziala na obu nogach przez caly
czas trwania tej operacji.

---

# Pomiar domkniety: mesh niesie ruch w OBIE strony — 2026-09-03

Poprzednia sekcja mierzyla wylacznie kierunek Windows -> CBMS. Brakowal kierunek odwrotny,
a bez niego nie dalo sie odroznic "mesh dziala" od "jedna strona cos widzi".

## Osiem linii

```
1 wezly              : 2 online - 100.64.0.1, 100.64.0.2
2 100.64.0.2:22      : TcpTestSucceeded False   <- SSH po mesh ODRZUCONE
3 ping 100.64.0.2    : True   (Windows -> CBMS)
4 ping 100.64.0.1    : OK     (CBMS -> Windows)   <- brakujacy kierunek, dziala
5 SaaS               : tailscaled.service active; mesh tailscaled-headscale active
6 pamiec 8000/8001   : 200/200, {"local_cbms_ok":true,"remote_e_ok":true}
7 udp 41642 w /etc   : 1
8 siec               : ping 192.168.2.1 OK, ping 8.8.8.8 OK, Sterownia :18080 -> 200
```

## Co ten brakujacy kierunek rozstrzyga

`ping` przechodzi **z obu stron**. Znaczy to, ze prywatny mesh naprawde przenosi pakiety
miedzy wezlami - to nie jest sama rejestracja w bazie serwera sterujacego, tylko dzialajaca
sciezka sieciowa. Klucze zostaly wymienione, tunel stoi, trasy sa.

Jednoczesnie SSH po tym samym adresie nadal odbija. To domyka diagnoze i wyklucza wszystkie
inne podejrzenia: nie jest to problem tras, kluczy ani serwera sterujacego. Zostaje
dokladnie jedna przyczyna - `management_ifaces` w `host-guard.nft` zawiera tylko
`tailscale0`, a ruch przychodzi na `headscale0`. Przechodzi ICMP, dopuszczony osobna,
ogolna regula; TCP nie.

## Sterownia dziala przez STARA droge

`http://127.0.0.1:18080/health` odpowiada `200`. Ten tunel idzie przez `tailscaled.service`,
czyli przez SaaS - nie przez prywatny mesh. To kolejny powod, dla ktorego SaaS zostaje:
wylaczenie go zerwaloby takze podglad Sterowni, nie tylko SSH.

## Stan koncowy

Prywatny mesh **dziala i przenosi ruch**, ale **nie jest jeszcze droga zarzadzania**.
Do tego brakuje jednej linii w zaporze - dopisania `headscale0` do zaufanych interfejsow.
Nie zostala dopisana; zakres tej rundy byl pomiarowy i dokumentacyjny.

---

# Poprawka SSH po mesh — regula w PLIKU, niezastosowana

## Co dodano

```
iifname "headscale0" tcp dport 22 accept comment "SSH over private Headscale mesh"
```

Jedna linia, wstawiona obok istniejacej reguly SSH z sieci prywatnej.

## Dlaczego NIE dopisano headscale0 do management_ifaces

Byla to druga z dwoch mozliwosci i zostala odrzucona swiadomie. Istniejaca regula brzmi:

```
iifname @management_ifaces accept comment "temporary Tailscale management"
```

To jest `accept` **bez zadnego ograniczenia portu** - wszystko, co przyjdzie z interfejsu
w tym zbiorze, wchodzi. Dopisanie tam `headscale0` naprawiloby SSH, ale przy okazji
otworzyloby cala maszyne dla kazdego portu z nowej sciezki, jako **skutek uboczny**
naprawy jednej rzeczy.

Nowa regula daje dokladnie to, o co chodzi - zdalne zarzadzanie - i nic ponad to.
Rozszerzenie zakresu, gdyby kiedys bylo potrzebne, ma byc osobna i widoczna decyzja.

## Stan regul wejsciowych po zmianie

```
44 loopback
45 established,related
48 ICMPv4
49 ICMPv6
51 @management_ifaces accept                    <- stary Tailscale, bez limitu portu
52 @upstream_ifaces udp 41641                    <- transport SaaS
54 @upstream_ifaces DHCP v4
55 @upstream_ifaces DHCP v6
57 @downstream_ifaces + 192.168.2.0/24  tcp 22
72 headscale0 tcp 22                             <- NOWA, tylko SSH
84 @downstream_ifaces + 192.168.2.0/24  tcp 8080
98 @downstream_ifaces + 192.168.2.0/24  udp 41642
```

Nowa regula nie dotyka `wlp2s0` ani sieci nadrzednej Vodafone. `0.0.0.0` nie wystepuje.

## Czego NIE zrobiono

`nft -f` nie padl, `darkstar-firewall.service` nie zostal przeladowany. Regula lezy w
wersjonowanym pliku i do czasu swiadomego przeladowania przez operatora **SSH po mesh
nadal bedzie odbijac**. Tak ma byc - stan `/etc` i stan repozytorium sa tu celowo
rozdzielone.

Linia dla operatora:

```
ssh owner@192.168.2.1 'sudo cp ~/polip-agi/deploy/firewall/host-guard.nft /etc/darkstar/host-guard.nft && sudo ~/polip-agi/deploy/firewall/darkstar-firewall-apply && grep -c headscale0 /etc/darkstar/host-guard.nft'
```

Ma zwrocic `loaded inet darkstar_host_guard` oraz `1`. Potem, z Windows:

```
Test-NetConnection 100.64.0.2 -Port 22       # ma dac TcpTestSucceeded True
```

SaaS zostaje `active` przez caly czas tej operacji.

---

# Stan reguly SSH po mesh — dwie linie

```
regula w PLIKU (deploy/firewall/host-guard.nft) : JEST
regula w /etc/darkstar/host-guard.nft           : 0   -> NIEZASTOSOWANA
```

`nft -f` nie padl i `darkstar-firewall.service` nie byl przeladowany. Do czasu swiadomego
przeladowania przez operatora SSH po adresie mesh nadal odbija sie od polityki `drop`.
