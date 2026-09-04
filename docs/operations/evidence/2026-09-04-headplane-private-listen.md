<!-- darkstar-header-v1 -->
<!-- po co: 2026-09-04-headplane-private-listen.md -->
<!-- nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie -->
<!-- autor: Marcin -->
<!-- powstal: 2026-09-04 -->
<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Sonnet 5
TIMESTAMP: 2026-09-04 16:37:00 Europe/London
REASON FOR CREATION: Zapisanie operatorskiego pomiaru potwierdzajacego, ze Headplane sluchacz
zostal na petli zwrotnej po przeladowaniu zapory dla portu 18080 - trzeba bylo sprawdzic, czy
ta zmiana zapory przypadkiem nie otworzyla tez portu 3000.
==========================================
-->

# Headplane - prywatny nasłuch, pomiar dla Step 14.3, 2026-09-04

## Werdykt operatora

**14.3 PASS.** Trzy fakty zmierzone przez operatora:

```
ss     -> tylko 127.0.0.1:3000 (zaden inny adres)
/admin -> przez petle zwrotna = 302
LAN    -> 000 (brak odpowiedzi z prywatnej sieci Darkstar, 192.168.2.0/24)
```

## Co ten pomiar faktycznie potwierdza

Rozdział "Test prywatnego nasłuchu — czego wymaga Step 14.3" w `deploy/headplane/README.md`
opisuje siedem sprawdzeń, które ma robić przyszły automatyczny test. Ten test **wciąż nie
istnieje jako skrypt** - poniższy pomiar jest ręczny i pokrywa część, nie wszystkie, z tych
siedmiu wierszy:

| Sprawdzenie z README | Pokryte tym pomiarem? |
|---|---|
| Nasłuch (`ss` poza `{127.0.0.1, 192.168.2.1}`) | TAK - `ss` pokazuje wyłącznie `127.0.0.1:3000`, więc nie ma adresu spoza dozwolonej pary. |
| Segment nadrzędny (`192.168.1.0/24`, Vodafone) | POŚREDNIO - skoro nasłuch nie obejmuje nawet `192.168.2.1` (bliższej, dozwolonej sieci), segment Vodafone za bramą tym bardziej nie ma jak się dostać. Nie jest to jednak bezpośredni pomiar z maszyny na `192.168.1.0/24`. |
| Źródło prawdy w plikach (`0.0.0.0`/`::` w compose/config) | NIE ZMIERZONE w tej turze - bez zmian od `deploy/headplane/compose.yml`, gdzie wartość jest wymuszona na `127.0.0.1`. |
| Interfejs `wlp2s0` | NIE ZMIERZONE w tej turze. |
| Zapora (`darkstar_host_guard` istnieje, `input` = `drop`) | NIE ZMIERZONE bezpośrednio w tej turze - pośrednio potwierdzone tym, że LAN nadal dostaje `000` mimo świeżego przeładowania zapory dla portu 18080. |
| Przypięcie (digest kontenera vs `version.lock`) | NIE ZMIERZONE w tej turze. |
| Wyciek sekretu | NIE ZMIERZONE w tej turze. |

## Dlaczego to ważne akurat teraz

Tego samego dnia zapora hosta została ręcznie przeładowana, żeby otworzyć port 18080 dla
sieci prywatnej (`docs/operations/evidence/2026-09-04-lan-18080.md`, commit `a8eb53b`).
Ten pomiar jest kontrolą, że ta zmiana nie otworzyła przy okazji portu 3000 - Headplane
zostaje dostępny wyłącznie z pętli zwrotnej hosta, dokładnie tak jak zaprojektowano.

## Co pozostaje nietknięte

Żaden unit systemd nie został `enable`d ani wystartowany. Żaden komponent Warlock nie
istnieje. Nic w tej turze nie zostało uruchomione na żywo - to wyłącznie zapis pomiaru,
który już wykonał operator.

## Powiązane

- Checklist źródłowy: `deploy/headplane/README.md`, sekcja "Test prywatnego nasłuchu".
- Poprzedni pomiar startu panelu: `docs/operations/evidence/2026-09-04-network-done.md`.
- Zmiana zapory tego samego dnia: `docs/operations/evidence/2026-09-04-lan-18080.md`.
