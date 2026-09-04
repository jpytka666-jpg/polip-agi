<!-- darkstar-header-v1 -->
<!-- po co: 2026-09-04-aions-server-wiedzy-inventory.md -->
<!-- nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie -->
<!-- autor: Marcin -->
<!-- powstal: 2026-09-04 -->
<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Opus 5
TIMESTAMP: 2026-09-04 21:40:00 Europe/London
REASON FOR CREATION: Inwentaryzacja serwera wiedzy AIONS na E: przed decyzja o budowie jego
nastepcy na Darkstarze. Marcin: "ma tyle roznych funkcji i systemow i nikt o nich nie pamieta".
Ten dokument jest odpowiedzia: co tam jest, co z tego zyje DZIS, a co umarlo.
MECHANICS: Wylacznie odczyt. Zywa baza Chroma otwierana z flaga -readonly. Zaden proces nie
zostal uruchomiony ani zatrzymany, zaden plik na E: nie zostal zmieniony.
SYSTEM PART: docs/research - materialy przed decyzja architektoniczna.
ARCHITECTURE FUNCTION: Podstawa faktograficzna dla planu "AIONS v2 na Darkstarze". Bez tego
plan bylby zgadywaniem, bo caly stan systemu istnieje dzis wylacznie w glowie i na dysku E:.
DEPENDENCIES/LINKS: E:\server wiedzy (odczyt), CBMS 100.64.0.2 (odczyt przez SSH),
docs/DARKSTAR_ROADMAP.md.
TECH STACK: Markdown - dokument, nie program.
LOCAL WORKSPACE: D:\codex-fresh-2026-08-28\worktrees\polip-agi-darkstar-plan
GIT COMMIT: PENDING
GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
==========================================
-->

# Inwentaryzacja AIONS na E:\server wiedzy — 2026-09-04

## Po co ten dokument

Serwer wiedzy AIONS rósł od listopada 2025 do dziś. Zawiera systemy, o których nikt już nie
pamięta, a które **nadal działają**. Przed budową następcy na Darkstarze trzeba wiedzieć, co
się dziedziczy. Ten dokument jest spisem z natury: co jest, co żyje, co umarło.

Metoda: wyłącznie odczyt. Żywa baza otwierana w trybie tylko-do-odczytu, żaden proces nie
uruchomiony ani zatrzymany, nic na E: nie zmienione.

## 1. Czym jest AIONS według własnej dokumentacji

Zasada architektoniczna, dosłownie z `AGENTS.md`:

> Linux jest warstwą sprzętową. CBMS jest systemem poznawczym. MCP jest warstwą wykonawczą.
> LLM jest wymienialnym interfejsem komunikacyjnym.

Cel projektu to **AI-operated OS** — system operacyjny prowadzony przez sztuczną inteligencję.
Roadmapa `AIONS_OS_ROADMAP.md` v16 (2026-07-10) dzieli to na 10 faz.

**Ważne dla nas:** "CBMS" w tej dokumentacji **nie oznacza laptopa Ubuntu**. CBMS to nazwa
systemu poznawczego (`E:\server wiedzy\aions_core`). Zbieżność z nazwą hosta `CBMS` jest
myląca i była źródłem nieporozumień.

## 2. Status faz według roadmapy v16

| Faza | Nazwa | Status | % |
|---|---|---|---|
| 1 | Core Runtime | zamknięta | 100% |
| 2 | Deploy Anywhere | zamknięta | 67% (Hyper-V + Proxmox PASS, VPS świadomie pominięty) |
| 3 | AI Control Plane | MVP | 60% |
| 4 | AIONS Identity | w toku | 70% |
| 5 | Local Intelligence | planowana | 0% |
| 6 | Distributed AIONS | planowana | 0% |
| 7 | Autonomous Infrastructure | iteracyjna | 25% |
| 8 | AIONS Image | MVP | 75% |
| 9 | AIONS Appliance | planowana | 0% |
| 10 | AIONS Ecosystem | tylko projekt | 0% |

Deklarowany następny duży krok (z lipca 2026): **migracja E:\server wiedzy → Redox**.

## 3. Mapa dysku — co gdzie leży

| Katalog | Rozmiar | Plików | Ostatni zapis | Co to jest |
|---|---|---|---|---|
| `runtime/` | 18,5 GB | 9 430 | **2026-09-03** | żywy runtime: integracje, host, node, systemd, llm, identity, secrets, transport, audit, state |
| `models/` | 1,8 GB | 3 | 2026-07-10 | lokalne modele |
| `acae/` | 1,6 GB | 288 | 2026-08-12 | osobny projekt: deterministyczne pakowanie kodu do kontekstu |
| `tools/` | 1,6 GB | 26 783 | 2025-12-24 | stare narzędzia |
| `venv/` | 1,2 GB | 33 228 | 2025-11-12 | środowisko produkcyjne MCP |
| `data/` | 367 MB | 807 | 2026-08-25 | **Chroma produkcyjna** + dwie kopie zapasowe |
| `index/` | 179 MB | 10 | 2025-10-03 | indeks |
| `_backups/` + `backups/` | 320 MB | 1 992 | 2026-08 | kopie |
| `aions_core/` | 53 MB | 1 003 | 2026-08-16 | **CBMS — system poznawczy** |
| `control_plane/` | 0,8 MB | 95 | 2026-08-16 | warstwa decyzyjna (Faza 3) |
| `mcpServers/` | 1,5 MB | 43 | 2025-11-13 | definicje serwerów MCP |
| `skills_lib/` | 0,2 MB | 261 | 2026-07-15 | biblioteka skilli |

## 4. Systemy odnalezione

**CBMS / `aions_core/`** — rdzeń poznawczy. Zawiera warstwę AGI (`agi_core`, `agi_evolution`,
`agi_learning`, `agi_neural_backprop`, `agi_soft_retrieval`, `agi_unified`), pamięć
(`cbms_memory`, `smart_cbms`, `cbms_curve_machine`, `memory_graph_engine`), wyszukiwanie
hybrydowe (`aions_hybrid_retrieval`, `aions_vector_enhancement`), bezpieczeństwo
(`aions_smart_guardrail`), most do Claude (`claude_aions_bridge`), własne benchmarki (GSM8K,
`full_bench_summary.json`, `PRODUCTION_AIONS_RESULTS.json`).

**Control Plane (Faza 3)** — `policy.py`, `planner.py`, `executor.py`, `scheduler.py`,
`orchestrator_loop.py`, `node_registry.py`, `node_dispatch.py`, `cbms_gate.py`,
`llm_adapter.py`. Wystawia `POST /v1/plan`, `POST /v1/execute`, `GET /v1/execution/{id}`.
Łańcuch: CBMS → Policy Engine → Task Planner → Execution Manager → MCP.

**Skill Engine** — 87 skilli plug-and-play plus 12 wyuczonych receptur sukcesu.
Potwierdzone na żywo, nie z dokumentacji.

**Operator (Fala 4b/5)** — pętla obserwuj→diagnozuj→działaj→zweryfikuj→ucz się, self-heal,
proaktywny brief dzienny (`scripts/operator_daily_brief.py`), integracje read-only Gmail i
Kalendarz (`runtime/integrations/`, prawdziwy przepływ OAuth, brak poświadczeń → nie live).

**AIONS Image (Faza 8, 75%)** — Packer buduje obraz systemu. Build PASS 2026-07-04, ~81 minut,
artefakt qcow2, walidacja instalacji 33/33 GREEN. **Gotowy obraz systemu już raz powstał.**

**ACAE** — osobny, przemyślany projekt: deterministyczne pakowanie repozytorium do kontekstu
LLM. Własny kontrakt terminologiczny (`pack`, `outline`, `drill`, `slice`, `name_path`,
`content_hash`, `pack_hash`, `B_ceiling`, `B_query`). Adresowanie symboli po nazwie, nie po
numerze linii — „linie się przesuwają, adres nie".

**Boot steps** — `runtime/aions_boot_steps.json` (zapis 2026-09-03) to żywy plik startowy;
jego krok 6 uruchamia tunel Control Room do Darkstara. **AIONS i Darkstar już się zazębiają.**

## 5. Co żyje DZIŚ — zmierzone, nie deklarowane

Trzy serwery działają w tej chwili na Windowsie, uruchomione 2026-09-03 ok. 13:55–14:02:

```
127.0.0.1:8765  AIONS Core (uvicorn)      /health -> 200 {"status":"ok","sessions":33}
127.0.0.1:8877  Phi-4-mini (llama.cpp)    /health -> 200 {"status":"ok"}
127.0.0.1:8000  ChromaDB                  /api/v1/heartbeat -> 200
0.0.0.0:8787    czwarty proces python     /health -> 404 (nie zidentyfikowany)
```

Warstwa MCP odpowiada na żywo:

```
system_health -> ok
skill_list    -> 87 skilli, 0 bledow
recipe_list   -> 12 receptur
session_list  -> 33 sesje
```

W tej sesji dostępnych jest **75 narzędzi MCP** z serwera `aions-context`: pamięć, CBMS,
skille, receptury, forge, git, docker, WSL, pulpit (klik/klawiatura/zrzut/drzewo UI), OCR,
przeglądarka, planowanie i wykonanie (`aions_plan`, `aions_execute_step`), myślenie
(`think_start`/`think_step`/`think_branch`), mowa (`speak`, `llm_speak`, `llm_understand`),
kalendarz, sieć, skanowanie projektów.

## 6. Co umarło albo zostało wyłączone

| Zadanie harmonogramu | Stan | Ostatni wynik |
|---|---|---|
| `AIONS Conductor` | Ready | 0 (sukces, 2026-09-03) |
| `AIONS_Operator_Soak` | Ready | 0 (sukces, **2026-09-04 21:31**) |
| `AIONS-Noworodek-nauka` | Ready | 0xC000013A (przerwane) |
| `AIONS_Chroma_Server` | **wyłączone** | 267014 |
| `AIONS_Core` | **wyłączone** | 1 |
| `AIONS_LLM_Server` | **wyłączone** | 267014 |
| `AIONS_Services_Watchdog` | **wyłączone** | 0 |
| `AIONS_Orchestrator` | **wyłączone** | 0 |
| `AIONS MCP HTTP` | **wyłączone** | 0xC000013A |

**Kluczowa obserwacja:** serwery działają, ale ich autostart jest wyłączony. Utrzymuje je
`AIONS Conductor`. Watchdog, który miał je pilnować i restartować, jest martwy. Czyli: system
żyje na jednej nodze — po następnym restarcie Windowsa nikt nie gwarantuje, że wstanie.

## 7. Pamięć — stan faktyczny

Żywa baza `E:\server wiedzy\data\chroma`:

```
33 kolekcje w bazie, ale 119 katalogow na dysku  -> 53 osierocone katalogi po skasowanych kolekcjach
4 629 embeddingow, wymiar 384 (domyslny model MiniLM, config kolekcji pusty)
SQLite 28,19 MB + indeksy HNSW 144,6 MB = ~173 MB
schemat sysdb v10
zakres czasu wpisow: 2025-11-15T16:35Z -> 2026-08-31T22:02Z (9,5 miesiaca)
API: v1 (nie v2!) - potwierdzone /api/v1/heartbeat -> 200
```

Klucze metadanych (bez wartości) pokazują gotowy schemat pamięci sesyjnej:
`role`, `agent`, `timestamp`, `ttl_days`, `expires_at`, `tags`, `run_id`, `host`, `os`,
`surface`, `era`, `tools_used`, `entry_count`, `skill_id`, `doc_type`, `type`, `categories`.

**Rozjazd wersji potwierdzony empirycznie:** Windows na E: mówi Chroma **v1**, CBMS mówi
**v2** (`chroma 1.4.4`, Python 3.12.3). To dokładnie wyjaśnia komentarz w kodzie Darkstara o
nodze `remote_e` domyślnie mówiącej v1.

## 8. Ryzyka i rozjazdy

1. **Trzy kopie pamięci, żadna nie jest kanonem.** E: (v1, żywa), CBMS `/home/owner/chroma-e-copy`
   (v2, kopia), CBMS `/var/lib/darkstar/chroma` (v2, 29 MB, **bez jednostki systemd — nie wstaje
   po restarcie**).
2. **Autostart wyłączony, watchdog martwy.** System trzyma się na `AIONS Conductor`.
3. **Wyszukiwanie w Darkstarze nie jest semantyczne** — dopasowuje nazwy kolekcji, nie treść.
4. **53 osierocone katalogi** w żywej bazie — śmieci po skasowanych kolekcjach.
5. **Cała wiedza operacyjna jest na jednym dysku w jednym komputerze.** Kopie są, ale nie ma
   dowodu odtworzenia.
6. **Zależność od Pythona** w całym stosie, przy deklarowanym celu Redox (Rust).

## 8a. Uzupełnienie — co naprawdę siedzi w 18,5 GB katalogu `runtime`

Pełny skan (zakończony po głównej analizie) pokazał, że rozmiar jest mylący:

```
runtime/host   18 449,8 MB / 9 010 plikow  <- artefakty obrazu systemu (Faza 8, qcow2)
runtime/state       64,3 MB /    30 plikow
runtime/audit       13,7 MB /    41 plikow
cala reszta        ~0,5 MB
```

Czyli **99,7% tego katalogu to jeden artefakt: zbudowany obraz systemu**. Kod runtime jest
mały. To dobra wiadomość dla migracji — nie ma czego ciągnąć poza artefaktem, który i tak
da się odtworzyć z przepisu.

**Trzy znaleziska, które skracają drogę Darkstarowi:**

1. **Gotowe jednostki systemd** — `runtime/systemd/` zawiera `templates/`, `user/` oraz
   `SERVICE_MAP.md` z rozpisanym podziałem na usługi (`aions-api.service`, health timer,
   `aions-mcp.service` jako adapter stdio). Problem autostartu na Linuksie jest częściowo
   rozwiązany na papierze — nikt tego tylko nie wdrożył na CBMS.

2. **Skille mają dojrzały kontrakt.** Każdy to `skills_lib/<id>/skill.json` z polami:
   `id`, `type`, `name`, `description`, `tags`, **`risk`**, **`access_class`** (read/write),
   **`platform`**, `handler`, `version`. To jest gotowy katalog z bramką ryzyka — dokładnie
   to, czego potrzebuje produkt.
   **Podział wg platformy: 24 skille jawnie `linux`, 63 bez pola platform** (do przeglądu).
   Czyli od razu wiadomo, które umiejętności należą do Darkstara.

3. **Semantyczne wyszukiwanie skilli było już badane i dostrojone.** W `runtime/` leży cała
   seria prac z lipca 2026: `build_skill_index_v2/v3/v4.py`, zbiory testowe
   (`skill_retrieval_testset.json`), siatki strojenia, analizy błędów, ewaluacje na zbiorze
   wstrzymanym i dokument `SEMANTIC_SKILL_RETRIEVAL_PROPOSAL.md` (2026-07-31, napisany po
   przebiegu wyłącznie odczytowym). To nie jest pomysł do wymyślenia od nowa - to jest praca
   do dokończenia.

## 9. Wniosek dla Darkstara

Darkstar nie musi budować AIONS od zera. Dziedziczy: sprawdzony schemat pamięci sesyjnej,
9,5 miesiąca realnej historii, 87 skilli z kontraktem ryzyka i platformy, 12 receptur,
projekt warstwy decyzyjnej, działającą pętlę operatora, gotowe szablony jednostek systemd,
dostrojone badania nad semantycznym wyszukiwaniem i gotowy przepis na obraz systemu.

Czego Darkstar musi dołożyć, bo tego nie ma: **trwałości** (autostart, watchdog, dowód
odtworzenia z kopii), **dostępu z wielu maszyn** (dziś wszystko jest przywiązane do jednego
Windowsa), **prawdziwego wyszukiwania semantycznego** i **niezależności od Pythona** w
warstwie, która ma trafić na Redox.

To nie jest przepisywanie AIONS. To jest przeniesienie mózgu z laptopa, który może paść,
na bramę, która ma stać.
