<!-- darkstar-header-v1 -->
<!-- po co: 2026-09-04-recall-first-light.md -->
<!-- nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie -->
<!-- autor: Marcin -->
<!-- powstal: 2026-09-04 -->
<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Opus 5
TIMESTAMP: 2026-09-04 23:00:00 Europe/London
REASON FOR CREATION: Pierwsze uruchomienie magazynu pamieci Darkstar (AIONS v2) na CBMS,
z dowodem odtworzenia z kopii - rzecza, ktorej w tym projekcie nigdy dotad nie wykonano.
==========================================
-->

# Pamięć Darkstara — pierwsze światło, 2026-09-04

## Werdykt

**Magazyn pamięci stoi, wstaje sam po restarcie, odmawia bez klucza, nie wychodzi poza pętlę
zwrotną, a kopia zapasowa została odtworzona i porównana co do punktu.** To zamyka trzy dziury
zmierzone tego samego dnia: brak autostartu, brak dowodu odtworzenia i pamięć uwiązaną do
jednego Windowsa.

## Co postawione

```
usluga:    darkstar-recall.service   -> active + enabled
kontener:  darkstar-recall            -> Up (Qdrant v1.19.1, digest z version.lock)
nasluch:   127.0.0.1:6333 (HTTP), 127.0.0.1:6334 (gRPC)  - i nic wiecej
dane:      /var/lib/darkstar/recall              (ext4, NIE exFAT)
migawki:   /var/lib/darkstar/recall-snapshots    (ext4)
klucz:     deploy/.env, tryb 0600, wygenerowany na hoscie, nigdy w repozytorium
```

## Autoryzacja — zmierzona, nie zalozona

```
curl 127.0.0.1:6333/collections bez klucza  -> 401
curl 127.0.0.1:6333/collections z kluczem   -> 200
ss -lnt: LISTEN 127.0.0.1:6333, LISTEN 127.0.0.1:6334   (zero 0.0.0.0, zero adresu bramy)
```

## Weryfikacja — 10 PASS, 1 WARN, exit 0

```
PASS  compose.yml nie uzywa taga latest
PASS  compose.yml wskazuje digest z version.lock
PASS  brak 0.0.0.0 w konfiguracji
PASS  nasluch wymuszony na petli zwrotnej
PASS  klucz API wylacznie jako odwolanie do deploy/.env
PASS  katalog danych poza dyskiem exFAT
PASS  kontener dziala
WARN  nie potwierdzono digestu dzialajacego obrazu (docker inspect zwraca lokalne ID)
PASS  nasluch na petli zwrotnej: 127.0.0.1:6333
PASS  dane na systemie plikow ext4
PASS  brak sladu klucza w ostatnich 500 wierszach logow
```

Pierwszy przebieg weryfikacji **zaczerwienil sie na wlasnym komentarzu** - doslowne szukanie
ciagu `/mnt/aionslab` znajdowalo go w ostrzezeniu wpisanym do `compose.yml`. Naprawione tak
samo, jak wczesniej naprawiono to samo przy sprawdzeniu `0.0.0.0`: linie komentarza sa
pomijane. Warto to odnotowac - test, ktory czerwieni sie z niewlasciwego powodu, jest gorszy
niz brak testu, bo uczy ignorowania czerwonego.

## DOWÓD ODTWORZENIA — to jest sedno tego wpisu

Kolekcja próbna z trzema punktami, migawka, odtworzenie do **osobnej** kolekcji obok żywej,
porównanie liczby punktów, sprzątnięcie:

```
INFO: kolekcja zrodlowa: proba_pamieci
INFO: punktow w zrodle: 3
PASS: migawka kolekcji: proba_pamieci-2744320037465945-2026-09-04-21-55-20.snapshot
INFO: odtwarzam migawke do OSOBNEJ kolekcji proba_pamieci__restore_probe (zywa nietkniete)
INFO: punktow po odtworzeniu: 3
PASS: DOWOD ODTWORZENIA: 3 = 3 punktow, kopia dziala
EXIT=0
```

Do 2026-09-04 w tym projekcie istniały cztery katalogi z kopiami pamięci
(`_backups`, `backups`, `chroma_backup_20251113`, `chroma.BACKUP-20260825`) i **żadnej próby
powrotu z którejkolwiek**. To jest pierwsza.

## Trwałość — przeżycie restartu usługi

```
systemctl restart darkstar-recall.service
systemctl is-active -> active
points_count po restarcie -> 3   (dane te same, nic nie zginelo)
```

Po zakończeniu prób kolekcja próbna została usunięta; baza jest pusta i gotowa na prawdziwe
dane.

## Wyszukiwarka plików — plocate co godzinę

```
plocate "chroma" -> real 0m0.032s, 4312 trafien (caly system)
find /home /var /etc -name '*chroma*' -> real 0m0.706s, 58 trafien (trzy katalogi)
indeks: /var/lib/plocate/plocate.db, 19 MB, obejmuje takze dysk 477 GB (91 141 pozycji)
darkstar-index.timer -> active, ostatni przebieg 22:53, nastepny za godzine
```

22 razy szybciej przy dwudziestokrotnie szerszym zakresie. Fabryczny timer plocate odświeża
raz na dobę; `darkstar-index.timer` robi to co godzinę, jako osobna jednostka, żeby
aktualizacja pakietu nie zdmuchnęła zmiany.

## Czego tu jeszcze nie ma

- Pamięć jest **pusta**. Migracja 4 629 wspomnień z Chromy na Windowsie to następny krok.
- Nie ma jeszcze warstwy Rust nad tą bazą — dostęp idzie dziś wprost do Qdranta po pętli.
- Nie ma katalogu narzędzi ani umiejętności w bazie (rewolwer narzędzi).
- `WARN` o digescie działającego obrazu do domknięcia: `docker inspect` zwraca lokalne ID
  obrazu, nie digest rejestru; sprawdzenie wymaga odczytu `RepoDigests` po `docker pull`.

## Powiązane

- `deploy/recall/{version.lock,compose.yml,verify,backup}`
- `deploy/systemd/{darkstar-recall.service,darkstar-index.service,darkstar-index.timer}`
- Inwentaryzacja tego, co dziedziczymy: `docs/research/2026-09-04-aions-server-wiedzy-inventory.md`
- Lekcja o autostarcie: `docs/operations/evidence/2026-09-04-real-reboot.md`
