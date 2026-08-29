# Task 0 — deterministic green baseline

Status: DONE_WITH_CONCERNS

## Zmienione pliki

- `crates/darkstar-server/tests/memory_http.rs`

Fixture integracyjny buduje `AppState` bezpośrednio z tokenem `secret`, pustą mapą sesji, świeżym `MemoryStore` i domyślnym `RunStreamHub`. Dodano test potwierdzający fail-closed przy `api_token=None`. Nie zmieniono produkcyjnego `AppState::from_env`, UI ani sieci.

## Decyzje

- Zachowano HEAD dokumentacyjny `3e1d9a2`; nie cofano zaakceptowanych dokumentów planu.
- Nie ustawiano globalnie `DARKSTAR_API_TOKEN`.
- Nie utworzono `test_support.rs`, bo API fixture jest używane tylko w tym module.

## Komendy testowe

| Komenda | Kod | Wynik |
|---|---:|---|
| `git rev-parse HEAD` | 0 | `3e1d9a2f3a4503b457b7d2b4daa80e9f3e9f0aa8` na wejściu |
| `git status --short --branch` | 0 | Czysty worktree na wejściu |
| `cargo test -p darkstar-server --test memory_http` (przed poprawką) | 101 | 3 testy failowały z 401 |
| `cargo test -p darkstar-server --test memory_http` (po poprawce, pierwsza próba) | 101 | MSVC `LNK1104` dla pliku tymczasowego |
| `cargo test -p darkstar-server --test memory_http` (po poprawce, z lokalnym TMP) | 0 | 16 passed, 0 failed |
| `cargo fmt --all -- --check` (przed poprawką importu) | 1 | Wykryto kolejność importu |
| `cargo fmt --all -- --check` + `cargo clippy --workspace --all-targets -- -D warnings` | 0 | Obie bramki zielone |
| `cargo test --workspace` | 0 | 47 + 1 + 11 + 16 testów, wszystkie zielone; doctest 0 |
| `npm ci` | 1 | Brak `frontend/package-lock.json`; polecenie przerwane po zawieszeniu |
| `npm run lint` | 1 | `eslint` niedostępny, zależności nie zostały odtworzone przez brak lockfile |
| `npm run build` | 1 | `tsc` niedostępny z tego samego powodu |
| `git diff --check` | 0 | Brak błędów białych znaków |

## Self-review

- Auth produkcyjny pozostaje fail-closed; test `missing_api_token_fails_closed` pokrywa brak tokenu.
- Żaden sekret nie został zapisany poza testową stałą `secret`.
- Brak zmian w `frontend`, `http.rs` i konfiguracji sieci.
- `target/` oraz `frontend/node_modules/` są ignorowane; lokalny `target/tmp` usunięto po testach.

## Commit

Kodowa poprawka: `b0a5f11a6ff70b6b620fdf30bcf830fd78abef79` (`test(darkstar):make-HTTP-fixtures-deterministic`).

## Git status

Po commicie poprawki: tylko nowy raport oczekujący na zapis/commit. Raport nie zawiera zmian produkcyjnych.
