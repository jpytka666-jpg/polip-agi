// darkstar-header-v1
// po co: git_http.rs
// nie wolno: wykonywac fetch, checkout, merge, reset ani zadnej innej mutacji repozytorium
// autor: Marcin
// powstal: 2026-09-02
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: Claude Opus 5
//! TIMESTAMP: 2026-09-02 19:14:58
//! REASON FOR CREATION: Granica HTTP dla stanu repozytorium Darkstar - wylacznie odczyt.
//! Domyka kontrakt zapisany wczesniej w crates/darkstar-server/tests/git_http.rs.
//! MECHANICS: Wystawia jedna sciezke `GET /v1/git/overview`. Szesc polecen gita, wszystkie
//! czytajace, idzie przez trait GitRunner - dzieki temu test integracyjny nie potrzebuje
//! ani repozytorium, ani hosta. Odpowiedz laczy galaz, HEAD, brud roboczy, dystans do
//! upstreamu i najwyzej dziesiec ostatnich commitow. Limit dziesieciu jest wymuszany tutaj,
//! a nie tylko przez --max-count, wiec nadmiarowe wejscie i tak zostanie przyciete.
//! Blad gita wraca jako 503 z golym kodem `git_unavailable` - bez tresci polecenia, bo
//! komunikaty gita niosa sciezki hosta i nazwy zdalnych.
//! SYSTEM PART: Darkstar Server / HTTP boundary - warstwa repozytorium.
//! ARCHITECTURE FUNCTION: Jedyne wyjscie stanu gita na zewnatrz. Router rejestruje wylacznie
//! GET, wiec POST, PUT, DELETE i PATCH koncza sie odmowa metody zamiast mutacji. Sterowania
//! repozytorium tu nie ma i nie wolno go dolozyc - to osobna zdolnosc z osobna autoryzacja.
//! DEPENDENCIES/LINKS: axum 0.8, serde_json, std::process::Command; binarka `git` na hoscie.
//! Stan tego routera jest osobny od AppState, zeby nie ruszac dzialajacych sciezek i ich testow.
//! TECH STACK: Rust 2024 + Axum 0.8. Wykonanie polecen wstrzykiwane przez trait, tak samo jak
//! w gateway_http.rs - spojnie z granica, ktora juz stoi.
//! LOCAL WORKSPACE: worktrees/polip-agi-darkstar-plan
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
//! ==========================================

use std::{fmt, path::PathBuf, process::Command, sync::Arc};

use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
};

/// Ile commitow wolno oddac. Wymuszane po stronie serwera, nie tylko w argumencie gita.
const COMMIT_LIMIT: usize = 10;

/// Separatory rekordu i pola w `--pretty=format`. Gita nie da sie zmusic do JSON-a, wiec
/// rozdzielamy znakami, ktorych tresc commita nie zawiera: 0x1f pole, 0x1e rekord.
const FIELD_SEPARATOR: char = '\u{1f}';
const RECORD_SEPARATOR: char = '\u{1e}';

/// Blad odczytu. Tresc sluzy wylacznie logom - nigdy nie trafia do odpowiedzi HTTP.
#[derive(Debug)]
pub struct GitReadError(String);

impl GitReadError {
    pub fn new(detail: impl Into<String>) -> Self {
        Self(detail.into())
    }
}

impl fmt::Display for GitReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Wykonawca polecen gita. Kazde wywolanie musi byc odczytem - implementacja produkcyjna
/// nie ma zadnej sciezki, ktora moglaby zmienic repozytorium.
pub trait GitRunner {
    fn run(&self, args: &[&str]) -> Result<String, GitReadError>;
}

#[derive(Clone)]
pub struct GitState {
    pub api_token: Option<Arc<str>>,
    pub runner: Arc<dyn GitRunner + Send + Sync>,
}

impl GitState {
    pub fn new(api_token: Option<Arc<str>>, runner: Arc<dyn GitRunner + Send + Sync>) -> Self {
        Self { api_token, runner }
    }
}

/// Produkcyjny wykonawca: uruchamia `git` w zadanym worktree.
///
/// Katalog idzie przez `current_dir`, a nie przez argument `-C`, zeby lista argumentow
/// pozostala dokladnie taka, jaka opisuje kontrakt w tescie.
pub struct SystemGitRunner {
    worktree: PathBuf,
}

impl SystemGitRunner {
    pub fn new(worktree: impl Into<PathBuf>) -> Self {
        Self {
            worktree: worktree.into(),
        }
    }
}

impl GitRunner for SystemGitRunner {
    fn run(&self, args: &[&str]) -> Result<String, GitReadError> {
        let out = Command::new("git")
            .current_dir(&self.worktree)
            .args(args)
            .output()
            .map_err(|e| GitReadError::new(e.to_string()))?;
        if !out.status.success() {
            return Err(GitReadError::new(format!(
                "git {} exited with {}",
                args.join(" "),
                out.status
            )));
        }
        String::from_utf8(out.stdout).map_err(|e| GitReadError::new(e.to_string()))
    }
}

fn authenticated(state: &GitState, headers: &HeaderMap) -> bool {
    // Fail-closed, tak samo jak brama: brak skonfigurowanego tokenu to brak dostepu.
    let Some(expected) = state.api_token.as_deref() else {
        return false;
    };
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|token| token == expected)
        .unwrap_or(false)
}

/// `2\t1` -> (2, 1). Cokolwiek innego traktujemy jak brak wiedzy, a nie jak pewne zero-zero.
fn parse_ahead_behind(raw: &str) -> Option<(u32, u32)> {
    let mut parts = raw.split_whitespace();
    let ahead = parts.next()?.parse().ok()?;
    let behind = parts.next()?.parse().ok()?;
    Some((ahead, behind))
}

fn parse_commits(raw: &str) -> Vec<serde_json::Value> {
    raw.split(RECORD_SEPARATOR)
        .map(str::trim)
        .filter(|record| !record.is_empty())
        .take(COMMIT_LIMIT)
        .map(|record| {
            let mut fields = record.split(FIELD_SEPARATOR);
            let hash = fields.next().unwrap_or_default();
            let parents = fields.next().unwrap_or_default();
            let refs = fields.next().unwrap_or_default();
            let subject = fields.next().unwrap_or_default();
            let author = fields.next().unwrap_or_default();
            let date = fields.next().unwrap_or_default();
            serde_json::json!({
                "hash": hash,
                "parents": parents.split_whitespace().collect::<Vec<_>>(),
                "refs": refs
                    .split(", ")
                    .map(str::trim)
                    .filter(|item| !item.is_empty())
                    .collect::<Vec<_>>(),
                "subject": subject,
                "author": author,
                "date": date,
            })
        })
        .collect()
}

/// Zbiera obraz repozytorium. HEAD i status sa obowiazkowe - bez nich nie ma o czym mowic.
/// Galaz moze nie istniec (detached HEAD), a upstream moze nie byc ustawiony; oba przypadki
/// sa normalnym stanem repozytorium, wiec nie wywracaja calej odpowiedzi.
fn read_overview(runner: &dyn GitRunner) -> Result<serde_json::Value, GitReadError> {
    let head = runner.run(&["rev-parse", "HEAD"])?.trim().to_string();
    let status = runner.run(&["status", "--porcelain"])?;

    let branch = runner
        .run(&["symbolic-ref", "--short", "HEAD"])
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    let upstream = runner
        .run(&["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"])
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    let (ahead, behind) = match upstream {
        Some(_) => runner
            .run(&["rev-list", "--left-right", "--count", "HEAD...@{u}"])
            .ok()
            .and_then(|raw| parse_ahead_behind(&raw))
            .unwrap_or((0, 0)),
        None => (0, 0),
    };

    let commits = parse_commits(&runner.run(&[
        "log",
        "--all",
        "--topo-order",
        "--max-count=10",
        "--pretty=format:%H%x1f%P%x1f%D%x1f%s%x1f%an%x1f%aI%x1e",
    ])?);

    Ok(serde_json::json!({
        "branch": branch,
        "head": head,
        "dirty": !status.trim().is_empty(),
        "ahead": ahead,
        "behind": behind,
        "hasUpstream": upstream.is_some(),
        "upstream": upstream,
        "commits": commits,
    }))
}

async fn git_overview(State(state): State<GitState>, headers: HeaderMap) -> impl IntoResponse {
    if !authenticated(&state, &headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "authentication_required" })),
        )
            .into_response();
    }

    match read_overview(state.runner.as_ref()) {
        Ok(overview) => (StatusCode::OK, Json(overview)).into_response(),
        Err(err) => {
            // Powod zostaje w logu serwera. Klient dostaje sam kod, bo tresc bledu gita
            // zawiera sciezki hosta i nazwy zdalnych.
            tracing::warn!(error = %err, "git overview unreadable");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({ "error": "git_unavailable" })),
            )
                .into_response()
        }
    }
}

/// Wylacznie GET. Zadna metoda mutujaca nie jest tu zarejestrowana i nie wolno jej dodac.
pub fn git_router(state: GitState) -> Router {
    Router::new()
        .route("/v1/git/overview", get(git_overview))
        .with_state(state)
}
