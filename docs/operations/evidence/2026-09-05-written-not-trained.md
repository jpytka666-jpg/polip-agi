<!-- darkstar-header-v1 -->
<!-- po co: 2026-09-05-written-not-trained.md -->
<!-- nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie -->
<!-- autor: Marcin -->
<!-- powstal: 2026-09-05 -->
<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Sonnet 5
TIMESTAMP: 2026-09-05 13:35:00 Europe/London
REASON FOR CREATION: Model przestal byc uczony, a zaczal byc pisany - i po raz pierwszy
przekroczyl prog, ktory blokowal wszystko inne. Dokument zapisuje, czym to zmierzono
i w jakiej kolejnosci, zeby dalo sie to powtorzyc, a nie tylko zapamietac.
==========================================
-->

# Model pisany, nie trenowany — 2026-09-05

## Werdykt

**Noworodek przekroczył próg, którego pięć przebiegów treningu nie ruszyło. Nie przez
naukę — przez wpisanie wiedzy wprost w wagi.**

Program oceniający ma trzy werdykty. Przez pięć prób mówił *UCZY SIE, ALE PLYTKO*.
Od chwili, gdy zaczęliśmy wpisywać, mówi:

```
WYNIK: UZYWA KONTEKSTU.
       Tego nie da sie osiagnac bez patrzenia na poprzednie symbole.
```

To jest ocena samego programu, nie nasza. I to jest ta poprzeczka, o której w
`train-cbms.rs` napisano: *everything larger is gated behind its answer*.

## Analogia, która to opisuje — słowami Marcina

> To jak w Matriksie ma być. **Wgraj mi umiejętność obsługi Bella.**

Dokładnie to robimy. Model nie odkrywa, które znaki są częste — dostaje to wgrane.
Nie odkrywa, co po czym idzie — dostaje to wgrane. Uczy się **tylko tego, czego wgrać
się nie da**.

## Zmierzone — jeden korpus, jeden model, zmienia się tylko punkt startu

| Co wpisano | Wynik | Kroków | Czas | Werdykt programu |
|---|---|---|---|---|
| nic — trening od zera | 8,5487 | 300 | 783 s | uczy się, ale płytko |
| nic — model 4× większy | 7,0127 | 180 | 3178 s | uczy się, ale płytko |
| częstości | **5,1612** | 70 | 169 s | **używa kontekstu** |
| częstości + pary | **4,9420** | 25 | 97 s | **używa kontekstu** |

Progi wypisane przez sam program: rzut kostką **10,8920**, sama znajomość częstości
**5,3937**.

**Trzynaście minut nauki dało 8,55. Półtorej minuty z wgraną wiedzą dało 4,94.**

## Trzy rzeczy, które trzeba było dołożyć modelowi

**1. Rubryka wyjściowa.** Model liczył wynik jednym mnożeniem — nie miał gdzie trzymać
zdania „ten znak jest po prostu częsty". Dołożone: `model.lm_head.bias`, po jednej
liczbie na symbol, dodawanej do każdej odpowiedzi.

Rubryka jest **zamrożona** — nie uczy się. Trzyma wiedzę wpisaną z zewnątrz, a reszta
sieci uczy się tego, czego rubryka nie wyraża.

**2. To samo w ścieżce treningu.** Trening **nie wywołuje** `forward()` — ma własną kopię
obliczeń, żeby zachować pośrednie wyniki. Rubryka dołożona tylko w jednym miejscu
oznaczałaby, że model uczy się na innej matematyce, niż potem odpowiada. Taka rozbieżność
wychodzi po miesiącach jako „niewyjaśniona różnica".

**3. Rejestracja.** Wczytywanie wag sprawdza listę oczekiwanych warstw i odrzuca nieznane.
Bez wpisania na listę rubryka była ignorowana z komunikatem `tensor not found in manifest`.

Wszystkie **75 testów Noworodka przechodzi** po tych zmianach.

## Co dokładnie wpisano

**Częstości** — jedna liczba na znak, wprost do rubryki. Odejmowana jest średnia, bo
`softmax` i tak ignoruje stałą dodaną do wszystkich pozycji; bez tego rubryka niosłaby
wielką stałą, która nic nie zmienia, za to psuje skalę gradientów.

**Pary** — związek między dwoma znakami, dla którego osobnego miejsca nie ma. Ale model
przewiduje B po A wtedy, gdy wektor **wejściowy** A pasuje do **wyjściowego** B. Więc
wektory dobrano tak, żeby pasowały dokładnie tam, gdzie pary występują w korpusie:
każdy symbol dostaje stały kierunek liczony ze swojego numeru, a wektor symbolu powstaje
jako suma kierunków jego sąsiadów. Symbole chodzące w podobnym towarzystwie wychodzą
wtedy podobne.

Wiersze są normalizowane do równej długości — inaczej symbol występujący 16 080 razy
zdominowałby każde porównanie. Liczy się **kierunek**, w który symbol wskazuje; jak często
występował, siedzi już w rubryce.

## Awaria po drodze, zapisana bo pouczająca

Pierwsza próba wpisania **rozwaliła wagi**. Bez ograniczenia mnożniki sięgnęły **239,9×**,
a trening zwrócił `odlozone: inf` — liczby wybuchły.

Przyczyna leżała w kształcie danych, nie w sile: z 53 746 symboli słownika w korpusie
występuje **tylko 5 312**. Pozostałe 48 434 mają tę samą, bardzo niską częstość, zaniżają
średnią i ściskają odchylenie — więc symbole obecne wychodzą daleko poza skalę.

Pomiar wykrył to w minutę. Dołożony twardy limit i test sprawdzający siły od 0,1 do 10.

## Kopie i nietykalność

| Katalog | Rola |
|---|---|
| `C:\temp\aions-cbms-train` | **oryginał — nietknięty**, otwierany wyłącznie do odczytu |
| `D:\noworodek-backup-2026-09-05` | kopia bezpieczeństwa, 324,2 MB, 1154 pliki |
| `D:\noworodek-bias-2026-09-05` | kopia robocza — tu powstały zmiany |

**Uwaga:** kopia robocza dzieli historię z oryginałem (`.git` wskazuje na worktree
w `C:\temp\aions-qwen-kv-probe-rebuild-2026-08-24`). Dlatego **nie zapisano tam nic** —
zmiany leżą jako pliki. Przeniesienie ich do repozytorium Noworodka wymaga osobnej
decyzji i osobnej gałęzi.

Wagi wszystkich wersji leżą na CBMS w `~/models/noworodek/`, poza repozytorium.

## Czego nadal nie ma

**Model nie umie opisać całego zdania.** Ma metodę zwracającą wektor **na token**, ale
zwijania po tokenach w kodzie nie ma. Bez tego nie wepnie się do gniazda ucznia, które
czeka od wczoraj.

**Rubryka się nie uczy.** To jest dziś celowe, ale warto kiedyś zmierzyć, czy odmrożenie
pomaga, czy szkodzi.

**Wpisano dwie rzeczy z korpusu.** Zostały co najmniej trójki i dłuższe okno.

## Powiązane

- `crates/darkstar-seed/` — narzędzie wpisujące
- `docs/operations/evidence/2026-09-05-noworodek-first-weights.md` — pierwsze wagi w ogóle
- `docs/operations/evidence/2026-09-04-shadow-mode.md` — gniazdo czekające na ucznia
