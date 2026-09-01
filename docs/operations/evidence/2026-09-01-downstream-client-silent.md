<!-- darkstar-header-v1 -->
<!-- po co: 2026-09-01-downstream-client-silent.md -->
<!-- nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie -->
<!-- autor: Marcin -->
<!-- powstal: 2026-09-01 -->
<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Opus 5
TIMESTAMP: 2026-09-01 08:35:00
REASON FOR CREATION: Stan downstreamu po dopasowaniu zapory - sprawdzenie, czy cokolwiek po drugiej stronie kabla poprosilo o adres.
==========================================
-->

# Downstream 192.168.2.0/24 — HUB-CICHO

Data: 2026-09-01 ~08:25 UTC. Host CBMS (`owner@100.71.8.70`). Wylacznie odczyt.

## Wynik

| Pomiar | Wynik |
|---|---|
| `ip neigh show dev enp1s0` | **pusto** — zaden sasiad nie odpowiedzial |
| `ip -4 addr show enp1s0` | `192.168.2.1/24` |
| `/var/lib/NetworkManager/` | brak plikow dzierzaw |

**Werdykt: HUB-CICHO.** Zadne urzadzenie nie poprosilo o adres. Ping pominiety — nie ma do kogo.

## Co dziennik pokazuje

Strona hosta jest gotowa i to potwierdzone, nie zalozone:

~~~text
Sep 01 07:54:00 dnsmasq-dhcp[7132]: DHCP, IP range 192.168.2.10 -- 192.168.2.254, lease time 1h
Sep 01 07:54:00 NetworkManager: device (enp1s0): Activation: successful, device activated.
~~~

Rozdawanie adresow dziala, pula to `192.168.2.10`–`192.168.2.254`, dzierzawa godzinna.

## Sygnal wart uwagi

Wpis `device (enp1s0): carrier: link connected` powtarza sie regularnie:

~~~text
07:54:15, 07:56:45, 07:59:46, 08:02:46
~~~

To co okolo trzech minut, przy braku jakiegokolwiek ruchu. Moze oznaczac urzadzenie po drugiej
stronie, ktore cyklicznie podnosi i opuszcza link — albo kabel z niepewnym stykiem. **Nie
rozstrzygniete**: bez klienta nie da sie tego oddzielic od zwyklego zachowania portu.

## Czego brakuje do zamkniecia

Podlaczyc urzadzenie do kabla i sprawdzic, czy pobierze adres z puli oraz wyjdzie na zewnatrz.
Dopiero to potwierdzi dopasowanie zapory z `2026-09-01-host-guard-192-168-2-apply.md`.

Nic nie zmieniono: bez `nmcli`, bez nowego profilu, bez rebootu, bez trybu hotspot.
