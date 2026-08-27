# Dark Star MCP — Warstwa 02: Łączność, izolacja i bezpieczeństwo

## Cel warstwy

Druga warstwa Dark Star MCP jest warstwą połączeniową. Jej zadaniem jest bezpieczne przyjmowanie agentów i klientów z Internetu, tworzenie dla nich kontrolowanego środowiska sesyjnego oraz odseparowanie ich działań od systemu nadrzędnego do czasu świadomego zatwierdzenia zmian.

Agent nie powinien otrzymywać bezpośredniego dostępu do całego systemu Dark Star. Najpierw nawiązuje połączenie, zostaje uwierzytelniony, otrzymuje sesję i dopiero wtedy dostaje kontrolowany dostęp do swoich narzędzi, kontekstu i środowiska pracy.

## Transport

Podstawowym mechanizmem komunikacji internetowej powinien być bezpieczny transport HTTPS.

Dla agentów korzystających z MCP naturalnym podstawowym interfejsem jest MCP over Streamable HTTP. MCP może również współpracować z innymi transportami w zależności od klienta, ale zdalny Dark Star powinien traktować HTTPS jako podstawową warstwę transportową.

MCP jest protokołem komunikacji z agentami; nie jest sam w sobie mechanizmem bezpieczeństwa. Autentykacja, autoryzacja, zarządzanie sesją i izolacja muszą być zapewnione przez Dark Star.

## Uwierzytelnianie

GitHub ma być jednym z głównych elementów tożsamości i kontroli dostępu Dark Star.

Nie należy jednak traktować osobistego tokena GitHub użytkownika jako uniwersalnego klucza do całego systemu.

Preferowanym kierunkiem dla integracji z GitHubem jest GitHub App oraz krótkotrwałe, możliwie precyzyjne tokeny instalacyjne lub tokeny użytkownika zgodnie z wymaganym scenariuszem. GitHub Apps oferują granularne uprawnienia i krótszy cykl życia tokenów.

Po stronie człowieka GitHub może dodatkowo korzystać z silnego uwierzytelnienia, w tym passkeys i 2FA. Taki mechanizm może być używany przy świadomym zatwierdzaniu operacji przez właściciela. Nie należy jednak mieszać uwierzytelnienia człowieka z mechanizmem komunikacji usługowej agent–Dark Star.

## Sesja agenta

Po poprawnym uwierzytelnieniu agent otrzymuje sesję.

Sesja powinna posiadać:

- jednoznaczną tożsamość,
- tożsamość klienta/agenta,
- właściciela,
- czas rozpoczęcia i wygaśnięcia,
- aktualny kontekst,
- zestaw dozwolonych możliwości,
- powiązanie z warstwą pamięci,
- powiązanie z izolowanym środowiskiem roboczym,
- historię operacji.

Sesja nie jest pełnym dostępem do Dark Star. Jest kontrolowanym kanałem dostępu do określonego zakresu funkcji.

## Izolowany „bąbel” agenta

Koncepcja „dockowania” agenta zostaje przyjęta jako założenie architektoniczne, ale sposób jej implementacji wymaga dalszej analizy.

Nie zakładamy jeszcze, że każdy agent musi otrzymywać pełną maszynę wirtualną. Rozważane są co najmniej trzy poziomy izolacji:

1. izolowany kontener/sandbox,
2. izolowane środowisko uruchomieniowe na VM,
3. pełna dedykowana VM dla wymagających lub niezaufanych sesji.

Najbardziej prawdopodobnym punktem startowym jest lekki, efemeryczny sandbox lub kontener uruchamiany dla sesji, a nie pełna VM dla każdego połączenia. GitHub Codespaces pokazuje podobny wzorzec: środowisko deweloperskie jest uruchamiane w kontenerze na VM i może być traktowane jako izolowane, efemeryczne środowisko pracy.

Dark Star musi jednak zaprojektować własny sandbox pod kątem uruchamiania potencjalnie niezaufanych działań agenta. Nie należy zakładać, że zwykły kontener jest automatycznie wystarczającą granicą bezpieczeństwa.

## Zasada „najpierw zmiana, potem zatwierdzenie”

Agent powinien móc wykonywać działania w swoim środowisku roboczym, ale skutki tych działań nie powinny automatycznie stawać się zmianami w chronionym środowisku nadrzędnym.

W praktyce oznacza to model:

agent pracuje → zmiany powstają w izolowanym środowisku → zmiany są wersjonowane → powstaje commit/branch lub inny artefakt zmian → system ocenia zmianę → właściciel lub uprawniony agent zatwierdza → dopiero wtedy następuje publikacja/merge/wykonanie operacji o większym zasięgu.

Git jest tutaj mechanizmem kontroli i pochodzenia zmian. Niezatwierdzone zmiany pozostają widoczne jako oddzielony stan roboczy, branch, commit lub Pull Request.

## Ważne ograniczenie bezpieczeństwa

Sam commit Git nie jest sandboxem.

To, że agent utworzył skrypt i zapisał go w branchu, nie oznacza jeszcze, że skrypt jest bezpieczny. Dark Star musi kontrolować możliwość wykonania kodu osobno od możliwości zapisania go w repozytorium.

Kod utworzony przez agenta powinien być traktowany jako potencjalnie niezaufany do czasu przejścia odpowiedniej kontroli.

W szczególności nie wolno dopuszczać do automatycznego wykonania niezatwierdzonego kodu tylko dlatego, że został zapisany lub zacommitowany.

## Model izolacji

Warstwa 02 ma zapewnić granicę pomiędzy:

- agentem,
- jego sesją,
- jego sandboxem,
- pamięcią Dark Star,
- repozytoriami GitHub,
- systemami zewnętrznymi,
- chronionym środowiskiem właściciela.

Agent może otrzymać dostęp do narzędzi, ale każde narzędzie powinno działać przez kontrolowany interfejs Dark Star, a nie przez nieograniczony dostęp do hosta.

## GitHub jako mechanizm zatwierdzania zmian

GitHub dostarcza naturalny mechanizm pracy z proponowanymi zmianami: branch, commit, Pull Request, review i merge.

Dark Star powinien wykorzystać ten model jako część swojej warstwy bezpieczeństwa.

Przykładowo agent może przygotować zmianę w swoim środowisku, zapisać ją w osobnym branchu i utworzyć Pull Request. Zmiana może być testowana automatycznie. Dopiero świadoma decyzja właściciela albo odpowiednio zaufanego procesu może dopuścić ją do chronionej gałęzi.

## Warstwa 02 jako granica zaufania

Warstwa połączeniowa nie jest tylko routerem HTTP.

Jest pierwszą granicą zaufania Dark Star.

Jej podstawowe obowiązki to:

1. przyjęcie połączenia,
2. uwierzytelnienie klienta,
3. utworzenie sesji,
4. nadanie minimalnych możliwości,
5. utworzenie lub przypisanie izolowanego środowiska,
6. kontrola dostępu do pamięci i narzędzi,
7. izolowanie zmian,
8. rejestrowanie działań,
9. przygotowanie zmian do review/akceptacji,
10. dopiero po zatwierdzeniu umożliwienie operacji o szerszym zasięgu.

## Stan architektoniczny

Warstwa 02 jest połączeniem transportu internetowego, sesji, uwierzytelniania, izolacji wykonawczej i mechanizmu zatwierdzania zmian.

Szczegółowy wybór sandboxa, modelu sieciowego, granic kontenera/VM oraz polityki wykonywania kodu zostaje odłożony do etapu projektu bezpieczeństwa i implementacji.
