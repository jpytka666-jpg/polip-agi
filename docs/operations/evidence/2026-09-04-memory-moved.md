<!-- darkstar-header-v1 -->
<!-- po co: 2026-09-04-memory-moved.md -->
<!-- nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie -->
<!-- autor: Marcin -->
<!-- powstal: 2026-09-04 -->
<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Opus 5
TIMESTAMP: 2026-09-04 23:35:00 Europe/London
REASON FOR CREATION: Przeprowadzka calej pamieci AIONS ze starej Chromy na Windowsie do
magazynu Darkstar na Ubuntu. Do dzis brak tego jednego laptopa oznaczal brak kontekstu.
==========================================
-->

# Przeprowadzka pamięci — 2026-09-04

## Werdykt

**4 631 wspomnień z 31 kolekcji leży na Darkstarze. Zero błędów. Wyszukiwanie po znaczeniu
działa, kopia zapasowa odtworzona i porównana co do punktu.** Źródło na Windowsie nie zostało
tknięte — narzędzie nie ma w kodzie ani jednej ścieżki zapisu do Chromy.

## Liczby

```
zrodlo (Chroma v1, Windows 127.0.0.1:8000):  33 kolekcje
proba na sucho:   odczytano 4631, zapisano 0, bledow 0
przeprowadzka:    odczytano 4631, zapisano 4631, bledow 0
cel (Darkstar):   31 kolekcji, suma punktow 4631 - policzone niezaleznie po stronie celu
```

Rozbieżność 33 wobec 31 jest zamierzona: dwie kolekcje w źródle były puste, a pusta kolekcja
nie niesie informacji, więc nie została utworzona.

Największe przeniesione: `session_aions_skills_v4` 1 543, `session_claude_marcin_main` 1 269,
`session_aions_skills_v3` 706, `session_aions_skills` 87.

## Co zostało zachowane

Struktura pamięci sesyjnej przeżyła przeprowadzkę w całości. Pola widoczne w przeniesionym
punkcie:

```
chroma_id, collection, document, agent, role, timestamp, ttl_days, expires_at, tags, categories
```

`chroma_id` to oryginalny identyfikator ze źródła — dzięki temu każdy wpis da się z powrotem
odnaleźć w Chromie, a przeprowadzka jest odwracalna.

## Wyszukiwanie po znaczeniu — działa na prawdziwych danych

Zapytanie wektorem istniejącego wspomnienia o trzy najbliższe:

```
score 1.00000000  role assistant  2026-07-15   (ten sam wpis)
score 0.76789150  role assistant  2026-07-15
score 0.75080734  role assistant  2026-07-15
```

To jest zdolność, ktorej Darkstar do dzis NIE MIAL: `/v1/context/search` dopasowywal jedynie
podlancuch do NAZWY kolekcji, nie do tresci.

## Kopia zapasowa i dowód odtworzenia — na prawdziwych danych

```
PASS: migawka calej bazy: full-snapshot-2026-09-04-22-10-03.snapshot
INFO: kolekcja zrodlowa: session_claude_marcin_main
INFO: punktow w zrodle: 1269
PASS: migawka kolekcji utworzona
INFO: odtwarzam do OSOBNEJ kolekcji (zywa nietkniete)
INFO: punktow po odtworzeniu: 1269
PASS: DOWOD ODTWORZENIA: 1269 = 1269 punktow, kopia dziala
EXIT=0
```

Poprzedni dowód (2026-09-04, pierwsze światło) był na trzech punktach próbnych. Ten jest na
najważniejszej kolekcji w systemie.

## Jak to zrobione — bez Pythona

`crates/darkstar-recall` z binarką `recall-migrate`: rozmowa dwóch serwerów HTTP. Czyta z
Chromy przez `/api/v1/collections/{id}/get`, zapisuje do Qdranta paczkami po 200. Własny,
minimalny klient HTTP na `std::net` — zero nowych zależności, bo każda z nich to kolejna
rzecz do przeniesienia na Redoxa.

Uruchomione **na Ubuntu**, z tunelem odwrotnym do Chromy na Windowsie — dzięki temu klucz
dostępu do pamięci ani razu nie opuścił maszyny, na której mieszka.

## Czego jeszcze nie ma — luka wykryta przy tym etapie

**Nie da się jeszcze zapytać pamięci zwykłym zdaniem.** Wyszukiwanie działa, ale przyjmuje
wektor, nie tekst. Zamiana zdania na wektor wymaga modelu osadzeń po stronie Darkstara —
Chroma robiła to sama w tle, Qdrant tego nie robi z założenia. Do zamknięcia w następnym
etapie; model musi być ten sam, którym liczono istniejące wektory (384 wymiary), inaczej
stare wspomnienia i nowe pytania nie trafią w ten sam świat.

Poza tym: źródło na Windowsie zostaje jako kopia historyczna. Nic z niego nie skasowano.

## Powiązane

- `crates/darkstar-recall/` — klient i narzędzie przeprowadzki
- `deploy/recall/` — magazyn, weryfikacja, kopie zapasowe
- `docs/operations/evidence/2026-09-04-recall-first-light.md` — postawienie magazynu
- `docs/research/2026-09-04-aions-server-wiedzy-inventory.md` — co dziedziczymy
