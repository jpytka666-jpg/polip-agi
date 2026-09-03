<!-- darkstar-header-v1 -->
<!-- po co: world-ingress.md -->
<!-- nie wolno: otwierac portow na routerze, wiazac na 0.0.0.0, gasic SaaS -->
<!--
Autor: Marcin Szul
Model: Claude
Czas: 2026-09-03T07:00:00+01:00
Po co: Wyjasnia po ludzku, jak wystawic wlasny serwer mesh do swiata bez wydawania
       zlotowki i bez otwierania czegokolwiek na routerze - oraz jaka jest tego cena.
-->

# Świat bez zakupów — jak telefon ma trafić do domu

## Problem w jednym zdaniu

Telefon poza domem musi jakoś dojść do serwera, który stoi w twojej piwnicy — a twój
router od Vodafone nie wpuszcza nikogo z zewnątrz i **nie chcemy tego zmieniać**.

## Rozwiązanie: tunel, który wychodzi, a nie wpuszcza

Zamiast otwierać drzwi od zewnątrz, maszyna sama **dzwoni na zewnątrz** i trzyma tę
rozmowę otwartą. Świat puka wtedy do Cloudflare, a nie do twojego domu, a Cloudflare
przekazuje pukanie po tej otwartej rozmowie.

To jest jak zadzwonić do kogoś i nie odkładać słuchawki — a nie jak zostawić otwarte okno.

Konkretnie:

- na routerze **nie trzeba przekierować ani jednego portu**,
- na `wlp2s0`, czyli od strony Vodafone, **nic nie nasłuchuje**,
- serwer dalej słucha tylko tam gdzie dziś, czyli na `192.168.2.1:8080`,
- zapory **nie ruszamy**.

## Cena, której nie ukrywam

Za darmo dostajesz tunel **doraźny**. Bez konta, bez domeny, bez płacenia. Ale:

> **Adres jest losowy i zmienia się przy każdym restarcie tunelu.**

A serwer mesh musi znać swój własny adres z zewnątrz — trzyma go w ustawieniu
`server_url`. Zmiana adresu znaczy więc trzy rzeczy naraz:

1. `server_url` trzeba poprawić,
2. serwer przeładować,
3. **urządzenia, które już dołączyły, mają zapisany stary adres** i przestaną trafiać.

Dlatego usługa jest ustawiona tak, żeby **nie wstawała sama po restarcie**. Cichy restart
w nocy zmieniłby adres i po cichu odciął ci telefon — a dowiedziałbyś się o tym dopiero
wtedy, gdy naprawdę byłby potrzebny.

## Kiedy warto zapłacić

Jedno: **własna domena** (kilkadziesiąt złotych rocznie). Wtedy adres jest stały, tunel
może wstawać sam, a `server_url` ustawia się raz. Konto Cloudflare jest darmowe — płaci
się wyłącznie za domenę.

Dopóki tego nie masz, wariant darmowy jest w porządku do **sprawdzenia, że działa** i do
doraźnego użycia. Nie nadaje się jako coś, na czym polegasz na co dzień.

## Co jest przygotowane, a co nie

| Plik | Rola |
|---|---|
| `deploy/headscale/world-tunnel` | skrypt: sprawdza warunki, stawia tunel, wyciąga adres |
| `deploy/systemd/cloudflared-headscale.service` | usługa trzymająca tunel |

**Nic nie zostało uruchomione.** `cloudflared` nie jest zainstalowany na maszynie,
tunel nie stoi, `config.yaml` nie został zmieniony.

## Kolejność, gdy zechcesz to włączyć

```
# 1. sprawdzenie warunków - niczego nie uruchamia
~/polip-agi/deploy/headscale/world-tunnel check

# 2. instalacja cloudflared (operator, wymaga roota)
#    pobranie ze strony Cloudflare, x86_64

# 3. tunel na pierwszym planie, wypisze adres https
~/polip-agi/deploy/headscale/world-tunnel run

# 4. wpisanie tego adresu do server_url i przeładowanie headscale
```

Dopiero po kroku 4 telefon ma dokąd dołączyć.

## Czego ta zmiana NIE dotyka

Kabla, prywatnego mesh ani Tailscale SaaS. Wszystkie trzy dotychczasowe drogi do maszyny
zostają nietknięte. Tunel jest **czwartą** ścieżką, wyłącznie dla klienta spoza domu, i
jego awaria nie rusza niczego w domu.

Telefon **nie został dołączony** i nie zostanie, dopóki nie powiesz.

---

## Dom kontra świat — cztery drogi, każda do czego innego

| Droga | Skąd działa | Do czego służy | Stan |
|---|---|---|---|
| Kabel `192.168.2.1` | tylko z domu | ostatnia deska ratunku, zawsze działa | żywa |
| Prywatny mesh `100.64.0.2` | z domu i z każdego dołączonego urządzenia | codzienna praca, Sterownia, SSH | żywa |
| Tailscale SaaS `100.71.8.70` | zewsząd | zapas, dopóki własne nie okrzepnie | żywa, **nie gasić** |
| Tunel Cloudflare | z całego świata | żeby telefon spoza domu miał dokąd dołączyć | **nie postawiony** |

Trzy pierwsze działają. Czwarta jest przygotowana i nic z niej nie jest uruchomione.

## Sterownia — jak się do niej dostać

Sterownia nie jest wystawiona nigdzie. Otwiera się ją tunelem, na czas pracy:

```
ssh -N -L 18080:127.0.0.1:18080 owner@100.64.0.2
```

Potem w przeglądarce `http://127.0.0.1:18080/`.

Adres `100.64.0.2` to CBMS **w prywatnym mesh** — nie w chmurze. Ta komenda wymaga, żeby
telefon lub komputer był dołączony do własnej sieci. Z domu można też użyć `192.168.2.1`.

Uwaga: to zwykły proces, nie usługa. **Nie przeżyje restartu komputera** — po restarcie
trzeba go uruchomić ponownie.

## Skąd tunel bierze dane

Z `192.168.2.1:8080` — adresu, pod którym Headscale **naprawdę odpowiada** (zmierzone,
kod 200). Mimo że wygląda „sieciowo", ruch **nie opuszcza maszyny**: to jej własny adres
na karcie do sieci domowej, a tunel stoi na tej samej maszynie.

### Uwaga o pętli zwrotnej

Naturalniejsze byłoby `127.0.0.1:8080` — dwa procesy na jednym komputerze nie muszą
rozmawiać przez kartę sieciową. **Dziś to jednak nie działa: pętla zwraca zero.**

Powód nie jest usterką. Headscale przyjmuje **jeden** adres nasłuchu i używa go dla sieci
domowej. Przestawienie go na pętlę odcięłoby węzły w domu od serwera. To wybór między
dwiema rzeczami, nie błąd do naprawienia — i dlatego pętla jest tu **uwagą**, a nie
ustawieniem domyślnym.

`world-tunnel check` sprawdza oba adresy i wypisuje, który odpowiada. Gdyby kiedyś
skonfigurowany adres zamilkł, a odpowiadał ten drugi — **skrypt odmówi startu zamiast po
cichu podstawić zamiennik**. Ciche podstawienie znaczyłoby wystawienie do świata adresu,
o którym nie wiesz.

---

# Sterownia przeniesiona na prywatny mesh — 2026-09-03

## Osiem linii

```
1 port 18080 trzyma   : ssh -N -L ... owner@100.64.0.2   <- MESH, odczytany wlasciciel
2 health              : 200
3 strona Sterowni     : 200
4 procesow ssh        : 1  (dokladnie jeden tunel)
5 po 20 sekundach     : nadal MESH, health 200, SaaS NIE wrocil
6 tailscaled SaaS     : nietkniety, nie zatrzymywany
7 ubite petle         : 2 (wskrzeszacze tunelu do 100.71.8.70)
8 tunel do SaaS       : zaden nie dziala
```

## Co naprawde blokowalo te zmiane

Nie sam tunel. Przez cala noc w tle chodzily **dwie petle** uruchomione we wczesniejszych
sesjach Claude:

```
while true; do ssh ... -L 127.0.0.1:18080:127.0.0.1:18080 owner@100.71.8.70; sleep 5; done
```

Co piec sekund odtwarzaly tunel przez SaaS. Kazde przelozenie Sterowni na mesh bylo po
kilku sekundach po cichu cofane - port wracal do starej drogi, a pomiar `200` wygladal
poprawnie, bo odpowiadala Sterownia, tylko inna droga.

Dlatego pierwsze przelozenie zostalo zaraportowane jako udane, a nie bylo. Wykrylo to
dopiero odczytanie **wlasciciela gniazda**, a nie samego kodu odpowiedzi.

Wniosek na przyszlosc: `200` na porcie mowi, ze cos odpowiada - nie mowi, **ktoredy**.
Przy zmianie drogi trzeba czytac proces trzymajacy port.

## Kolejnosc, ktora zadzialala

```
1. ubic PETLE (nie tylko ssh) - inaczej wskrzesza tunel w 5 sekund
2. ubic pozostale procesy ssh
3. sprawdzic, ze port 18080 jest WOLNY
4. postawic jeden tunel na owner@100.64.0.2
5. odczytac wlasciciela portu - czy to na pewno mesh
6. odczekac i sprawdzic ponownie, czy nic nie wrocilo
```

## Nietkniete

`tailscaled.service` (SaaS) dziala i nie byl zatrzymywany - zmienila sie droga Sterowni,
nie stan demona. `nft`, `apt`, `cloudflared` i tunel do swiata bez zmian.

Uwaga: ten tunel to zwykly proces, nie usluga. **Nie przezyje restartu komputera.**

---

# Tunel Sterowni trwaly — wpiety w istniejacego nadzorce, 2026-09-03

## Osiem linii

```
1 plik w repo        : deploy/windows/Start-ControlRoomTunnel.ps1
2 launcher na E      : E:\server wiedzy\runtime\control_room_tunnel_run.cmd
3 krok w JSON        : "Control Room tunnel (mesh)", pozycja 6 z 6 - OSTATNIA
4 JSON po zmianie    : poprawny, 6 krokow, pozostale piec nietkniete
5 Test-Path launchera: True  (tak samo sprawdza to Start-Cmd)
6 suchy start        : port najpierw WOLNY, po starcie zajety
7 kto trzyma 18080   : ssh ... owner@100.64.0.2   -> MESH: TAK
8 health / ile ssh   : 200 / dokladnie 1 proces
```

## Nie powstal drugi orkiestrator

Istniejacy nadzorca to `AIONS Conductor` w Harmonogramie -> `AUTOSTART AIONS SERVER FULL.bat`
-> `AIONS SERVER FULL.ps1`. Kroki czyta z `E:\server wiedzy\runtime\aions_boot_steps.json`
i **sam opisuje droge rozszerzania**:

> "To add a new service: (1) create a launcher script, (2) add a step object (...),
> (3) restart conductor. No code changes required."

Zrobiono dokladnie to. Kodu nadzorcy **nie tknieto**.

## Trzy rzeczy wyniesione z lektury, ktore zmienily wykonanie

**1. Krok musi byc OSTATNI.** Nadzorca leci krokami po kolei i przy nieudanym robi
`exit 2` - przerywa caly start. Tunel wstawiony wyzej blokowalby ChromaDB, VectorStore,
MCP i Llame. Na koncu jego awaria opoznia najwyzej samo zakonczenie.

**2. Launcher musi byc PLIKIEM.** `Start-Cmd` robi `Test-Path $Path` i odpala przez
`cmd.exe /d /s /c`. Wskazanie w kroku wprost na `.ps1` albo na linie z argumentami by nie
zadzialalo. Stad `.cmd` obok `chroma_server_run.cmd` i reszty - konwencja tego katalogu.

**3. Sam kod 200 NIE wystarcza.** Nadzorca sprawdza `type: http`, wiec zobaczy 200 z
dowolnego tunelu na 18080 - takze z tunelu przez SaaS. Dlatego to LAUNCHER pilnuje drogi:
czyta proces trzymajacy gniazdo i konczy sie bledem, jesli to nie mesh. Ta sama pomylka
sprawila wczesniej, ze nieudane przelozenie Sterowni wygladalo na udane.

## Co robi launcher

```
1. ubija WYLACZNIE ssh z przekierowaniem portu 18080  (nie `Get-Process ssh | Stop-Process`,
   bo to zrywa takze sesje roboczne operatora i agentow)
2. czeka, az mesh 100.64.0.2 odpowie - do 60 s, konfigurowalne
3. stawia dokladnie jeden tunel
4. czyta WLASCICIELA gniazda i sprawdza, czy to mesh
5. sprawdza health 200
6. konczy sie 0 albo 1 - nadzorca dostaje uczciwa odpowiedz
```

Gdy mesh nie odpowiada, launcher **nie siega po 100.71.8.70**. Cicha zamiana drogi jest
gorsza niz brak tunelu: dawalaby 200 i falszywe poczucie, ze Sterownia idzie po wlasnej sieci.

## Czego nie ruszono

Kodu nadzorcy, pozostalych pieciu krokow, demona SaaS na CBMS, `nft`, `cloudflared`.
Nie powstalo zadne nowe zadanie w Harmonogramie.
