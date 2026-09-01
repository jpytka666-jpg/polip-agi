<!-- darkstar-header-v1 -->
<!-- po co: README.md -->
<!-- nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie -->
<!-- autor: Marcin -->
<!-- powstal: 2026-09-01 -->
<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Opus 5
TIMESTAMP: 2026-09-01 20:10:00
REASON FOR CREATION: Inwentarz stanu przed wdrozeniem Headscale (Task 13) oraz wiazace ustalenia co do adresow nasluchu. Sam katalog jeszcze nic nie uruchamia.
==========================================
-->

# Headscale — inwentarz i zasady nasluchu

**Nic z tego katalogu nie jest jeszcze wdrozone.** Ten plik to zapis stanu i granic, ktore
obowiazuja, zanim powstanie `compose.yml`, `config.yaml.example` i reszta plikow z Task 13.

## Co jest na hoscie CBMS — zmierzone 2026-09-01

| Element | Stan |
|---|---|
| `headscale` | **BRAK** (`command -v` pusty) |
| `headplane` | **BRAK** |
| `tailscaled` | `active` — SaaS nadal jedyna droga mesh, wciaz droga ratunkowa |
| mesh | `100.71.8.70 darkstar-cbms` (idle, offers exit node); `100.96.213.103 desktop-udi6m9f` **active, direct `192.168.2.50:41641`** — Windows laczy sie juz przez brame Darkstar |
| kontener | `darkstar` na `127.0.0.1:18080->8080/tcp` — API tylko na petli zwrotnej |
| porty `8080` / `50443` / `443` | **wolne**, nikt nie nasluchuje |
| port `41641/udp` | zajety przez `tailscaled` (`0.0.0.0` i `[::]`) |

Wniosek: porty, ktorych Headscale potrzebuje, sa wolne. Kolizja wystapi wylacznie z `41641/udp`,
gdyby kiedys uruchamiac wlasnego klienta obok istniejacego `tailscaled`.

## Adresy nasluchu — wiazace

| Zasada | Wartosc |
|---|---|
| Dozwolone | `127.0.0.1` oraz `192.168.2.1` (adres bramy Darkstar na `enp1s0`) |
| **Zakazane** | `0.0.0.0`, `::`, dowolny adres z `192.168.1.0/24` (segment nadrzedny Vodafone) |
| **Zakazane** | `10.44.0.0/24` — ta pula nigdy nie istniala na tym hoscie, zostala porzucona |
| Panel Headplane | `127.0.0.1` albo `192.168.2.1`, nigdy adres publiczny |
| Zapora | dostep wylacznie z `192.168.2.0/24`; `darkstar_host_guard` juz ma ten zakres |

Powod: `input` w `darkstar_host_guard` ma polityke `drop`, a jedyne dopuszczone zrodla to petla
zwrotna, `tailscale0` i `192.168.2.0/24`. Nasluch na `0.0.0.0` wystawilby usluge takze na
`wlp2s0`, czyli do sieci nadrzednej — to jest dokladnie to, czego ten host ma nie robic.

## Pierwszy krok wdrozenia — Task 13, Step 13.1

**Przeglad upstream przed wyborem wersji.** Przeczytac README, LICENSE, informacje o wydaniu,
przyklad konfiguracji, dokumentacje polityki, uwagi o migracji bazy i instrukcje kontenera;
zapisac wybrany **niezmienny tag wydania i skrot commita**. Zaraz po tym Step 13.2: ustalic
`sha256` obrazu i zapisac `version.lock`. **Nigdy `latest`.**

Upstream: `https://github.com/juanfont/headscale`, licencja BSD-3-Clause.

## Czego nie ruszac

- `DARKSTAR-WiFi` / `192.168.2.1` — dzialajaca brama, poza zakresem tej pracy.
- `wlp2s0` i profil Vodafone — jedyne wyjscie na swiat.
- `tailscaled` — **nie wylaczac**, dopoki wlasny mesh nie obsluzy klienta testowego
  (Task 13, Step 13.13). To jedyna droga ratunkowa do hosta.
- `darkstar-firewall.service` i `/etc/darkstar/host-guard.nft` — zmiany tylko przez
  wersjonowany plik w repozytorium, nigdy recznie na hoscie.
- Kontener `darkstar` na `127.0.0.1:18080` — nie wystawiac go szerzej przy okazji.

## Warunek wejscia, ktory NIE jest spelniony

Task 13 wymaga stabilnej sieci „tylko przez Darkstar" z Task 6. Task 6 nie zostal wykonany:
Windows ma dzialajaca droge przez Darkstar, ale **nadal korzysta z Tailscale SaaS**, a skryptow
`Enable-DarkstarOnly` / `Test-DarkstarOnly` / `Restore-PreviousNetwork` jeszcze nie ma.
Instalacja Headscale przed domknieciem Task 6 bylaby wejsciem w zadanie z niespelniona bramka.

Zadne haslo ani token nie naleza do tego katalogu.
