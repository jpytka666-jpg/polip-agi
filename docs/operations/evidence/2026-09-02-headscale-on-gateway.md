<!-- darkstar-header-v1 -->
<!-- po co: 2026-09-02-headscale-on-gateway.md -->
<!-- nie wolno: nft -f, restart darkstar-firewall.service, tailscale up, tailscale logout -->
<!--
Autor: Marcin Szul
Model: Claude
Czas: 2026-09-02T23:25:00+01:00
Po co: Zapis stanu po przeniesieniu nasluchu Headscale na brame prywatna 192.168.2.1
       oraz reguly zapory dopisanej do pliku, ale swiadomie NIEZASTOSOWANEJ.
-->

# Headscale na bramie prywatnej — 2026-09-02

Headscale slucha na `192.168.2.1:8080`. Darkstar czyta go pod tym samym adresem i
odpowiada 200. Tailscale nietkniety.

## Pomiar

```
3 darkstar /v1/headscale/health : 200
4 tresc                          : {"baseUrl":"http://192.168.2.1:8080",
                                    "health":"pass","nodeCount":null,"nodes":null,
                                    "nodesReason":"no_api_key","reachable":true}
5 nasluch 8080                   : 192.168.2.1:8080
6 sshd                           : 0.0.0.0:22  [::]:22
7 tailscale ControlURL           : https://controlplane.tailscale.com
8 tailscale wezly                : 2, headscale: running, darkstar: healthy
```

Petla zwrotna na porcie 8080 przestala odpowiadac i jest to skutek zamierzony: Headscale
przyjmuje **jeden** adres nasluchu. Darkstar zostal przestawiony przez
`DARKSTAR_HEADSCALE_URL`; przy sieci hosta to ta sama maszyna i ruch jej nie opuszcza.

## sshd — zapasowa droga

`sshd` nasluchuje na `0.0.0.0:22`, co samo w sobie wyglada szeroko. Zwezenie robi zapora:
z sieci nadrzednej Vodafone przechodzi wylacznie transport WireGuard i klient DHCP, wiec
SSH z `wlp2s0` **nie wchodzi**. Zmierzone z Windows `192.168.2.50`:

```
192.168.2.1:22   -> TcpTestSucceeded: True
192.168.2.1:8080 -> TcpTestSucceeded: False
```

Droga zapasowa na wypadek padu SaaS jest sprawna.

## Regula zapory — dopisana do pliku, NIEZASTOSOWANA

Przed zmiana porto 8080 **nie wystepowalo w `host-guard.nft` w ogole**. Dopisano jedna
linie, dokladnie w ksztalcie sasiedniej reguly SSH:

```
iifname @downstream_ifaces ip saddr @darkstar_downstream_ipv4 tcp dport 8080 accept \
    comment "downstream Headscale control server, Task 13"
```

Zakres jest ten sam co u SSH: wylacznie interfejs downstream i wylacznie zrodla z
`@darkstar_downstream_ipv4`, czyli `192.168.2.0/24`. Sieci nadrzednej to nie dotyczy.

**W tej sesji nie wykonano ani `nft -f`, ani restartu `darkstar-firewall.service`.**
Regula lezy w wersjonowanym pliku i czeka na swiadome przeladowanie przez operatora —
zgodnie z `deploy/firewall/README.md`, ktory mowi, ze zmiany zapory ida wylacznie przez
ten plik, nigdy recznie na hoscie. Dopoki przeladowania nie ma, port 8080 pozostaje z
sieci prywatnej nieosiagalny i **zaden wezel nie dolaczy**.

## Komendy JOIN — zapisane, NIEWYKONANE

Zadna nie zostala uruchomiona. Klucz podstawia operator; tutaj nie jest zapisany.

Ubuntu / CBMS:

```
tailscale up --login-server=http://192.168.2.1:8080 --authkey=<klucz>
```

**Uwaga do powyzszej.** Uruchomiona na biezacym demonie CBMS zerwie `100.71.8.70`, czyli
jedyna zdalna droge do hosta. Bezpiecznie tylko z osobnym stanem — wlasna jednostka
systemd, wlasny `--statedir`, wlasne urzadzenie `tun` — co wymaga roota.

Windows:

```
tailscale up --login-server=http://192.168.2.1:8080 --authkey=<klucz>
```

Weryfikacja po dolaczeniu:

```
docker exec darkstar-headscale headscale nodes list
```

## Warunek wyjscia — nadal NIE spelniony

SaaS zostaje, dopoki **dwa** wezly nie odpowiedza na nowym serwerze. Odpowiada zero.
`tailscale up`, `tailscale down` i `tailscale logout` nie padly w tej sesji i nie padna,
dopoki ten warunek nie zostanie spelniony.

## Nietkniete

Siec domowa, `nft` (plik zmieniony, reguly NIE przeladowane), dysk E, pociagi w Sterowni.
Zadna nowa siec dockerowa nie powstala. `0.0.0.0` nigdzie nie zostalo uzyte jako adres
nasluchu Headscale ani Darkstara.
