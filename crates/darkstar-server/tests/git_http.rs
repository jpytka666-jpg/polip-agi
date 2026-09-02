// darkstar-header-v1
// po co: git_http.rs
// nie wolno: wykonywac fetch, checkout, merge, reset ani zadnej innej mutacji repozytorium
// autor: Marcin
// powstal: 2026-09-02
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: GPT-5 Codex
//! TIMESTAMP: 2026-09-02 18:40:37 Europe/London
//! REASON FOR CREATION: Lock the authenticated, read-only GET /v1/git/overview contract before implementing the server-side Git boundary.
//! ==========================================

use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::Value;
use tower::ServiceExt;

#[allow(dead_code)]
#[path = "../src/git_http.rs"]
mod git_http;

use git_http::{GitReadError, GitRunner, GitState, git_router};

struct FixtureRunner;

impl GitRunner for FixtureRunner {
    fn run(&self, args: &[&str]) -> Result<String, GitReadError> {
        match args {
            ["symbolic-ref", "--short", "HEAD"] => Ok("docs/example\n".into()),
            ["rev-parse", "HEAD"] => {
                Ok("0123456789abcdef0123456789abcdef01234567\n".into())
            }
            ["status", "--porcelain"] => Ok(" M frontend/src/GitPanel.tsx\n".into()),
            ["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"] => {
                Ok("origin/docs/example\n".into())
            }
            ["rev-list", "--left-right", "--count", "HEAD...@{u}"] => Ok("2\t1\n".into()),
            [
                "log",
                "--all",
                "--topo-order",
                "--max-count=10",
                "--pretty=format:%H%x1f%P%x1f%D%x1f%s%x1f%an%x1f%aI%x1e",
            ] => Ok((0..11)
                .map(|index| {
                    format!(
                        "{index:040x}\x1f\x1fHEAD -> docs/example\x1fcommit {index}\x1fOperator\x1f2026-09-02T18:00:00+01:00\x1e"
                    )
                })
                .collect()),
            _ => Err(GitReadError::new("unexpected test command")),
        }
    }
}

fn state(runner: Arc<dyn GitRunner + Send + Sync>) -> GitState {
    GitState::new(Some(Arc::from("secret")), runner)
}

#[tokio::test]
async fn overview_returns_structured_branch_state_and_ten_commits() {
    let response = git_router(state(Arc::new(FixtureRunner)))
        .oneshot(
            Request::builder()
                .uri("/v1/git/overview")
                .header("authorization", "Bearer secret")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["branch"], "docs/example");
    assert_eq!(json["head"], "0123456789abcdef0123456789abcdef01234567");
    assert_eq!(json["dirty"], true);
    assert_eq!(json["ahead"], 2);
    assert_eq!(json["behind"], 1);
    assert_eq!(json["hasUpstream"], true);
    assert_eq!(json["commits"].as_array().unwrap().len(), 10);
    assert_eq!(json["commits"][0]["subject"], "commit 0");
}

#[tokio::test]
async fn overview_requires_the_operator_token() {
    let response = git_router(state(Arc::new(FixtureRunner)))
        .oneshot(
            Request::builder()
                .uri("/v1/git/overview")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn mutating_methods_are_not_registered() {
    for method in ["POST", "PUT", "DELETE", "PATCH"] {
        let response = git_router(state(Arc::new(FixtureRunner)))
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri("/v1/git/overview")
                    .header("authorization", "Bearer secret")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }
}

#[tokio::test]
async fn missing_git_reports_service_unavailable_without_raw_command_output() {
    struct MissingGit;
    impl GitRunner for MissingGit {
        fn run(&self, _args: &[&str]) -> Result<String, GitReadError> {
            Err(GitReadError::new("secret host path and command failure"))
        }
    }

    let response = git_router(state(Arc::new(MissingGit)))
        .oneshot(
            Request::builder()
                .uri("/v1/git/overview")
                .header("authorization", "Bearer secret")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json, serde_json::json!({ "error": "git_unavailable" }));
}
