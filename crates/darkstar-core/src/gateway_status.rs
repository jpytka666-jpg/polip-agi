// darkstar-header-v1
// po co: gateway_status.rs
// nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie
// autor: Marcin
// powstal: 2026-09-01
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: Claude Opus 5
//! TIMESTAMP: 2026-09-01 21:05:00
//! REASON FOR CREATION: Typowany kontrakt stanu natywnej bramy Darkstar (Task 7, Step 7.2 i 7.3).
//! MECHANICS: Opisuje tryb pracy, kondycje i adresacje bramy jako typy, a nie luzne napisy, i
//! sprawdza je zanim ktokolwiek ich uzyje. Walidacja odrzuca nazwy interfejsow puste lub
//! wygladajace na sciezke, ten sam interfejs po obu stronach bez zgody preflightu, podsieci
//! spoza zakresow prywatnych, adresy nieokreslone i petle zwrotna, porzucona pule 10.44.0.0/24
//! oraz nasluch na 0.0.0.0. Modul niczego nie uruchamia i nie dotyka systemu.
//! SYSTEM PART: Darkstar Core / Network Boundary - kontrakty warstwy bramy.
//! ARCHITECTURE FUNCTION: Wspolny slownik miedzy dostawca na Ubuntu, API Axum i interfejsem
//! uzytkownika. Bledna adresacja ma zostac odrzucona tutaj, zanim dojdzie do nmcli albo nft.
//! DEPENDENCIES/LINKS: serde; deploy/network/darkstar-gateway-apply, deploy/firewall/host-guard.nft.
//! TECH STACK: Rust 2024 + serde, bez nowych zaleznosci. Rozbior adresu IPv4 realizuje
//! std::net::Ipv4Addr, wiec dokladanie ipnet tylko po to, zeby policzyc maske, byloby kosztem
//! bez zysku; zakresy prywatne sa krotkie i wprost wypisane.
//! LOCAL WORKSPACE: D:\codex-fresh-2026-08-28\worktrees\polip-agi-darkstar-plan
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
//! ==========================================

use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;

/// Pula porzucona 2026-09-01. Nigdy nie istniala na hoscie CBMS; zapora i brama
/// pracuja na 192.168.2.0/24. Odrzucamy ja jawnie, zeby nie wrocila przez pomylke.
const ABANDONED_SUBNET_PREFIX: &str = "10.44.0.";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GatewayMode {
    Ethernet,
    Hotspot,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GatewayHealth {
    Offline,
    Starting,
    Ready,
    Degraded,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum GatewayContractError {
    #[error("interface name is empty or path-like")]
    InvalidInterfaceName,
    #[error("upstream equals downstream without preflight approval")]
    SameInterfaceWithoutApproval,
    #[error("downstream address is malformed")]
    MalformedAddress,
    #[error("downstream subnet is outside the private ranges")]
    NonPrivateSubnet,
    #[error("downstream address is loopback or unspecified")]
    LoopbackOrUnspecifiedAddress,
    #[error("10.44.0.0/24 was abandoned and must not be used")]
    AbandonedSubnet,
    #[error("bind address 0.0.0.0 / :: would expose the service to the upstream segment")]
    UnspecifiedBindAddress,
}

/// Stan bramy w postaci typowanej. Zadne pole nie przenosi sekretow -
/// SSID i haslo trybu hotspot zyja wylacznie w konfiguracji hosta.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GatewayStatus {
    pub mode: GatewayMode,
    pub health: GatewayHealth,
    pub upstream_interface: String,
    pub downstream_interface: String,
    pub downstream_cidr: String,
    pub downstream_subnet: String,
    pub connection_profile: String,
    pub connected_clients: u32,
    pub last_verified_unix_ms: u64,
}

fn interface_name_is_sane(name: &str) -> bool {
    !name.is_empty()
        && !name.contains('/')
        && !name.contains('\\')
        && !name.starts_with('.')
        && name.len() <= 15
}

/// Zakresy prywatne wg RFC 1918. Wypisane wprost - jest ich trzy,
/// wiec dodatkowa zaleznosc nic by tu nie uproscila.
fn is_private_v4(addr: Ipv4Addr) -> bool {
    let o = addr.octets();
    o[0] == 10 || (o[0] == 172 && (16..=31).contains(&o[1])) || (o[0] == 192 && o[1] == 168)
}

fn parse_addr_of(cidr: &str) -> Result<Ipv4Addr, GatewayContractError> {
    cidr.split('/')
        .next()
        .ok_or(GatewayContractError::MalformedAddress)?
        .parse::<Ipv4Addr>()
        .map_err(|_| GatewayContractError::MalformedAddress)
}

/// Nasluch na adresie nieokreslonym wystawilby usluge takze na uplink,
/// czyli do sieci nadrzednej. Dozwolona jest petla zwrotna i adres prywatny.
pub fn validate_bind_address(addr: &str) -> Result<(), GatewayContractError> {
    if addr == "0.0.0.0" || addr == "::" || addr.is_empty() {
        return Err(GatewayContractError::UnspecifiedBindAddress);
    }
    if addr == "::1" {
        return Ok(());
    }
    let parsed = addr
        .parse::<Ipv4Addr>()
        .map_err(|_| GatewayContractError::MalformedAddress)?;
    if parsed.is_loopback() || is_private_v4(parsed) {
        Ok(())
    } else {
        Err(GatewayContractError::NonPrivateSubnet)
    }
}

impl GatewayStatus {
    /// Sprawdzenie domyslne: ten sam interfejs po obu stronach jest bledem.
    pub fn validate(&self) -> Result<(), GatewayContractError> {
        self.validate_with_concurrent_approval(false)
    }

    /// `concurrent_approved` ustawia sie wylacznie wtedy, gdy preflight potwierdzil
    /// zdolnosc karty do pracy jednoczesnej (Task 4, Step 4.4).
    pub fn validate_with_concurrent_approval(
        &self,
        concurrent_approved: bool,
    ) -> Result<(), GatewayContractError> {
        if !interface_name_is_sane(&self.upstream_interface)
            || !interface_name_is_sane(&self.downstream_interface)
        {
            return Err(GatewayContractError::InvalidInterfaceName);
        }
        if self.upstream_interface == self.downstream_interface && !concurrent_approved {
            return Err(GatewayContractError::SameInterfaceWithoutApproval);
        }
        if self.downstream_cidr.starts_with(ABANDONED_SUBNET_PREFIX)
            || self.downstream_subnet.starts_with(ABANDONED_SUBNET_PREFIX)
        {
            return Err(GatewayContractError::AbandonedSubnet);
        }

        let addr = parse_addr_of(&self.downstream_cidr)?;
        if addr.is_loopback() || addr.is_unspecified() {
            return Err(GatewayContractError::LoopbackOrUnspecifiedAddress);
        }
        if !is_private_v4(addr) {
            return Err(GatewayContractError::NonPrivateSubnet);
        }

        let subnet = parse_addr_of(&self.downstream_subnet)?;
        if !is_private_v4(subnet) {
            return Err(GatewayContractError::NonPrivateSubnet);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn live_status() -> GatewayStatus {
        GatewayStatus {
            mode: GatewayMode::Ethernet,
            health: GatewayHealth::Ready,
            upstream_interface: "wlp2s0".into(),
            downstream_interface: "enp1s0".into(),
            downstream_cidr: "192.168.2.1/24".into(),
            downstream_subnet: "192.168.2.0/24".into(),
            connection_profile: "DARKSTAR-WiFi".into(),
            connected_clients: 1,
            last_verified_unix_ms: 1_788_300_000_000,
        }
    }

    #[test]
    fn live_configuration_is_accepted() {
        assert!(live_status().validate().is_ok());
    }

    #[test]
    fn abandoned_subnet_is_rejected() {
        let mut s = live_status();
        s.downstream_cidr = "10.44.0.1/24".into();
        s.downstream_subnet = "10.44.0.0/24".into();
        assert_eq!(s.validate(), Err(GatewayContractError::AbandonedSubnet));
    }

    #[test]
    fn unspecified_bind_is_rejected() {
        assert_eq!(
            validate_bind_address("0.0.0.0"),
            Err(GatewayContractError::UnspecifiedBindAddress)
        );
        assert_eq!(
            validate_bind_address("::"),
            Err(GatewayContractError::UnspecifiedBindAddress)
        );
        assert!(validate_bind_address("127.0.0.1").is_ok());
        assert!(validate_bind_address("192.168.2.1").is_ok());
    }

    #[test]
    fn public_downstream_subnet_is_rejected() {
        let mut s = live_status();
        s.downstream_cidr = "8.8.8.8/24".into();
        s.downstream_subnet = "8.8.8.0/24".into();
        assert_eq!(s.validate(), Err(GatewayContractError::NonPrivateSubnet));
    }

    #[test]
    fn path_like_interface_is_rejected() {
        let mut s = live_status();
        s.downstream_interface = "../etc/passwd".into();
        assert_eq!(
            s.validate(),
            Err(GatewayContractError::InvalidInterfaceName)
        );

        let mut empty = live_status();
        empty.upstream_interface = String::new();
        assert_eq!(
            empty.validate(),
            Err(GatewayContractError::InvalidInterfaceName)
        );
    }

    #[test]
    fn same_interface_needs_preflight_approval() {
        let mut s = live_status();
        s.downstream_interface = "wlp2s0".into();
        assert_eq!(
            s.validate(),
            Err(GatewayContractError::SameInterfaceWithoutApproval)
        );
        assert!(s.validate_with_concurrent_approval(true).is_ok());
    }

    #[test]
    fn loopback_downstream_address_is_rejected() {
        let mut s = live_status();
        s.downstream_cidr = "127.0.0.1/24".into();
        s.downstream_subnet = "127.0.0.0/24".into();
        assert_eq!(
            s.validate(),
            Err(GatewayContractError::LoopbackOrUnspecifiedAddress)
        );
    }

    #[test]
    fn status_round_trips_through_json() {
        let original = live_status();
        let json = serde_json::to_string(&original).unwrap();
        let back: GatewayStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(original, back);
        assert!(json.contains("\"ethernet\""));
        assert!(json.contains("\"ready\""));
        // Kontrakt nie przenosi zadnych sekretow.
        assert!(!json.contains("psk") && !json.contains("password"));
    }
}
