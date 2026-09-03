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
