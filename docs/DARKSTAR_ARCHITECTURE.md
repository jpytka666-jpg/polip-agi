# Darkstar Architecture

## 1. Cel

Darkstar jest niezależnym środowiskiem dla agentów AI. Nie jest modelem i nie jest przywiązany do jednego dostawcy modeli.

Rdzeń Darkstar odpowiada za tożsamość, sesje, politykę, pamięć, koordynację agentów, podłączanie narzędzi, automatyzację i audyt. Same narzędzia oraz wykonawcy mają być wymienialnymi rozszerzeniami.

## 2. Zasada nadrzędna

**Agent może zaproponować działanie. Darkstar decyduje, czy działanie wolno wykonać.**

Zgoda jednego lub wielu agentów nie jest sama w sobie zgodą właściciela.

## 3. Warstwa uruchomieniowa

Na pierwszym etapie pięć logicznych warstw działa w jednym procesie Rust i jednym kontenerze:

1. Context / Memory
2. Connection / Session / Isolation
3. Agent Round Table
4. Tentacles / Tools
5. Automation

Podział logiczny pozostaje wyraźny. Podział na osobne procesy lub usługi nastąpi dopiero wtedy, gdy pomiary lub granica bezpieczeństwa dadzą konkretny powód.

## 4. Rdzeń kontra pluginy

Rdzeń Darkstar nie może być zależny od języka pluginu.

Plugin jest osobnym wykonawcą, który deklaruje:

- nazwę i wersję,
- wersję kontraktu Darkstar,
- język/runtime,
- platformę,
- dostępne możliwości,
- wymagane uprawnienia,
- sposób połączenia.

Rdzeń widzi manifest, żądanie i wynik. Nie musi wiedzieć, czy implementacja jest napisana w Rust, Pythonie, C, C++, Go, C#, Javie, PowerShellu lub innym języku.

## 5. Transporty pluginów

Minimalny zestaw adapterów projektowych:

- stdio dla lokalnych procesów;
- HTTPS dla usług zdalnych;
- MCP dla kompatybilnych narzędzi i klientów;
- uwierzytelniony peer dla AIONS/Ionis i innych zaufanych wykonawców.

Native FFI jest opcją specjalną dla przypadków wymagających bardzo niskiego narzutu. Nie jest głównym mechanizmem rozszerzania platformy.

## 6. Model przepływu

```text
Agent / Client
      |
      v
Authentication
      |
      v
Session
      |
      v
Policy
      |
      v
Round Table
      |
      v
Proposal
      |
      v
Authorization
      |
      v
Plugin / Tentacle
      |
      v
Execution
      |
      v
Verification
      |
      v
Audit + Context
```

## 7. Poziomy ryzyka

Każda możliwość jest klasyfikowana co najmniej jako:

- Read — odczyt;
- Propose — przygotowanie zmiany bez skutku końcowego;
- Execute — wykonanie skutku zewnętrznego;
- Destructive — działanie potencjalnie niszczące.

Rdzeń może wymagać dodatkowej zgody dla operacji wykonawczych i niszczących.

## 8. AIONS / Ionis

AIONS nie jest wbudowywany do Darkstar. Jest zaufanym wykonawcą dostępnym przez osobny, uwierzytelniony kanał.

Dzięki temu Darkstar może działać w chmurze, a AIONS może korzystać z lokalnego GPU, plików, WPC i innych lokalnych zasobów.

## 9. Azure

Azure jest wymienialnym runtime'em, nie źródłem prawdy projektu.

Docelowo:

```text
GitHub repository
    -> GitHub Actions
    -> versioned container image
    -> Azure Container Apps
```

Kod, historia i definicje pozostają na GitHubie. Odtworzenie Darkstar na innym serwerze ma wymagać jedynie nowego uruchomienia tego samego obrazu lub zbudowania obrazu z tego samego commita.

## 10. Pamięć

Kontener nie jest traktowany jako główny magazyn trwałego stanu.

Warstwa pamięci ma być kompatybilna z CBMS i ma umożliwiać selektywny odczyt dużych zbiorów bez konieczności odtwarzania całości.

## 11. Security lab

Przyszły zestaw narzędzi bezpieczeństwa będzie budowany jako osobne capability packs. Narzędzia mogą obejmować diagnostykę sieci, analizę ruchu, testy aplikacji, analizę systemową, forensics, Linux/Windows i połączenia z odizolowanymi maszynami laboratoryjnymi.

Domyślne uprawnienia mają być minimalne. Narzędzie nie otrzymuje dostępu tylko dlatego, że zostało zainstalowane.

## 12. Zasada rozwoju

Nie dodajemy rozproszenia, usług ani abstrakcji bez potrzeby. Najpierw działający pionowy fragment, potem pomiar, następnie rozszerzenie.

Każdy etap powinien kończyć się możliwym do sprawdzenia wynikiem.
