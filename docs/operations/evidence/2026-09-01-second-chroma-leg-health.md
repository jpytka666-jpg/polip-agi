<!-- darkstar-header-v1 -->
<!-- po co: 2026-09-01-second-chroma-leg-health.md -->
<!-- nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie -->
<!-- autor: Marcin -->
<!-- powstal: 2026-09-01 -->
<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Opus 5
TIMESTAMP: 2026-09-02 02:05:00
REASON FOR CREATION: Stan drugiej nogi Chroma na CBMS przed jakimkolwiek przenoszeniem danych z E:.
==========================================
-->

# Druga noga Chroma na CBMS — stan wyjsciowy

Data: 2026-09-01. Host CBMS. **Baza na Windows nietknieta** — nie zatrzymywana, nie kopiowana.

## Co zastano na CBMS

| Pomiar | Wynik |
|---|---|
| `chromadb` | **BRAK** — `command -v chroma` puste, `import chromadb` → `ModuleNotFoundError` |
| `python3` | `3.12.3` |
| `/var/lib/darkstar` | nie istnial |
| port `8000` | wolny |
| miejsce na `/` | 49 GB wolne z 114 GB (55% zajete) — z zapasem na 173 MB danych |

## Jedyna zmiana wykonana na hoscie

Jedna komenda z uprawnieniami administratora, uzasadniona tym, ze `/var/lib` nalezy do roota i
zwykly uzytkownik nie moze tam zalozyc katalogu:

~~~bash
sudo install -d -o owner -g owner -m 0750 /var/lib/darkstar/chroma
~~~

Wynik: `drwxr-x--- owner owner /var/lib/darkstar/chroma`. Prawa `0750` — dane pamieci nie sa
czytelne dla innych kont. Nic wiecej nie zainstalowano, nie uruchomiono i nie skopiowano.

## Health pustej nogi

`deploy/context/darkstar-chroma-health` (POSIX sh, tylko odczyt, `sh -n` = 0):

~~~text
data_dir=/var/lib/darkstar/chroma
bind_allowed=127.0.0.1,192.168.2.1
bind_forbidden=0.0.0.0,::
port=8000
data_dir_present=yes
data_dir_mode=750 owner:owner
data_dir_files=0
data_dir_bytes=0
chroma_binary=absent
port_state=free
heartbeat_127.0.0.1=000
heartbeat_192.168.2.1=000
collections=unavailable:no-response
~~~

## Werdykt — SYNC ZABLOKOWANY

Warunek z polecenia brzmial: pelny sync z E: **tylko gdy health=ok**. Health **nie jest ok** —
`chroma_binary=absent`, wiec pusta baza nie moze nawet wystartowac, a `heartbeat` na obu
dozwolonych adresach zwraca `000`.

Czego brakuje, w kolejnosci:

1. **Instalacja `chromadb` na CBMS** — pobranie pakietu i jego zaleznosci z sieci. To zmiana
   srodowiska hosta, wiec **czeka na wyrazne polecenie**; nie wykonano jej.
2. Uruchomienie na `127.0.0.1` **i** `192.168.2.1` — nigdy `0.0.0.0`. Nasluch na adresie
   nieokreslonym wystawilby pamiec AIONS na segment nadrzedny Vodafone.
3. Powtorzenie tego health i zapis wyniku tutaj.
4. Dopiero potem sync 173 MB z `E:\server wiedzy\data\chroma` — **przy zatrzymanej bazie na
   Windows**, inaczej kopia bedzie niespojna.

## Strona kodu — gotowa wczesniej niz noga

`ContextClient` w `darkstar-core` obsluguje juz obie nogi i dziala z jedna: gdy CBMS milczy,
odpowiada noga na E:, a gdy milcza obie, zwraca blad zamiast pustej listy. Przejscie na noge
lokalna bedzie zmiana kolejnosci, nie przepisywaniem kodu.

Zadne haslo nie zostalo zapisane. Chroma na Windows dziala nieprzerwanie.
