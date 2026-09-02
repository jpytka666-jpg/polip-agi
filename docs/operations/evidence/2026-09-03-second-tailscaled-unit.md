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
