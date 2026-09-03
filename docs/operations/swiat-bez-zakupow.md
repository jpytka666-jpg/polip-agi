<!-- darkstar-header-v1 -->
<!-- po co: swiat-bez-zakupow.md -->
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
