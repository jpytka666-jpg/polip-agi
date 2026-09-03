// darkstar-header-v1
// po co: context_client.rs
// nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie
// autor: Marcin
// powstal: 2026-09-01
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: Claude Opus 5
//! TIMESTAMP: 2026-09-02 01:10:00
//! REASON FOR CREATION: Klient kontekstu AIONS z dwiema nogami - zdalna na E: i lokalna na CBMS.
//! MECHANICS: `ContextTransport` wykonuje wylacznie zapytania odczytu. Klient probuje najpierw
//! nogi preferowanej, a przy jej awarii siega po zapasowa; gdy obie milcza, zwraca blad zamiast
//! pustej listy - brak odpowiedzi nie moze wygladac jak brak wynikow. Nie ma tu ani jednej
//! sciezki zapisu, wiec baza na E: jest nietykalna z definicji, a nie z dobrej woli.
//! SYSTEM PART: Darkstar Core / warstwa kontekstu.
//! ARCHITECTURE FUNCTION: Pozwala Darkstarowi czytac pamiec AIONS, zanim dane przenios sie na
//! CBMS, i dziala dalej po przeniesieniu - zmienia sie tylko kolejnosc nog.
//! DEPENDENCIES/LINKS: Chroma HTTP API v1 (`/api/v1/heartbeat`, `/api/v1/collections`);
//! zywe zrodlo `E:\server wiedzy\data\chroma`; przyszla noga `/var/lib/darkstar/chroma`.
//! TECH STACK: Rust 2024 + serde, bez nowych zaleznosci. Transport jest wstrzykiwany traitem,
//! wiec testy nie potrzebuja ani sieci, ani zywej bazy.
//! LOCAL WORKSPACE: D:\codex-fresh-2026-08-28\worktrees\polip-agi-darkstar-plan
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
//! ==========================================

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ContextError {
    #[error("leg unreachable: {0}")]
    Unreachable(String),
    #[error("both context legs are unreachable")]
    AllLegsUnreachable,
    #[error("malformed response: {0}")]
    Malformed(String),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContextLegKind {
    /// Zywe zrodlo prawdy na E: - dzis jedyne z danymi.
    RemoteE,
    /// Druga noga na CBMS - docelowa, dzis jeszcze pusta.
    LocalCbms,
}

/// Wersja HTTP API Chromy. Zrodlo na E: pracuje na starszej wersji i mowi `v1`;
/// chromadb 1.5.9 na CBMS odpowiada na `v1` kodem 410 i wymaga `v2`, w ktorej
/// kolekcje leza pod sciezka z tenantem i baza.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChromaApi {
    V1,
    V2,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContextLeg {
    pub kind: ContextLegKind,
    pub base_url: String,
    pub api_version: ChromaApi,
    pub tenant: String,
    pub database: String,
}

impl ContextLeg {
    pub fn remote_e(base_url: &str) -> Self {
        Self {
            kind: ContextLegKind::RemoteE,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_version: ChromaApi::V1,
            tenant: "default_tenant".into(),
            database: "default_database".into(),
        }
    }

    pub fn local_cbms(base_url: &str) -> Self {
        Self {
            kind: ContextLegKind::LocalCbms,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_version: ChromaApi::V2,
            tenant: "default_tenant".into(),
            database: "default_database".into(),
        }
    }

    pub fn heartbeat_url(&self) -> String {
        match self.api_version {
            ChromaApi::V1 => format!("{}/api/v1/heartbeat", self.base_url),
            ChromaApi::V2 => format!("{}/api/v2/heartbeat", self.base_url),
        }
    }

    pub fn collections_url(&self) -> String {
        match self.api_version {
            ChromaApi::V1 => format!("{}/api/v1/collections", self.base_url),
            ChromaApi::V2 => format!(
                "{}/api/v2/tenants/{}/databases/{}/collections",
                self.base_url, self.tenant, self.database
            ),
        }
    }
}

/// Wylacznie odczyt. Nie ma tu metody wysylajacej cokolwiek do bazy,
/// wiec zrodlo na E: nie da sie zmodyfikowac przez ta warstwe.
pub trait ContextTransport {
    fn get(&self, url: &str) -> Result<String, ContextError>;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContextCollection {
    pub id: String,
    pub name: String,
    pub dimension: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContextListing {
    pub served_by: ContextLegKind,
    pub collections: Vec<ContextCollection>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContextHealth {
    pub remote_e_ok: bool,
    pub local_cbms_ok: bool,
}

impl ContextHealth {
    pub fn any_ok(&self) -> bool {
        self.remote_e_ok || self.local_cbms_ok
    }
}

pub struct ContextClient<T: ContextTransport> {
    transport: T,
    preferred: ContextLeg,
    fallback: ContextLeg,
}

impl<T: ContextTransport> ContextClient<T> {
    pub fn new(transport: T, preferred: ContextLeg, fallback: ContextLeg) -> Self {
        Self {
            transport,
            preferred,
            fallback,
        }
    }

    pub fn transport(&self) -> &T {
        &self.transport
    }

    fn parse_collections(payload: &str) -> Result<Vec<ContextCollection>, ContextError> {
        serde_json::from_str::<Vec<ContextCollection>>(payload)
            .map_err(|e| ContextError::Malformed(e.to_string()))
    }

    /// Probuje nogi preferowanej, potem zapasowej. Milczenie obu to blad,
    /// nigdy pusta lista - inaczej awaria wygladalaby jak brak wiedzy.
    fn list_from_any_leg(&self) -> Result<ContextListing, ContextError> {
        for leg in [&self.preferred, &self.fallback] {
            let url = leg.collections_url();
            if let Ok(body) = self.transport.get(&url) {
                return Ok(ContextListing {
                    served_by: leg.kind,
                    collections: Self::parse_collections(&body)?,
                });
            }
        }
        Err(ContextError::AllLegsUnreachable)
    }

    pub fn list(&self) -> Result<ContextListing, ContextError> {
        self.list_from_any_leg()
    }

    /// Dopasowanie po nazwie kolekcji, bez rozroznienia wielkosci liter.
    /// Pelne wyszukiwanie wektorowe wymaga zapytania z embeddingiem i przyjdzie osobno.
    pub fn search(&self, query: &str, limit: usize) -> Result<ContextListing, ContextError> {
        let listing = self.list_from_any_leg()?;
        let needle = query.to_lowercase();
        let collections = listing
            .collections
            .into_iter()
            .filter(|c| c.name.to_lowercase().contains(&needle))
            .take(limit)
            .collect();
        Ok(ContextListing {
            served_by: listing.served_by,
            collections,
        })
    }

    pub fn health(&self) -> ContextHealth {
        let mut health = ContextHealth {
            remote_e_ok: false,
            local_cbms_ok: false,
        };
        for leg in [&self.preferred, &self.fallback] {
            let url = leg.heartbeat_url();
            let ok = self.transport.get(&url).is_ok();
            match leg.kind {
                ContextLegKind::RemoteE => health.remote_e_ok = ok,
                ContextLegKind::LocalCbms => health.local_cbms_ok = ok,
            }
        }
        health
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    /// Utrwalone wyjscie zywej bazy z E:, odczytane 2026-09-01.
    const COLLECTIONS_JSON: &str = r#"[
      {"id":"0b45be0a-3ad5-403a-9fb1-36c74170c0bc","name":"session_AIONS_ARCHITECTURE_MASTERPLAN","dimension":384},
      {"id":"0d20b0b4-9a0b-4aa7-8076-f8773616ec49","name":"session_aions_skills_v4_diag2","dimension":384}
    ]"#;

    struct FakeTransport {
        remote_ok: bool,
        local_ok: bool,
        calls: RefCell<Vec<String>>,
    }

    impl FakeTransport {
        fn new(remote_ok: bool, local_ok: bool) -> Self {
            Self {
                remote_ok,
                local_ok,
                calls: RefCell::new(Vec::new()),
            }
        }
    }

    impl ContextTransport for FakeTransport {
        fn get(&self, url: &str) -> Result<String, ContextError> {
            self.calls.borrow_mut().push(url.to_string());
            let healthy = if url.contains("192.168.2.1") || url.contains("127.0.0.1:9000") {
                self.local_ok
            } else {
                self.remote_ok
            };
            if !healthy {
                return Err(ContextError::Unreachable(url.to_string()));
            }
            if url.ends_with("/heartbeat") {
                Ok(r#"{"nanosecond heartbeat":1788298934144612400}"#.into())
            } else {
                Ok(COLLECTIONS_JSON.into())
            }
        }
    }

    fn client(remote_ok: bool, local_ok: bool) -> ContextClient<FakeTransport> {
        ContextClient::new(
            FakeTransport::new(remote_ok, local_ok),
            ContextLeg::remote_e("http://100.71.8.70:8000"),
            ContextLeg::local_cbms("http://192.168.2.1:8000"),
        )
    }

    #[test]
    fn lists_collections_from_the_preferred_leg() {
        let c = client(true, true);
        let listed = c.list().unwrap();
        assert_eq!(listed.served_by, ContextLegKind::RemoteE);
        assert_eq!(listed.collections.len(), 2);
        assert_eq!(
            listed.collections[0].name,
            "session_AIONS_ARCHITECTURE_MASTERPLAN"
        );
        assert_eq!(listed.collections[0].dimension, Some(384));
    }

    #[test]
    fn falls_back_to_the_local_leg_when_e_is_silent() {
        let c = client(false, true);
        let listed = c.list().unwrap();
        assert_eq!(listed.served_by, ContextLegKind::LocalCbms);
        assert_eq!(listed.collections.len(), 2);
    }

    /// Kolejnosc jak w SERWERZE, czyli odwrotna do `client()` powyzej: noga lokalna jest
    /// pierwsza, a udzial z E jest zapasem. Pozostale testy sprawdzaly wylacznie uklad
    /// odwrotny, wiec ten kierunek przelaczenia - ten, ktory faktycznie dziala w
    /// produkcji - nie byl pokryty niczym.
    #[test]
    fn falls_back_to_the_share_when_the_local_leg_is_silent() {
        let c = ContextClient::new(
            // local_ok = false: noga lokalna milczy. remote_ok = true: udzial odpowiada.
            FakeTransport::new(true, false),
            ContextLeg::local_cbms("http://127.0.0.1:9000"),
            ContextLeg::remote_e("http://127.0.0.1:8001"),
        );

        let listed = c.list().unwrap();
        assert_eq!(listed.served_by, ContextLegKind::RemoteE);
        assert_eq!(listed.collections.len(), 2);

        // Zdrowie ma pokazac to samo rozroznienie, a nie jedno zbiorcze "dziala".
        let health = c.health();
        assert!(!health.local_cbms_ok, "noga lokalna milczy");
        assert!(health.remote_e_ok, "udzial odpowiada");
        assert!(health.any_ok());
    }

    #[test]
    fn fails_closed_when_both_legs_are_silent() {
        let c = client(false, false);
        match c.list() {
            Err(ContextError::AllLegsUnreachable) => {}
            other => panic!("oczekiwano AllLegsUnreachable, dostano {other:?}"),
        }
    }

    #[test]
    fn search_filters_collection_names_without_touching_the_store() {
        let c = client(true, true);
        let found = c.search("skills", 10).unwrap();
        assert_eq!(found.collections.len(), 1);
        assert_eq!(found.collections[0].name, "session_aions_skills_v4_diag2");

        // Zadne z wywolan nie moze byc zapisem.
        for url in c.transport().calls.borrow().iter() {
            assert!(url.contains("/api/v1/"), "nieoczekiwany adres: {url}");
            assert!(!url.contains("/add"), "wykryto sciezke zapisu: {url}");
        }
    }

    #[test]
    fn limit_is_respected() {
        let c = client(true, true);
        assert_eq!(c.search("session", 1).unwrap().collections.len(), 1);
        assert_eq!(c.search("session", 10).unwrap().collections.len(), 2);
    }

    #[test]
    fn each_leg_uses_the_api_version_its_server_speaks() {
        // Zywe zrodlo na E: mowi v1. Noga na CBMS to chromadb 1.5.9 - v1 zwraca tam 410,
        // a kolekcje leza pod sciezka z tenantem i baza.
        let e = ContextLeg::remote_e("http://100.71.8.70:8000");
        assert_eq!(e.api_version, ChromaApi::V1);
        assert_eq!(
            e.collections_url(),
            "http://100.71.8.70:8000/api/v1/collections"
        );
        assert_eq!(
            e.heartbeat_url(),
            "http://100.71.8.70:8000/api/v1/heartbeat"
        );

        let cbms = ContextLeg::local_cbms("http://127.0.0.1:8000");
        assert_eq!(cbms.api_version, ChromaApi::V2);
        assert_eq!(
            cbms.collections_url(),
            "http://127.0.0.1:8000/api/v2/tenants/default_tenant/databases/default_database/collections"
        );
        assert_eq!(
            cbms.heartbeat_url(),
            "http://127.0.0.1:8000/api/v2/heartbeat"
        );
    }

    #[test]
    fn health_reports_each_leg_separately() {
        let c = client(true, false);
        let health = c.health();
        assert!(health.remote_e_ok);
        assert!(!health.local_cbms_ok);
        assert!(health.any_ok());
    }
}
