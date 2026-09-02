<!-- darkstar-header-v1 -->
<!-- po co: 2026-09-02-headscale-health.md -->
<!-- nie wolno: gasic tailscaled, tworzyc sieci dockerowych, wiazac na 0.0.0.0 -->
<!--
Autor: Marcin Szul
Model: Claude
Czas: 2026-09-02T23:05:00+01:00
Po co: Zapis pomiaru, ze wlasny serwer mesh Headscale dziala obok dzialajacego Tailscale,
       na petli zwrotnej, bez dolozenia czegokolwiek do sieci hosta.
-->

# Headscale obok Tailscale — 2026-09-02

Wlasny serwer sterujacy prywatnego mesh stoi i odpowiada. Tailscale **nie zostal ruszony**
i nadal jest jedyna droga ratunkowa do hosta.

## Osiem linii pomiaru

```
1 headscale /health        : 200
2 darkstar->headscale      : 200
3 tresc                    : {"baseUrl":"http://127.0.0.1:8080","health":"pass",
                              "nodeCount":null,"nodes":null,
                              "nodesReason":"no_api_key","reachable":true}
4 wezel na grafie          : Headscale (18 wezlow / 16 krawedzi)
5 brama bez naglowka       : 200
6 nasluch headscale        : 1 na petli, poza petla: 0
7 sieci docker             : bridge deploy_default host none
8 tailscale                : 2 wezly, darkstar: healthy, headscale: running
```

Linia 6 i 7 sa tu sednem. Headscale nasluchuje **wylacznie** na `127.0.0.1:8080`, a lista
sieci dockerowych jest dokladnie taka sama jak przed wdrozeniem — **zadna nowa siec nie
powstala**. Zadnej reguly `nft` nie dotknieto.

## Czego nie zrobiono i dlaczego

| Krok z planu | Co zrobiono zamiast | Powod |
|---|---|---|
| `docker network create darkstar` | nic — `network_mode: host` | Ten host jest brama domowa. Wlasna siec dockerowa dokladalaby mostek i reguly przekierowania do zywej maszyny routujacej. |
| montaz `/var/lib/darkstar`, `/run/darkstar` | zwykle wolumeny dockerowe | Zalozenie tych katalogow wymaga roota. Dodatkowo `/run` to tmpfs — katalog gniazd znikalby przy kazdym restarcie. |
| `docker pull --network=host` | zwykly `docker pull` | `docker pull` nie zna flagi `--network`. Obraz `v0.29.3` i tak byl juz pobrany. |
| `listen_addr: 0.0.0.0:8080` z przykladu | `127.0.0.1:8080` | Przy sieci hosta `0.0.0.0` wystawiloby usluge takze na `wlp2s0`, czyli do sieci nadrzednej Vodafone. |

## Konfiguracja — cztery poprawki podyktowane przez sam Headscale

Zaden z tych bledow nie zostal zgadniety. Kazdy jest cytatem z odmowy startu:

```
"headscale now requires a new noise.private_key_path field"
"dns.nameservers.global must be set when dns.override_local_dns is true"
"no IPv4 or IPv6 prefix configured, minimum one prefix is required"
"invalid database type \"\", must be sqlite, sqlite3 or postgres"
"dns.base_domain must be set when using MagicDNS (dns.magic_dns)"
```

Przelozenie na zmiany w `config.yaml`:

| Stary klucz z przykladu | Nowy, wymagany przez 0.29 |
|---|---|
| `ip_prefixes:` (lista) | `prefixes.v4` / `prefixes.v6` |
| `db_type`, `db_path` | `database.type`, `database.sqlite.path` |
| `dns.base_domains:` (lista) | `dns.base_domain`, `magic_dns: false` |
| brak | `noise.private_key_path` |

Przejmowanie DNS-u wylaczono (`override_local_dns: false`) zamiast podawac serwery nazw.
Przejecie zmienialoby rozwiazywanie nazw na maszynach w domu — to dokladnie ta klasa
zmiany, ktorej ten host ma nie robic.

Do tego brakowalo polecenia startowego: obraz uruchamial `headscale` bez podpolecenia,
wypisywal pomoc i konczyl sie natychmiast. Dodano `command: ["serve"]`.

## Weryfikacja ich wlasnym skryptem

`deploy/headscale/verify`:

```
PASS: Container darkstar-headscale is running
PASS: HTTP API reachable on 127.0.0.1:8080
PASS: Headscale version: headscale version v0.29.3
PASS: Listeners bound to private addresses only
PASS: No connection to Tailscale SaaS
```

Ostrzezenia dotyczace bazy i pliku polityki pochodza z tego, ze obraz Headscale nie
zawiera `sh` ani `test`, wiec skrypt nie ma czym zajrzec do srodka. To ograniczenie
narzedzia, nie objaw awarii — serwer odpowiada i zna swoja wersje.

## Lista wezlow — stan uczciwy, nie udawany

`nodesReason: "no_api_key"` znaczy dokladnie tyle: klucza API nie ma, wiec listy wezlow
**nie odczytano**. To nie to samo co pusta lista. Sciezka zwraca `null` z podanym powodem
zamiast zera, bo "nie wiem" i "wiem, ze zero" to dwa rozne stany.

Klucz API wymaga naglowka w zapytaniu, a klient HTTP, ktory juz stoi w warstwie kontekstu,
naglowkow nie wysyla. Swiadomie **nie powstal drugi klient** — ta czesc jest niedokonczona
i tak jest zaraportowana.

## Testy

```
cargo test --workspace              143 passed, 0 failed
  w tym headscale_http                5 passed  (zdrowie, brak klucza, 503, 401, 405)
  w tym loopback_http                 5 passed
npm run test:pin                      5 passed, 0 failed
```

## Nietkniete

`tailscaled` dziala, oba wezly widoczne. Sieci domowej, `nft`, dysku E i pociagow w
Sterowni nie dotknieto. Nasluch Darkstara nadal wylacznie na petli zwrotnej.

---

# Serwer sterujacy zyje — uzytkownik i klucz, 2026-09-02

Bramka zdrowia przeszla (`health: 200`), wiec wykonano dwa kroki zapisu **w samym
Headscale**. Poza jego wlasna baza nic nie zostalo zmienione.

```
uzytkownicy przed : null
headscale users create darkstar  -> "User created"
uzytkownicy po    : [{ "id": 1, "name": "darkstar" }]
preauthkey        : utworzony, wielokrotnego uzytku, waznosc 24h
```

Sam klucz **nie jest tutaj zapisany i nie moze byc**. Zostal pokazany operatorowi raz,
w oknie rozmowy. Jesli przepadnie, wlasciwa droga to wydac nowy, nie szukac starego.

## Tailscale — nietkniety, zmierzone po fakcie

```
100.71.8.70     darkstar-cbms     linux    idle; offers exit node
100.96.213.103  desktop-udi6m9f   windows  active; direct 192.168.2.50:41641
ControlURL: https://controlplane.tailscale.com
```

Zaden wezel nie zostal wypiety, `tailscale down` ani `logout` nie padly. SaaS zostaje
droga ratunkowa dopoki dwa wezly nie odpowiedza na nowym serwerze.

## Czego brakuje do dolaczenia drugiego wezla — decyzja operatora

```
LISTEN  127.0.0.1:8080
```

Headscale slucha **wylacznie na petli zwrotnej**. To byl swiadomy wybor: siec hosta bez
ani jednej nowej reguly. Konsekwencja jest jednak twarda i nie da sie jej obejsc od
strony klienta — **Windows pod `192.168.2.50` nie ma jak sie polaczyc**. Zaden klucz tego
nie zmieni; to kwestia adresu nasluchu, nie autoryzacji.

Dolaczenie dwoch wezlow wymaga rozszerzenia nasluchu na `192.168.2.1` — adres bramy
prywatnej, jeden z dwoch dopuszczonych przez `README` tego katalogu i przez zapore
`darkstar_host_guard`. To nadal **nie jest** `0.0.0.0` i nie dotyka `wlp2s0`.

Decyzja nalezy do operatora i dlatego nasluch NIE zostal rozszerzony samowolnie.

## Dolaczenie CBMS do wlasnego mesh — czemu nie zrobiono

Drugi profil albo osobny stan `tailscaled` wymaga uprawnien roota (wlasna jednostka
systemd i wlasne urzadzenie `tun`). Przelaczenie profilu w istniejacym demonie
odpielo by biezace polaczenie do SaaS, czyli zlamalo warunek "SaaS zostaje".
Zapisano wiec komende dolaczenia i czekano na zgode, zgodnie z poleceniem.

---

# Nasluch na bramie prywatnej — i sciana zapory, 2026-09-02

Headscale slucha teraz na `192.168.2.1:8080`. Petla zwrotna przestala odpowiadac, bo
Headscale przyjmuje **jeden** adres nasluchu. Darkstar zostal przestawiony na ten sam
adres przez `DARKSTAR_HEADSCALE_URL`; przy sieci hosta to ta sama maszyna i ruch jej nie
opuszcza.

## Pomiar

```
1 headscale przez brame   : 200
2 headscale przez petle   : 000        <- jeden adres nasluchu, skutek zamierzony
3 darkstar->headscale     : 200
4 tresc                   : {"baseUrl":"http://192.168.2.1:8080","health":"pass",
                             "nodeCount":null,"nodes":null,
                             "nodesReason":"no_api_key","reachable":true}
5 nasluch 8080            : 192.168.2.1:8080
6 sshd zapas z Windows    : 192.168.2.1:22 -> TcpTestSucceeded: True
7 sieci docker            : bridge deploy_default host none   (bez zmian)
8 tailscale               : 2 wezly, ControlURL: controlplane.tailscale.com
```

## Zapasowa droga dziala — i nie jest wystawiona na Vodafone

`sshd` nasluchuje na `0.0.0.0:22`, co samo w sobie wyglada szeroko, ale zapora
`darkstar_host_guard` zwezа to do jednego zrodla. Z `host-guard.nft`:

```
type filter hook input priority filter; policy drop;
iifname @downstream_ifaces ip saddr @darkstar_downstream_ipv4 tcp dport 22 accept
iifname @upstream_ifaces udp dport 41641 accept
iifname @upstream_ifaces udp sport 67 udp dport 68 accept
```

Z sieci nadrzednej Vodafone dopuszczony jest wylacznie transport WireGuard i klient DHCP.
SSH z `wlp2s0` **nie przechodzi**. Zmierzone z Windows `192.168.2.50`: port 22 odpowiada.
Droga zapasowa na wypadek padu SaaS jest wiec sprawna.

## Sciana: dolaczenie wezla jest zablokowane przez zapore

Zmierzone z Windows `192.168.2.50`:

```
192.168.2.1:22   -> TcpTestSucceeded: True
192.168.2.1:8080 -> TcpTestSucceeded: False
```

Powod jest w wersjonowanym `host-guard.nft` i jest jednoznaczny: z sieci prywatnej
przepuszczany jest **wylacznie port 22**. Polityka wejscia to `drop`, wiec port 8080
przepada po cichu. To nie jest problem klucza, adresu ani konfiguracji Headscale —
serwer slucha poprawnie i odpowiada lokalnie.

Otwarcie 8080 dla `192.168.2.0/24` wymaga zmiany `host-guard.nft` i przeladowania regul.
To jest wprost zakazane w tej sesji, wiec **nie zostalo zrobione**. Bez tej zmiany zaden
wezel spoza samego CBMS nie dolaczy do wlasnego mesh.

## Komendy JOIN — zapisane, NIEWYKONANE

Zadna z nich nie zostala uruchomiona. Wykonanie ktorejkolwiek na CBMS zerwaloby
`100.71.8.70`, czyli jedyna zdalna droge do hosta.

Windows (po otwarciu portu 8080 w zaporze):

```
tailscale login --login-server http://192.168.2.1:8080 --authkey <klucz z preauthkeys>
```

CBMS — wylacznie z osobnym stanem, nigdy na dzialajacym demonie:

```
# NIE uruchamiac na biezacym tailscaled - zerwie polaczenie do SaaS.
# Wymaga roota: osobna jednostka systemd, osobny --statedir i osobne urzadzenie tun.
tailscale --socket /run/tailscale-darkstar.sock up \
    --login-server http://192.168.2.1:8080 --authkey <klucz>
```

Weryfikacja po dolaczeniu:

```
docker exec darkstar-headscale headscale nodes list
```

## Warunek wyjscia, ktory nadal NIE jest spelniony

SaaS zostaje, dopoki **dwa** wezly nie odpowiedza na nowym serwerze. Odpowiada zero.
`tailscale down` i `tailscale logout` nie padly i nie padna, dopoki ten warunek nie
zostanie spelniony.
