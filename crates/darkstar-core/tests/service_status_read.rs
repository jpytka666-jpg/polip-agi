// darkstar-header-v1
// po co: service_status_read.rs
// nie wolno: start/stop/restart/enable/disable uslugi, mutacja hosta, zmiana domyslnego providera
// autor: Marcin
// powstal: 2026-09-03
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: Claude Opus 5
//! TIMESTAMP: 2026-09-03 22:15:00
//! REASON FOR CREATION: Kontrakt fazy czerwonej dla JEDNEJ operacji tylko do odczytu - stanu
//! uslugi systemd `darkstar-chroma-e-copy` razem z pulsem HTTP na 8001. Test powstaje przed
//! implementacja, zeby zapisac oczekiwane zachowanie, zanim ktokolwiek napisze kod.
//! MECHANICS: Sonda jest wstrzykiwana przez trait i zapisuje kazde wywolanie, wiec test
//! sprawdza nie tylko wynik, ale i to, ze sciezka odczytu nie wydala zadnego polecenia
//! zmieniajacego. Zadnego systemd, zadnej sieci, zadnego hosta - wylacznie utrwalone wyjscie
//! zmierzone na CBMS 2026-09-03.
//! SYSTEM PART: Darkstar Core / Module Control - testy granicy odczytu stanu uslugi.
//! ARCHITECTURE FUNCTION: Utrwala regule fail-closed: usluga aktywna, ktora przestala
//! odpowiadac na 8001, NIE moze byc raportowana jako dzialajaca, a nierozpoznane wyjscie
//! `systemctl` jest bledem, nie zgadywaniem.
//! DEPENDENCIES/LINKS: darkstar_core::service_status (jeszcze nie istnieje - stad czerwien),
//! darkstar_core::module_state::ModuleState.
//! TECH STACK: Rust 2024, test integracyjny bez nowych zaleznosci; wzorzec sondy wstrzykiwanej
//! przez trait zgodny z gateway_provider::CommandRunner i context_client::ContextTransport,
//! ktore juz stoja w tym crate.
//! LOCAL WORKSPACE: D:\codex-fresh-2026-08-28\worktrees\polip-agi-darkstar-plan
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
//! ==========================================

use std::cell::RefCell;

use darkstar_core::module_state::ModuleState;
use darkstar_core::service_status::{
    ServiceProbe, ServiceProbeError, ServiceStatusProvider, UnitActivity,
};

/// Utrwalone wyjscie pulsu Chromy zmierzone na CBMS 2026-09-03 pod
/// `http://127.0.0.1:8001/api/v2/heartbeat` (HTTP 200).
const HEARTBEAT_BODY: &str = r#"{"nanosecond heartbeat":1788471102000000000}"#;

/// Sonda testowa: zwraca utrwalone wyjscie i zapisuje kazde wywolanie,
/// zeby test mogl udowodnic, ze nic poza odczytem sie nie wydarzylo.
struct RecordingProbe {
    is_active_stdout: Result<String, ServiceProbeError>,
    heartbeat: Result<String, ServiceProbeError>,
    calls: RefCell<Vec<String>>,
}

impl RecordingProbe {
    fn new(is_active_stdout: &str, heartbeat: Result<String, ServiceProbeError>) -> Self {
        Self {
            is_active_stdout: Ok(is_active_stdout.to_string()),
            heartbeat,
            calls: RefCell::new(Vec::new()),
        }
    }

    fn calls(&self) -> Vec<String> {
        self.calls.borrow().clone()
    }
}

impl ServiceProbe for RecordingProbe {
    fn unit_is_active(&self, unit: &str) -> Result<String, ServiceProbeError> {
        self.calls
            .borrow_mut()
            .push(format!("systemctl is-active {unit}"));
        self.is_active_stdout.clone()
    }

    fn heartbeat(&self, url: &str) -> Result<String, ServiceProbeError> {
        self.calls.borrow_mut().push(format!("GET {url}"));
        self.heartbeat.clone()
    }
}

fn unreachable() -> Result<String, ServiceProbeError> {
    Err(ServiceProbeError::ProbeFailed(
        "127.0.0.1:8001: connection refused".into(),
    ))
}

#[test]
fn active_unit_answering_on_8001_reads_as_running() {
    let probe = RecordingProbe::new("active\n", Ok(HEARTBEAT_BODY.into()));
    let report = ServiceStatusProvider::chroma_e_copy(probe)
        .read_status()
        .expect("zywa usluga z pulsem musi dac raport");

    assert_eq!(report.module_id, "darkstar-chroma-e-copy");
    assert_eq!(report.unit, "darkstar-chroma-e-copy.service");
    assert_eq!(report.activity, UnitActivity::Active);
    assert!(report.heartbeat_ok);
    assert_eq!(report.state, ModuleState::Running);
}

#[test]
fn active_unit_that_stopped_answering_is_not_reported_as_running() {
    // Fail-closed: systemd mowi "active", ale kontrakt tej uslugi to odpowiadanie na 8001.
    // Milczenie pulsu nie moze wygladac jak zdrowie.
    let probe = RecordingProbe::new("active\n", unreachable());
    let report = ServiceStatusProvider::chroma_e_copy(probe)
        .read_status()
        .expect("brak pulsu to nadal raport, nie blad odczytu");

    assert_eq!(report.activity, UnitActivity::Active);
    assert!(!report.heartbeat_ok);
    assert_ne!(report.state, ModuleState::Running);
    assert_eq!(report.state, ModuleState::Failed);
}

#[test]
fn inactive_unit_reads_as_offline_and_puls_is_not_probed() {
    let probe = RecordingProbe::new("inactive\n", Ok(HEARTBEAT_BODY.into()));
    let provider = ServiceStatusProvider::chroma_e_copy(probe);
    let report = provider
        .read_status()
        .expect("martwa usluga to poprawny odczyt");

    assert_eq!(report.activity, UnitActivity::Inactive);
    assert!(!report.heartbeat_ok);
    assert_eq!(report.state, ModuleState::Offline);
    assert_eq!(
        provider.probe().calls(),
        vec!["systemctl is-active darkstar-chroma-e-copy.service".to_string()],
        "gdy jednostka nie jest aktywna, nie ma po co pukac na 8001"
    );
}

#[test]
fn activating_unit_reads_as_starting() {
    let probe = RecordingProbe::new("activating\n", unreachable());
    let report = ServiceStatusProvider::chroma_e_copy(probe)
        .read_status()
        .expect("jednostka w rozruchu to poprawny odczyt");

    assert_eq!(report.activity, UnitActivity::Activating);
    assert_eq!(report.state, ModuleState::Starting);
}

#[test]
fn failed_unit_reads_as_failed() {
    let probe = RecordingProbe::new("failed\n", unreachable());
    let report = ServiceStatusProvider::chroma_e_copy(probe)
        .read_status()
        .expect("jednostka failed to poprawny odczyt");

    assert_eq!(report.activity, UnitActivity::Failed);
    assert_eq!(report.state, ModuleState::Failed);
}

#[test]
fn unrecognised_systemctl_output_is_an_error_not_a_guess() {
    let probe = RecordingProbe::new("nieznane-slowo\n", Ok(HEARTBEAT_BODY.into()));
    let error = ServiceStatusProvider::chroma_e_copy(probe)
        .read_status()
        .expect_err("nierozpoznane wyjscie nie moze udawac stanu");

    assert_eq!(
        error,
        ServiceProbeError::Unparseable("nieznane-slowo".into())
    );
}

#[test]
fn status_read_issues_only_read_commands() {
    let probe = RecordingProbe::new("active\n", Ok(HEARTBEAT_BODY.into()));
    let provider = ServiceStatusProvider::chroma_e_copy(probe);
    provider.read_status().expect("odczyt musi sie udac");

    let calls = provider.probe().calls();
    assert_eq!(
        calls,
        vec![
            "systemctl is-active darkstar-chroma-e-copy.service".to_string(),
            "GET http://127.0.0.1:8001/api/v2/heartbeat".to_string(),
        ]
    );

    for forbidden in ["start", "stop", "restart", "enable", "disable"] {
        assert!(
            !calls.iter().any(|call| call.contains(forbidden)),
            "sciezka odczytu wydala polecenie zmieniajace: {forbidden}"
        );
    }
}

#[test]
fn chroma_e_copy_preset_targets_the_unit_and_endpoint_measured_on_cbms() {
    let provider = ServiceStatusProvider::chroma_e_copy(RecordingProbe::new(
        "active\n",
        Ok(HEARTBEAT_BODY.into()),
    ));

    assert_eq!(provider.unit(), "darkstar-chroma-e-copy.service");
    assert_eq!(
        provider.heartbeat_url(),
        "http://127.0.0.1:8001/api/v2/heartbeat"
    );
}
