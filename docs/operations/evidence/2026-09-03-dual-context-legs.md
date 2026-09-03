<!-- darkstar-header-v1 -->
<!-- po co: 2026-09-03-dual-context-legs.md -->
<!-- nie wolno: pisac na dysk E, montowac bez zgody operatora, ruszac Chromy na Windows -->
<!--
Autor: Marcin Szul
Model: Claude
Czas: 2026-09-03T01:05:00+01:00
Po co: Zapis stanu dwoch nog pamieci - z gorzka konkluzja, ze do tej pory druga noga
       byla martwa z definicji, i z plikami, ktore maja to naprawic.
-->

# Dwie nogi pamieci — 2026-09-03

## Najwazniejsze: druga noga byla martwa Z DEFINICJI

Mechanizm dwoch nog **istnieje i dziala** — `ContextClient` ma noge preferowana i
zapasowa, przelacza sie przy ciszy, a odpowiedz mowi, ktora obsluzyla. Nie bylo czego
pisac od zera.

Martwy byl **adres** drugiej nogi:

```
LOCAL  : http://127.0.0.1:8000    -> 200
REMOTE : http://100.71.8.70:8000  -> 000
kto nasluchuje na 8000 na CBMS: 127.0.0.1:8000  i NIC WIECEJ
```

`100.71.8.70` to wlasny adres CBMS w sieci Tailscale. Chroma slucha wylacznie na petli
zwrotnej, wiec pod tym adresem nie odpowiadalo **nic i nigdy**. Nawet gdyby odpowiadalo,
bylby to **ten sam proces** co noga lokalna, tylko osiagniety inna droga.

Znaczy to tyle: cel "jedna pada, druga sluzy" **nie byl spelniony**. Padniecie Chromy na
CBMS zabieralo obie nogi naraz.

## Zmiana w kodzie — jedna linia, ale zasadnicza

```
- "http://100.71.8.70:8000"   (wlasny adres CBMS - martwy z definicji)
+ "http://192.168.2.50:8000"  (Windows - inna maszyna, inny proces, inny dysk)
```

To samo w `deploy/docker-compose.yml`. Druga noga wskazuje teraz na **inna maszyne**, a nie
na sama siebie. Nadal odpowiada `false`, ale z powodu, ktory da sie usunac, zamiast z
powodu wpisanego w architekture.

Testu nie dopisano i jest to swiadome: przelaczanie nog jest juz pokryte (12 testow w
`context_http`), a zmiana dotyczy wylacznie wartosci domyslnej konfiguracji. Test
sprawdzajacy literal adresu potwierdzalby wpis w pliku, nie zachowanie.

## Co juz istnieje po stronie Windows — nie trzeba tworzyc

Zmierzone, nie zalozone:

```
udzialy SMB : "server wiedzy" -> E:\server wiedzy   JUZ ISTNIEJE
              "E"             -> E:\                JUZ ISTNIEJE
dysk E      : 884.3 GB zajete, 47.2 GB wolne
port 8000   : nasluchuje, PID 15208, WYLACZNIE na 127.0.0.1
```

Udzialu **nie trzeba zakladac**. Jest.

## Dwie drogi do zywej drugiej nogi

| Droga | Co daje | Czego wymaga |
|---|---|---|
| A. Chroma na Windows slucha takze na `192.168.2.50` | druga noga odpowiada od razu, zero montazu, zero kopii | zmiana w konfiguracji AIONS po stronie Windows - **nie ruszalem** |
| B. Montaz udzialu `ro` na CBMS + lokalna kopia | druga noga zyje takze przy wylaczonym Windows | root na CBMS, cifs-utils, poswiadczenia |

Droga A jest krotsza. Droga B jest odporniejsza: przezywa wylaczenie Windowsa, bo dane
leza juz na CBMS. Obie sa rozlaczne i nie wykluczaja sie.

## Pliki przygotowane dla drogi B — NIC nie zamontowane

| Plik | Rola |
|---|---|
| `deploy/context/share.fstab.example` | wpis fstab, montaz `ro`, `nofail`, uid 1000 |
| `deploy/context/refresh-local-copy` | odswiezanie lokalnej kopii, POSIX sh |

Montaz jest `ro`, `file_mode=0444`, `dir_mode=0555` — brak bitu zapisu takze w widoku
systemu plikow. `nofail` jest tam po to, zeby **wylaczony Windows nie zablokowal startu
bramy domowej**.

Skrypt kopiujacy ma cztery bezpieczniki i kazdy przerywa robote zamiast ja udawac:
zrodlo musi byc punktem montazu, montaz musi byc `ro`, zrodlo nie moze byc puste, a nowa
kopia podmienia stara **dopiero po udanym skopiowaniu**. Nie ma `--delete` i nie ma
zadnej operacji na zrodle — kierunek jest jednostronny.

Poswiadczen do udzialu **nie ma w repozytorium** i miec nie moze; plik wzorcowy opisuje
tylko, gdzie je zalozyc i z jakimi prawami.

## Nietkniete

Dysk E — ani jednego zapisu. Chroma na Windows nie ruszona. Siec domowa, `nft`, pociagi
w Sterowni, Tailscale SaaS na CBMS. Drugi demon `tailscaled` nie zostal uruchomiony.
`0.0.0.0` nie uzyte nigdzie jako adres nasluchu.

---

# Prawdziwa blokada: zapora Windows, nie CBMS — 2026-09-03

Obie drogi z poprzedniej sekcji zakladaly, ze CBMS moze siegnac do Windowsa. **Nie moze.**

```
CBMS -> 192.168.2.50   ping        : NIE
CBMS -> 192.168.2.50:445  (SMB)    : cisza
CBMS -> 192.168.2.50:8000 (Chroma) : cisza
cifs-utils na CBMS                 : BRAK
```

Przyczyna jest po stronie Windows i nie ma nic wspolnego z `nft` na CBMS:

```
karta z adresem 192.168.2.50 : "WiFi 2"
profil sieciowy tej karty    : Public
zapora Windows               : wlaczona na wszystkich trzech profilach
reguly "File and Printer Sharing" przychodzace, wlaczone : BRAK (zero)
```

Na profilu **Public** zapora Windows odrzuca ruch przychodzacy. Dlatego milczy ping,
milczy 445 i milczy 8000 — mimo ze udzialy istnieja, a Chroma dziala.

Odblokowanie wymagaloby zmiany profilu karty na Private i wlaczenia regul udostepniania.
To jest zmiana zapory na maszynie operatora i **nie zostala wykonana**.

## Droga, ktora nie wymaga ruszania zadnej zapory

Ruch **wychodzacy** z Windowsa nie jest blokowany. SSH z Windowsa na CBMS dziala i jest
uzywane przez cala te sesje. Wystarczy odwrocic kierunek: zamiast CBMS ciagnac dane,
to Windows je wypycha.

```
sklad chroma : E -> data/chroma
rozmiar      : 371 plikow, 172.8 MB   (28.2 MB to sam chroma.sqlite3)
narzedzia    : scp, ssh, robocopy - wszystkie obecne na Windows
```

172 MB przenosi sie w minute. To jest **cala** pamiec, nie probka.

Powstal `deploy/context/push-chroma-to-cbms.ps1`. Proba na sucho, wykonana:

```
push-chroma: zrodlo ma 371 plikow, 172.8 MB
What if: Performing the operation "wyslij 172.8 MB" on target "owner@192.168.2.1"
```

Skrypt czyta zrodlo i **nie zapisuje na nim ani jednego bajtu** - katalog roboczy powstaje
w TEMP. Kopiowanie idzie dwuetapowo, bo sklad Chromy to zywa baza SQLite: najpierw
robocopy do TEMP, dopiero stamtad scp. Kopiowanie bazy wprost w trakcie zapisu dalo by
plik rozdarty w polowie transakcji.

Podmiana na CBMS nastepuje **dopiero po udanym transferze**; przerwany scp zostawia
poprzednia kopie nietknieta. Zadna sciezka nie jest wpisana w kod - wszystkie przychodza
ze srodowiska.

## Czego NIE zrobiono i dlaczego remote_e_ok nadal falszywe

`remote_e_ok` pozostaje `false` i **nie zostalo podkolorowane**. Kopia jeszcze nie
istnieje, a nawet gdy powstanie, musi ja ktos serwowac - Chroma nie odpowiada z katalogu,
tylko z procesu. Drugi proces Chromy na CBMS, na osobnym porcie petli zwrotnej, jest
nastepnym krokiem i wymaga decyzji operatora.

Do tego czasu druga noga jest uczciwie martwa i tak sie raportuje.
