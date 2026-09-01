<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Opus 5
TIMESTAMP: 2026-09-01 06:23:11
REASON FOR CREATION: Evidence pierwszego przebiegu darkstar-network-preflight na hoscie CBMS po udowodnionym reboocie (Task 4).
==========================================
-->

# Darkstar network preflight — przebieg na CBMS

Data: 2026-09-01 06:23:11 UTC
Host: CBMS (`owner@100.71.8.70`), boot `dbe6ad68-eff2-4954-baa4-66c7077d1c8c`
Sposob uruchomienia: skrypt podany na stdin (`ssh ... 'sh -s'`) — **nic nie zapisano na hoscie**,
bez sudo, bez zmiany polaczen.

## Wynik surowy

~~~text
# --- identity
preflight_version=1
generated_at=2026-09-01T06:23:11Z
hostname=CBMS
upstream_iface=wlp2s0
downstream_iface=enp1s0

# --- tool versions
networkmanager_version=nmcli tool, version 1.46.0
nft_version=nftables v1.0.9 (Old Doc Yak #3)
systemd_version=systemd 255 (255.4-1ubuntu8.17)
ip_version=ip utility, iproute2-6.1.0, libbpf 1.3.0
iw_present=iw version 6.7

# --- upstream radio
upstream_operstate=up
upstream_carrier=1
upstream_driver=iwlwifi
upstream_firmware=36.c8e8e144.0 8000C-36.ucode

# --- ap capability
ap_mode_supported=probable-see-combinations
ap_valid_combinations=1

# --- downstream ethernet
downstream_operstate=down
downstream_carrier=0
downstream_link_detected=no

# --- routes
default_route_v4=default via 192.168.1.1 dev wlp2s0 proto dhcp src 192.168.1.48 metric 600
default_route_v6=default via fe80::62d8:a4ff:fe48:8b37 dev wlp2s0 proto ra metric 600 pref medium

# --- networkmanager profiles
active_profile=Vodafone480B38 1:wlp2s0:802-11-wireless:activated
active_profile=tailscale0:tailscale0:tun:activated
active_profile=br-e0a0946cdec3:br-e0a0946cdec3:bridge:activated
active_profile=lo:lo:loopback:activated
active_profile=docker0:docker0:bridge:activated

# --- forwarding
ipv4_forwarding=1
ipv6_forwarding=1

# --- host guard
host_guard_table=absent-or-unprivileged

darkstar-network-preflight: read-only discovery complete
~~~

## Uwagi do przebiegu

- Kod wyjscia 127 z ostatniej linii: plik w kopii roboczej Windows ma CRLF, wiec zdalny `sh`
  zobaczyl osierocony CR. **Wszystkie sekcje wykonaly sie w calosci**; blob w Git ma LF (0 CR),
  wiec uruchomienie z checkoutu na Ubuntu tego nie powtorzy. Przy podawaniu na stdin z Windows
  nalezy usunac CR.
- `host_guard_table=absent-or-unprivileged` wynika z uruchomienia **bez sudo** — nie jest to
  sprzecznosc z dowodem `2026-09-01-firewall-reboot-persistence.md`, gdzie tabela byla obecna
  przy odczycie z uprawnieniami root.

## Odczyt faktow

- Upstream `wlp2s0` dziala: iwlwifi, firmware `36.c8e8e144.0`, trasa domyslna przez
  `192.168.1.1`, adres lokalny `192.168.1.48`.
- Downstream `enp1s0` **istnieje, ale nie ma linku** — `operstate=down`, `carrier=0`,
  `Link detected: no`. Interfejs jest, kabel nie.
- `iw` obecny (6.7); zgloszona jedna grupa kombinacji interfejsow. Zdolnosc AP+STA na tym samym
  radiu pozostaje **nierozstrzygnieta** — wymaga odczytania pelnego `iw list`.
- Forwarding IPv4 i IPv6 juz wlaczony.
- Na hoscie sa aktywne mostki `docker0` i `br-e0a0946cdec3` — przy adresacji `10.44.0.0/24`
  nalezy sprawdzic brak kolizji z sieciami Dockera przed apply bramy.

## Wybor baseline — NIE PODJETY

Kolejnosc z planu stawia na pierwszym miejscu `wlp2s0` upstream + `enp1s0` downstream. Sprzet na
to pozwala, ale Step 4.5 wymaga potwierdzonego carrier na `enp1s0`, a ten wynosi `0`. Decyzja
czeka na podlaczenie kabla Ethernet miedzy CBMS a Windows i powtorzenie pomiaru.

Zadne haslo nie zostalo zapisane. Brama nie byla uruchamiana; `darkstar-gateway-apply` nie byl
wywolany ani w trybie probnym, ani zadnym innym.
