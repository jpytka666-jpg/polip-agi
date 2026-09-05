<!-- darkstar-header-v1 -->
<!-- po co: obowiązkowy punkt wejścia dla agentów i bezpieczne przekazanie pracy -->
<!-- nie wolno: ruszać sieci, wag, księgi ani cudzej pracy bez zgody na ten zakres -->
<!-- autor: Marcin -->
<!-- powstal: 2026-09-05 -->
<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: GPT-5 Codex
TIMESTAMP: 2026-09-05
REASON FOR CREATION: Marcin polecił dodać AGENTS.md po przeglądzie memów Claude'a,
aby następny agent odczytał intencje, dowody i ograniczenia przed przejęciem pracy.
SYSTEM PART: Darkstar / instrukcje pracy w repozytorium.
GITHUB METADATA: jpytka666-jpg/polip-agi
==========================================
-->

# Instrukcje dla agentów — przeczytaj przed pracą

Obowiązują w całym repozytorium. Ten plik jest mapą lektury i zasad, nie raportem
bieżącego stanu hosta. Nie zastępuje aktualnego polecenia Marcina ani instrukcji
wyższego priorytetu. Nie daje samodzielnej zgody na wdrożenie lub przebudowę.

## 1. Najpierw odtwórz stan, nie zaczynaj od nowa

- Ustal faktyczne repozytorium, branch i HEAD. Branch roboczy w chwili utworzenia
  tego pliku: `docs/darkstar-headscale-hotspot-plan`. Nie przełączaj automatycznie
  na `main`, `Darkstar` ani dawny `feat/darkstar-module-control`.
- W lokalnej kopii przeczytaj `git status --short --branch`, `git diff`,
  `git diff --cached` oraz ostatnie commity. Zachowaj zmiany nieśledzone i cudzą
  pracę. Przy dostępie tylko przez GitHub powiedz wprost, że nie znasz lokalnego
  stanu ani działających procesów.
- Przeczytaj w całości `README.md`, `docs/operations/PRZYWRACAJKA.md`
  i `docs/DARKSTAR_ROADMAP.md`. Następnie najnowszy handoff, jeśli istnieje,
  oraz dokumenty z `docs/operations/evidence/` dotyczące zadania.
- Porównuj daty, commity i dowody. Starsza Przywracajka lub nagłówek nie wygrywa
  z nowszym, konkretnym pomiarem. Sprzeczności nazwij, zamiast wybierać po cichu.
- Jeśli masz dostęp do wskazanego przez Marcina pliku startowego
  `E:\szul\Documents\OneDrive - Global Banking School\Desktop\START PROMPT GPT.MD`,
  przeczytaj go przed pracą. Jeśli nie masz dostępu, powiedz to; nie twierdź,
  że został przeczytany, i nie wymyślaj jego treści.

## 2. Mema na początku plików są obowiązkową lekturą

Przed analizą lub zmianą danego pliku przeczytaj CAŁY jego blok wstępny:
`darkstar-header-v1`, `THIS IS VERY IMPORTANT!!!`, powód utworzenia,
mechanikę, funkcję architektoniczną, zależności, ograniczenia i rewizje.
Przeczytaj powiązane mema i kod wywołujący w zakresie potrzebnym do zadania.

- Najpierw historia, dokumenty i istniejące dowody; nie odkrywaj ponownie
  rzeczy już ustalonych. Nowy test uruchamiaj dla rzeczywistej luki lub regresji.
- Memo opisuje intencję i historię, NIE jest dowodem poprawności kodu.
  Sprawdzaj deklaracje względem implementacji, testów i późniejszych wyników.
- Nie przepisuj całego pliku dla drobnej poprawki. Zachowaj autorstwo
  `AUTHOR: M. SZUL`, wcześniejsze mema i uzasadnienia.
- Nowe pliki opatruj nagłówkiem w składni właściwej dla formatu: autor,
  rzeczywisty model wykonujący zmianę, data, po co plik istnieje, rola,
  zależności i ograniczenia. Nie kopiuj cudzej tożsamości modelu.
  Gdy format nie dopuszcza komentarzy, umieść memo w towarzyszącym dokumencie.
- Przy zmianie zachowania dopisz zwięzłą, datowaną rewizję uzasadnienia.
  Nie traktuj starego `GIT COMMIT: PENDING` jako aktualnego SHA; sprawdź Git.

## 3. Mapa lektury: Noworodek / CBMS / tokenizer

Dla tego obszaru przeczytaj w całości dokumenty, w tej kolejności historycznej:

1. `docs/operations/evidence/2026-09-05-written-not-trained.md`
2. `docs/operations/evidence/2026-09-05-reflex-dictionary.md`
3. `docs/operations/evidence/2026-09-05-most-nie-jest-wpiety.md`
4. `docs/operations/design/2026-09-05-tokenizer-hybryda.md`
5. `docs/operations/evidence/2026-09-05-espdic-kandydaci.md`
6. Nowsze dokumenty i commity dotyczące tego samego obszaru.

Następnie mema odpowiednich plików w `crates/darkstar-embed/src/`:
`noworodek.rs`, `bin/spine.rs`, `bin/spine_loop.rs`, `bin/implant.rs`,
`bin/lathe.rs`, `bin/tokenize.rs`, `bin/espdic.rs`; czytaj implementację
zmienianego toru i jego testy, nie sam opis.

Ustalony podział ról: duży model lub agent = mózg; Noworodek = rdzeń odruchowy;
AIONS/Darkstar = ciało. Nie zmieniaj rdzenia w rozmówcę ani nie utożsamiaj
docelowej architektury z aktualnym prototypem wyszukiwania.

- Tokenizer ma doprowadzać języki do wspólnej reprezentacji CBMS.
  Sam wspólny zapis nie dowodzi rozumienia ani poprawności tłumaczenia.
- ESPDIC daje kandydatów esperanto/angielski, nie gotową obsługę polskiego.
  Oddzielaj pokrycie, wieloznaczność i poprawność. Mała próbka nie dowodzi
  stuprocentowej jakości całego słownika.
- Nie utożsamiaj pierwszego kandydata z właściwym znaczeniem ani jednorazowego
  wyboru z rozwiązaniem wieloznaczności we wszystkich kontekstach.
- Nie dodawaj modelu, nie przypisuj automatycznie selekcji Noworodkowi i nie
  przebudowuj księgi lub wag bez uzgodnienia tego etapu z Marcinem.
- Przed integracją sprawdź zgodność księgi, numeracji symboli, wag i szerokości
  identyfikatorów u16/u32. Nie odtwarzaj numeracji CBMS domniemaną regułą.
- Nieznane wejście i częściowy wynik muszą być odróżniane od pełnego sukcesu.
  Progi podobieństwa i odstępu są heurystykami, nie gwarancją poprawności.

## 4. Granice bezpieczeństwa i zakresu

- Nie ruszaj hotspotu, `wlp2s0`, `DARKSTAR-WiFi`, adresacji ani nie wracaj
  do dawnego układu 10.44 w ramach prac nad tokenizerem.
- Nie wykonuj `tailscale logout/down`, nie wyłączaj zapasowego Tailscale SaaS,
  nie przeładowuj zapory, nie restartuj hosta i nie wdrażaj usług bez zgody
  na konkretną operację. Nie przywracaj starego tunelu Sterowni do
  `100.71.8.70`; przeczytaj Przywracajkę i nowsze dowody sieciowe.
- Nie wystawiaj usług na `0.0.0.0` jako obejścia problemu połączenia.
- Nie publikuj haseł, kluczy, tokenów, plików `.env`, prywatnego korpusu ani wag.
  Korzystaj ze skonfigurowanego dostępu bez wypisywania sekretów.
- Oryginalna księga, wagi, bazy i kopie bezpieczeństwa nie są materiałem
  do nadpisywania. Nie zatrzymuj cudzych długich procesów ani ich nie duplikuj.
- GitHub nie obejmuje całego warsztatu: sprawdź wskazane w memach lokalne
  zmiany, zewnętrzne worktree, wagi i logi przed kontynuacją.
  Ścieżki z dokumentów są wskazówkami do weryfikacji, nie dowodem istnienia.
- Nie używaj force-push, reset --hard, checkout -- ani zbiorczego czyszczenia.
  Nie commituj cudzych plików. Commit/push wykonuj w zakresie zleconym przez
  Marcina; przed zapisem sprawdź, czy drugi agent nie przesunął brancha.

## 5. Dowód i przekazanie pracy

Dostosuj sprawdzenia do zmiany. Korzystaj z manifestów i istniejącej konfiguracji
testów/CI. Sam build nie dowodzi poprawności znaczenia, a test parsera nie dowodzi
działania tłumacza. Dla samej dokumentacji sprawdź treść, ścieżki i zakres diffu;
nie uruchamiaj z tego powodu treningu ani usług.

Raportuj po polsku, prosto: co zmieniono, czego dowodzą wyniki, co pozostaje
niezweryfikowane. Nie ogłaszaj dawnego wyniku jako świeżo wykonanego testu.

Przed zakończeniem lub przekazaniem sesji zapisz w uzgodnionym handoffie:

- repo, branch, dokładny HEAD i stan zmian lokalnych, w tym nieśledzonych;
- wykonane kroki oraz następny niedokończony krok i zakres zgody;
- komendy sprawdzające, wyniki, środowisko i datę;
- uruchomione procesy: host, PID/job, log, postęp i czy wolno je zatrzymać;
- lokalizacje ksiąg, wag, danych i zmian poza GitHubem, bez sekretów;
- dokumenty/mema wymagane do wznowienia, blokery i znane ograniczenia.

Nowa sesja nie dziedziczy automatycznie rozmowy ani pamięci innego agenta.
