# Dark Star MCP — Warstwa 01: Kontekst i pamięć

## Cel warstwy

Pierwsza warstwa Dark Star MCP jest warstwą magazynowania, organizowania i odczytu kontekstu oraz pamięci powstającej podczas pracy agentów.

Dark Star nie jest kolejnym AI ani samodzielnym „myślącym obiektem”. Jest infrastrukturą dla agentów AI. Jego zadaniem jest zapewnienie agentom trwałego, kontrolowanego i możliwego do odtworzenia kontekstu.

Pamięć ma obejmować między innymi:

- sesje pochodzące z różnych urządzeń,
- sesje różnych agentów,
- kontekst konkretnych spraw i projektów,
- wykonane działania,
- wyniki działań,
- decyzje i ich uzasadnienia,
- notatki kontekstowe,
- historię zmian kontekstu,
- relacje pomiędzy sesjami, agentami, zadaniami i artefaktami.

## Kompatybilność z CBMS

Dark Star ma używać tego samego języka reprezentacji pamięci, w którym komunikują się CBMS i Noworodek. Nie tworzymy drugiego, konkurencyjnego formatu pamięci.

Założenie architektoniczne jest takie, że pamięć Dark Star może osiągnąć bardzo duży rozmiar. Dlatego format musi umożliwiać efektywny odczyt danych bez konieczności pełnego rozpakowywania całego zbioru.

CBMS jest punktem odniesienia dla tej warstwy: kompresja i reprezentacja danych mają umożliwiać dostęp do potrzebnych fragmentów pamięci bez kosztownego rekonstruowania całości.

## Sesje

Każdy agent lub klient, który łączy się z Dark Star, otrzymuje własną sesję. Sesja jest identyfikowalnym kontenerem kontekstu, działań i wyników.

Sesja musi mieć:

- jednoznaczną tożsamość,
- właściciela lub podmiot nadrzędny,
- źródło połączenia,
- czas utworzenia i cykl życia,
- kontekst roboczy,
- historię działań,
- powiązania z pamięcią trwałą,
- możliwość odtworzenia stanu potrzebnego agentowi.

Sesje z różnych urządzeń nie powinny być traktowane jako przypadkowe rozmowy. Mają być częścią jednego systemu pamięci, przy zachowaniu izolacji i kontroli dostępu.

## Historia i wersjonowanie

Git ma być jednym z fundamentalnych mechanizmów śledzenia zmian w artefaktach i stanie roboczym Dark Star.

Tam, gdzie ma to sens, zmiany powinny być możliwe do przedstawienia jako wersje, commity, branche i relacje pomiędzy nimi.

GitHub zapewnia wizualny commit graph, Pull Requesty i historię zmian. Dark Star ma wykorzystywać tę właściwość jako element obserwowalności i kontroli zmian, a nie tylko jako sposób przechowywania kodu.

## Zasada projektowa

Pamięć Dark Star nie może być tylko wielkim magazynem tekstu. Powinna być strukturą, z której agent może selektywnie pobierać właściwy kontekst bez konieczności ładowania wszystkiego.

Warstwa 01 ma więc łączyć:

1. trwałą pamięć,
2. pamięć sesyjną,
3. historię zmian,
4. relacje pomiędzy informacjami,
5. kompatybilność z CBMS,
6. selektywny odczyt dużych zbiorów danych.

## Stan architektoniczny

Warstwa 01 jest fundamentem pamięci Dark Star MCP. Nie definiuje jeszcze konkretnego backendu bazodanowego ani finalnego formatu implementacyjnego. Te decyzje mają wynikać z istniejącego formatu CBMS i wymagań dotyczących skali, opóźnienia oraz możliwości bezpośredniego odczytu skompresowanych danych.
