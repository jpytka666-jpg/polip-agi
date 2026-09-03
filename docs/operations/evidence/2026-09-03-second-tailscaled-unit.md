<!-- darkstar-header-v1 -->
<!-- po co: 2026-09-03-second-tailscaled-unit.md -->
<!-- nie wolno: tailscale up/logout na biezacym demonie, start drugiego demona bez decyzji operatora -->
<!--
Autor: Marcin Szul
Model: Claude
Czas: 2026-09-03T00:30:00+01:00
Po co: Zapis przygotowanych plikow drugiego demona tailscaled wraz z instrukcja instalacji,
       ktora wymaga roota i dlatego NIE zostala wykonana.
-->

# Drugi tailscaled — pliki gotowe, NIC nie zainstalowane

Powstaly dwa pliki. **Zaden nie zostal skopiowany na maszyne, zaden demon nie zostal
uruchomiony, `systemctl daemon-reload` nie padl.**

| Plik w repozytorium | Docelowe miejsce |
|---|---|
| `deploy/systemd/tailscaled-headscale.service` | `/etc/systemd/system/tailscaled-headscale.service` |
| `deploy/headscale/join-cbms` | zostaje w repozytorium, uruchamiany stamtad |

## Osiem linii — co sprawdzono na maszynie

Zadna wartosc w plikach nie jest zgadnieta. Kazda zostala potwierdzona odczytem z CBMS:

```
1 binarka                : /usr/sbin/tailscaled  (istnieje)
2 flaga -statedir        : jest
3 flaga -socket          : jest
4 flaga -tun             : jest
5 flaga -port            : jest
6 port demona SaaS       : 41641 (0.0.0.0 i [::])  -> nowy bierze 41642
7 jednostka SaaS         : active, --state=/var/lib/tailscale/tailscaled.state
                                    --socket=/run/tailscale/tailscaled.sock
8 /var/lib/tailscale-headscale : NIE ISTNIEJE (utworzy je systemd przez StateDirectory=)
```

## Cztery rozdzielenia — kazde konieczne

Dwa demony `tailscaled` na jednym hoscie bija sie o cztery zasoby. Pominiecie
ktoregokolwiek konczy sie albo tym, ze nowy demon nie wstanie, albo **tym, ze przejmie
stan starego** — a stary trzyma jedyne zdalne wejscie na ten host.

| Zasob | Demon SaaS (istniejacy) | Demon Headscale (nowy) |
|---|---|---|
| stan | `/var/lib/tailscale/tailscaled.state` | `--statedir=/var/lib/tailscale-headscale` |
| gniazdo | `/run/tailscale/tailscaled.sock` | `--socket=/run/tailscale-headscale/tailscaled.sock` |
| urzadzenie | `tailscale0` | `--tun=headscale0` |
| port UDP | `41641` | `--port=41642` |

Jednostka celowo **nie ma** `Requires=` ani `After=` na `tailscaled.service`: te demony nie
zaleza od siebie i awaria jednego nie ma prawa pociagnac drugiego. `ExecStopPost` nie
wywoluje `down` ani `logout` — zatrzymanie uslugi ma usunac demona, a nie wypisac wezla
z Headscale.

## Instalacja — do wykonania przez operatora, wymaga roota

```
sudo install -m 0644 ~/polip-agi/deploy/systemd/tailscaled-headscale.service \
    /etc/systemd/system/tailscaled-headscale.service
sudo systemctl daemon-reload
sudo systemctl start tailscaled-headscale.service
systemctl is-active tailscaled-headscale.service
```

Po starcie, dolaczenie (klucz podstawia operator, nie jest w repozytorium):

```
sudo ~/polip-agi/deploy/headscale/join-cbms <preauth-key>
```

Skrypt sam sprawdza trzy rzeczy i przerywa, jesli ktoras nie gra: czy gniazdo drugiego
demona istnieje, czy Headscale odpowiada 200, i czy demon SaaS **nadal zyje**. Ostatni
warunek jest tam po to, zeby nie dolaczac drugiego wezla w momencie, w ktorym jedyna
zdalna droga na host wlasnie przestala dzialac.

## Znana przeszkoda po starcie — port UDP w zaporze

`darkstar_host_guard` przepuszcza `udp dport 41641` wylacznie z `@upstream_ifaces`,
a z sieci prywatnej dopuszcza tylko `tcp dport 22` i `tcp dport 8080`. Nowy demon
nasluchuje na **41642**, ktorego nie przepuszcza zadna regula.

Skutkiem bedzie brak polaczenia bezposredniego miedzy CBMS a Windows w prywatnym mesh.
Ruch spadnie na przekaznik DERP, a `config.yaml` wskazuje publiczna mape DERP Tailscale,
wiec zadziala to tylko dopoki jest internet — czyli **gorzej niz obecny SaaS**.

Domkniecie wymaga dopisania jednej reguły do `host-guard.nft` i swiadomego przeladowania
przez `darkstar-firewall-apply`. Regula NIE zostala dopisana: w tej sesji zapory dotknieto
raz, na wyrazne polecenie, i drugi raz bez polecenia nie bedzie.

## Nietkniete

Biezacy `tailscaled` do SaaS dziala i nie zostal dotkniety. Zaden `tailscale up`, `down`
ani `logout` nie padl na CBMS. Drugi demon nie zostal uruchomiony. Klucza preauth nie ma
w repozytorium.

---

# Regula UDP 41642 dopisana — w PLIKU, nie w /etc

Przeszkoda opisana wyzej zostala domknieta po stronie repozytorium. Regula lezy w
wersjonowanym `host-guard.nft` i **nie zostala zastosowana**.

## Osiem linii stanu

```
1 regula udp 41642 w repo   : linia 83, jest
2 regula udp 41642 w /etc   : 0 wpisow           <- NIE zastosowana
3 regula tcp 8080 w /etc    : 1 wpis             <- ta byla zastosowana wczesniej
4 nft -f w tej sesji        : nie wykonany
5 drugi demon tailscaled    : jednostka niezainstalowana, nieaktywna
6 demon SaaS                : active, nietkniety
7 port demona SaaS          : 41641, poza zakresem tej reguly
8 klucz preauth w repo      : brak, sprawdzone przed zapisem
```

## Trzy reguly downstream — komplet

```
tcp dport 22    accept   "downstream SSH management after Task 5"
tcp dport 8080  accept   "downstream Headscale control server, Task 13"
udp dport 41642 accept   "downstream Headscale WireGuard transport"
```

Wszystkie trzy maja ten sam zakres: wylacznie interfejs downstream i wylacznie zrodla
z `@darkstar_downstream_ipv4`, czyli `192.168.2.0/24`. Sieci nadrzednej Vodafone zadna
z nich nie dotyczy.

Port `41641` nalezy do demona do SaaS i **nie pojawia sie w zadnej nowej regule**. Nowy
demon dostal `41642` wlasnie po to, zeby te dwa ruchy nie mialy jak sie zejsc.

## Po co ta regula

Bez niej wezly prywatnego mesh nie polaczylyby sie bezposrednio i spadlyby na publiczny
przekaznik DERP — dzialaloby to tylko przy dostepie do internetu, czyli **gorzej niz
obecny SaaS**. Z nia lacza sie po kablu, nie wychodzac na zewnatrz.

## Jedna linia dla operatora

Wymaga roota; `sudo` wpisuje operator.

```
ssh owner@192.168.2.1 'sudo install -m 0644 ~/polip-agi/deploy/firewall/host-guard.nft /etc/darkstar/host-guard.nft && sudo ~/polip-agi/deploy/firewall/darkstar-firewall-apply && grep -c "dport 41642" /etc/darkstar/host-guard.nft'
```

Kopiuje plik, przeladowuje **wylacznie** tabele `inet darkstar_host_guard` ich wlasnym
skryptem (kasuje ta jedna tabele, waliduje `nft -c`, dopiero potem laduje) i pokazuje,
czy regula doszla. Ma wrocic `loaded inet darkstar_host_guard` oraz `1`.
