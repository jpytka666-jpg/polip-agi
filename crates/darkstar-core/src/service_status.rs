// darkstar-header-v1
// po co: service_status.rs
// nie wolno: start/stop/restart/enable/disable uslugi, mutacja hosta, zmiana domyslnego providera
// autor: Marcin
// powstal: 2026-09-03
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: Claude Opus 5
//! TIMESTAMP: 2026-09-03 22:25:00
//! REASON FOR CREATION: Jedna operacja tylko do odczytu - stan uslugi systemd
//! `darkstar-chroma-e-copy` razem z pulsem HTTP na 8001. Najmniejszy krok od granicy v1, ktora
//! dzis niczego nie wykonuje (module_control_http.rs), do granicy, ktora potrafi COS ZOBACZYC.
//! MECHANICS: `ServiceProbe` udostepnia dokladnie dwa odczyty - `systemctl is-active <unit>`
//! oraz GET na adres pulsu - i nic wiecej; nie ma w tym traicie miejsca na start, stop ani
//! restart. Rozbior wyjscia jest czysta funkcja, wiec testuje sie go na utrwalonym wyjsciu
//! hosta CBMS bez dotykania systemd i bez sieci. Gdy jednostka nie jest aktywna, puls nie jest
//! nawet pytany. Regula fail-closed: jednostka aktywna, ktora milczy na 8001, to `Failed`, a
//! nierozpoznane wyjscie `systemctl` to blad, nie zgadywanie.
//! SYSTEM PART: Darkstar Core / Module Control - granica odczytu stanu uslugi.
//! ARCHITECTURE FUNCTION: Ten modul CELOWO nie implementuje `ModuleProvider`. Dzieki temu
//! zadna komenda cyklu zycia nie moze zostac tu skierowana, a `DryRunProvider` pozostaje
//! jedyna i domyslna implementacja `ModuleProvider`. Warstwa mutujaca - jesli kiedys powstanie -
//! bedzie osobna, z wlasna autoryzacja zdolnosci.
//! DEPENDENCIES/LINKS: module_state (ModuleState), context_client (ContextTransport - tylko przy
//! wlaczonej fladze `systemd-read`); `systemctl` na hoscie CBMS, Chroma pod
//! http://127.0.0.1:8001/api/v2/heartbeat.
//! TECH STACK: Rust 2024 + serde, bez nowych zaleznosci. Sonda wstrzykiwana przez trait, tak jak
//! gateway_provider::CommandRunner i context_client::ContextTransport, wiec testy nie potrzebuja
//! ani hosta, ani SSH. Sonda dotykajaca hosta stoi za nieaktywna domyslnie cecha `systemd-read`.
//! LOCAL WORKSPACE: D:\codex-fresh-2026-08-28\worktrees\polip-agi-darkstar-plan
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
//! ==========================================

use serde::{Deserialize, Serialize};

use crate::module_state::ModuleState;

/// Domyslny cel tej sciezki: kopia bazy wiedzy z dysku E, chodzaca na CBMS jako usluga systemd.
pub const CHROMA_E_COPY_MODULE_ID: &str = "darkstar-chroma-e-copy";
pub const CHROMA_E_COPY_UNIT: &str = "darkstar-chroma-e-copy.service";
/// Zmierzone 2026-09-03: HTTP 200 pod tym adresem. `v2`, bo chromadb 1.5.9 odpowiada na `v1`
/// kodem 410 - ten sam wniosek zapisano juz w context_client.rs.
pub const CHROMA_E_COPY_HEARTBEAT_URL: &str = "http://127.0.0.1:8001/api/v2/heartbeat";

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ServiceProbeError {
    #[error("probe failed: {0}")]
    ProbeFailed(String),
    #[error("could not parse host output: {0}")]
    Unparseable(String),
}

/// Waskie okno na hosta. Dwa odczyty, zero mutacji - nie ma tu metody, ktora zmienialaby
/// stan uslugi, i nie wolno takiej dodac. Warstwa mutujaca powstanie osobno, z wlasna
/// autoryzacja zdolnosci.
pub trait ServiceProbe {
    /// `systemctl is-active <unit>` - wylacznie standardowe wyjscie.
    ///
    /// UWAGA: `systemctl is-active` konczy sie kodem 3, gdy jednostka nie jest aktywna,
    /// a mimo to wypisuje na stdout uczciwa odpowiedz `inactive` albo `failed`. Implementacja
    /// NIE moze traktowac niezerowego kodu wyjscia jak bledu odczytu.
    fn unit_is_active(&self, unit: &str) -> Result<String, ServiceProbeError>;

    /// GET na adres pulsu. Bez ciala zadania - nie da sie tym nic zapisac.
    fn heartbeat(&self, url: &str) -> Result<String, ServiceProbeError>;
}

/// Slowa, ktorymi `systemctl is-active` opisuje jednostke.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UnitActivity {
    Active,
    Activating,
    Deactivating,
    Inactive,
    Failed,
}

/// Rozbior wyjscia `systemctl is-active`. Czysta funkcja.
/// Nierozpoznane slowo jest bledem - stan uslugi to nie miejsce na domysly.
pub fn parse_is_active(raw: &str) -> Result<UnitActivity, ServiceProbeError> {
    match raw.trim() {
        "active" => Ok(UnitActivity::Active),
        "activating" | "reloading" => Ok(UnitActivity::Activating),
        "deactivating" => Ok(UnitActivity::Deactivating),
        "inactive" => Ok(UnitActivity::Inactive),
        "failed" => Ok(UnitActivity::Failed),
        other => Err(ServiceProbeError::Unparseable(other.to_string())),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServiceStatusReport {
    pub module_id: String,
    pub unit: String,
    pub activity: UnitActivity,
    pub heartbeat_ok: bool,
    pub state: ModuleState,
}

/// Odczyt stanu jednej uslugi. Konstruktor przyjmuje sonde, wiec test nie potrzebuje hosta.
pub struct ServiceStatusProvider<P: ServiceProbe> {
    probe: P,
    module_id: String,
    unit: String,
    heartbeat_url: String,
}

impl<P: ServiceProbe> ServiceStatusProvider<P> {
    pub fn new(probe: P, module_id: &str, unit: &str, heartbeat_url: &str) -> Self {
        Self {
            probe,
            module_id: module_id.to_string(),
            unit: unit.to_string(),
            heartbeat_url: heartbeat_url.to_string(),
        }
    }

    /// Gotowe nastawienie na usluge zmierzona na CBMS 2026-09-03.
    pub fn chroma_e_copy(probe: P) -> Self {
        Self::new(
            probe,
            CHROMA_E_COPY_MODULE_ID,
            CHROMA_E_COPY_UNIT,
            CHROMA_E_COPY_HEARTBEAT_URL,
        )
    }

    pub fn probe(&self) -> &P {
        &self.probe
    }

    pub fn unit(&self) -> &str {
        &self.unit
    }

    pub fn heartbeat_url(&self) -> &str {
        &self.heartbeat_url
    }

    /// Jedyna operacja tego modulu. Dwa odczyty, w najgorszym razie jeden.
    pub fn read_status(&self) -> Result<ServiceStatusReport, ServiceProbeError> {
        let activity = parse_is_active(&self.probe.unit_is_active(&self.unit)?)?;

        // Puls pytamy tylko wtedy, gdy jednostka twierdzi, ze zyje. Pukanie na port
        // martwej uslugi nic nie wnosi, a kosztuje limit czasu polaczenia.
        let heartbeat_ok = match activity {
            UnitActivity::Active => self.probe.heartbeat(&self.heartbeat_url).is_ok(),
            _ => false,
        };

        let state = match (activity, heartbeat_ok) {
            (UnitActivity::Active, true) => ModuleState::Running,
            // Fail-closed: systemd mowi "active", ale kontrakt tej uslugi to odpowiadanie
            // na 8001. Milczenie pulsu nie moze wygladac jak zdrowie.
            (UnitActivity::Active, false) => ModuleState::Failed,
            (UnitActivity::Activating, _) => ModuleState::Starting,
            (UnitActivity::Deactivating, _) => ModuleState::Stopping,
            (UnitActivity::Inactive, _) => ModuleState::Offline,
            (UnitActivity::Failed, _) => ModuleState::Failed,
        };

        Ok(ServiceStatusReport {
            module_id: self.module_id.clone(),
            unit: self.unit.clone(),
            activity,
            heartbeat_ok,
            state,
        })
    }
}

/// Sonda dotykajaca hosta. Kompiluje sie WYLACZNIE przy jawnym `--features systemd-read`;
/// cecha nie jest domyslna, wiec zwykly build Darkstara nie zawiera nawet tego kodu.
/// Nadal jest to tylko odczyt: `systemctl is-active` i GET, nic wiecej.
#[cfg(feature = "systemd-read")]
pub mod host {
    use std::process::Command;

    use super::{ServiceProbe, ServiceProbeError};
    use crate::context_client::ContextTransport;

    /// Nazwa jednostki musi wygladac jak nazwa jednostki. Odrzucenie tutaj jest tansze
    /// niz tlumaczenie sie potem, dlaczego `systemctl` dostal cudzy argument.
    fn validate_unit(unit: &str) -> Result<(), ServiceProbeError> {
        let shaped = !unit.is_empty()
            && unit.len() <= 128
            && unit
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | '@' | ':'));
        if shaped {
            Ok(())
        } else {
            Err(ServiceProbeError::ProbeFailed(format!(
                "refusing suspicious unit name: {unit}"
            )))
        }
    }

    /// Puls idzie przez ten sam tylko-do-odczytu transport HTTP, ktorego uzywa juz
    /// sciezka kontekstu (`darkstar-server::context_http::ReadOnlyHttp`), wiec nie powstaje
    /// drugi klient HTTP do utrzymania.
    pub struct SystemctlProbe<T: ContextTransport> {
        transport: T,
    }

    impl<T: ContextTransport> SystemctlProbe<T> {
        pub fn new(transport: T) -> Self {
            Self { transport }
        }
    }

    impl<T: ContextTransport> ServiceProbe for SystemctlProbe<T> {
        fn unit_is_active(&self, unit: &str) -> Result<String, ServiceProbeError> {
            validate_unit(unit)?;
            // `--` konczy liste opcji. Jedyny podpolecenie to `is-active`;
            // start, stop, restart, enable i disable nie maja tu wstepu.
            let out = Command::new("systemctl")
                .args(["is-active", "--", unit])
                .output()
                .map_err(|e| ServiceProbeError::ProbeFailed(e.to_string()))?;

            // Kod wyjscia 3 znaczy "nie jest aktywna" i jest poprawna odpowiedzia,
            // a nie awaria odczytu. Liczy sie stdout.
            String::from_utf8(out.stdout).map_err(|e| ServiceProbeError::Unparseable(e.to_string()))
        }

        fn heartbeat(&self, url: &str) -> Result<String, ServiceProbeError> {
            self.transport
                .get(url)
                .map_err(|e| ServiceProbeError::ProbeFailed(e.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_every_word_systemctl_actually_prints() {
        assert_eq!(parse_is_active("active\n"), Ok(UnitActivity::Active));
        assert_eq!(parse_is_active("  active  "), Ok(UnitActivity::Active));
        assert_eq!(parse_is_active("activating"), Ok(UnitActivity::Activating));
        assert_eq!(parse_is_active("reloading"), Ok(UnitActivity::Activating));
        assert_eq!(
            parse_is_active("deactivating"),
            Ok(UnitActivity::Deactivating)
        );
        assert_eq!(parse_is_active("inactive\n"), Ok(UnitActivity::Inactive));
        assert_eq!(parse_is_active("failed\n"), Ok(UnitActivity::Failed));
    }

    #[test]
    fn empty_output_is_an_error_not_an_assumption() {
        assert_eq!(
            parse_is_active("\n"),
            Err(ServiceProbeError::Unparseable(String::new()))
        );
    }

    #[test]
    fn the_read_path_exposes_no_lifecycle_command() {
        // Zapis kontraktu: `ServiceProbe` ma dokladnie dwie metody i obie sa odczytem.
        // Gdyby ktos dolozyl tu start, stop albo restart, ten komentarz i ten test
        // sa miejscem, w ktorym ma sie zatrzymac.
        fn assert_read_only<P: ServiceProbe>(_: &P) {}
        struct Stub;
        impl ServiceProbe for Stub {
            fn unit_is_active(&self, _: &str) -> Result<String, ServiceProbeError> {
                Ok("inactive".into())
            }
            fn heartbeat(&self, _: &str) -> Result<String, ServiceProbeError> {
                Err(ServiceProbeError::ProbeFailed("stub".into()))
            }
        }
        assert_read_only(&Stub);

        let report = ServiceStatusProvider::chroma_e_copy(Stub)
            .read_status()
            .expect("stub reads cleanly");
        assert_eq!(report.state, ModuleState::Offline);
    }
}
