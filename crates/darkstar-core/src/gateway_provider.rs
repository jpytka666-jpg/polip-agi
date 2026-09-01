// darkstar-header-v1
// po co: gateway_provider.rs
// nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie
// autor: Marcin
// powstal: 2026-09-01
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: Claude Opus 5
//! TIMESTAMP: 2026-09-01 21:35:00
//! REASON FOR CREATION: Waski, tylko-do-odczytu dostawca stanu bramy Darkstar na Ubuntu (Task 8, Step 8.2 i 8.4).
//! MECHANICS: `CommandRunner` udostepnia dokladnie cztery odczyty (nmcli active, ip addr, nmcli
//! ipv4, ip neigh) i nic wiecej - nie ma w nim miejsca na apply, stop ani restart. Rozbior
//! wyjscia jest czysta funkcja, wiec testuje sie go na utrwalonym wyjsciu z hosta CBMS bez
//! dotykania sieci. Wynik przechodzi przez walidacje z gateway_status, zanim wroci do wolajacego.
//! SYSTEM PART: Darkstar Core / Network Boundary - dostawca odczytu bramy.
//! ARCHITECTURE FUNCTION: Granica miedzy systemem operacyjnym a reszta Darkstara. Zwezenie
//! interfejsu jest zabezpieczeniem: dostawca nie moze zmienic sieci, nawet gdyby ktos go o to
//! poprosil - warstwa mutujaca powstanie osobno i z osobna autoryzacja.
//! DEPENDENCIES/LINKS: gateway_status (GatewayStatus, walidacja); nmcli i ip na hoscie Ubuntu.
//! TECH STACK: Rust 2024 + serde, bez nowych zaleznosci. Rozbior tekstu na strukturach std;
//! wykonanie polecen jest wstrzykiwane przez trait, wiec testy nie potrzebuja hosta.
//! LOCAL WORKSPACE: D:\codex-fresh-2026-08-28\worktrees\polip-agi-darkstar-plan
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
//! ==========================================

use crate::gateway_status::{GatewayContractError, GatewayHealth, GatewayMode, GatewayStatus};
use std::net::Ipv4Addr;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GatewayProviderError {
    #[error("command failed: {0}")]
    CommandFailed(String),
    #[error("could not parse host output: {0}")]
    Unparseable(String),
    #[error(transparent)]
    Contract(#[from] GatewayContractError),
}

/// Waskie okno na system operacyjny. Cztery odczyty, zero mutacji -
/// nie ma tu metody, ktora zmienialaby stan sieci, i nie wolno takiej dodac.
/// Warstwa mutujaca powstanie osobno, z wlasna autoryzacja (Task 8, Step 8.5).
pub trait CommandRunner {
    /// `nmcli -t -f NAME,DEVICE,STATE connection show --active`
    fn active_connections(&self) -> Result<String, GatewayProviderError>;
    /// `ip -4 -o addr show <iface>`
    fn interface_addresses(&self, iface: &str) -> Result<String, GatewayProviderError>;
    /// `nmcli -t -g ipv4.method,ipv4.addresses connection show <profile>`
    fn profile_ipv4(&self, profile: &str) -> Result<String, GatewayProviderError>;
    /// `ip -o neigh show dev <iface>`
    fn neighbours(&self, iface: &str) -> Result<String, GatewayProviderError>;
}

/// Wylicza adres sieci z adresu i dlugosci prefiksu. Czysta funkcja.
pub fn subnet_of(cidr: &str) -> Option<String> {
    let (addr_part, prefix_part) = cidr.split_once('/')?;
    let addr: Ipv4Addr = addr_part.parse().ok()?;
    let prefix: u32 = prefix_part.parse().ok()?;
    if prefix > 32 {
        return None;
    }
    let mask = if prefix == 0 {
        0u32
    } else {
        u32::MAX << (32 - prefix)
    };
    let network = Ipv4Addr::from(u32::from(addr) & mask);
    Some(format!("{network}/{prefix}"))
}

/// Liczy sasiadow nalezacych do prywatnej podsieci bramy. Wpisy spoza niej
/// (hub w innej adresacji, adresy operatorskie) nie sa klientami downstreamu.
pub fn count_downstream_clients(neigh_output: &str, subnet: &str) -> u32 {
    let Some((net_part, prefix_part)) = subnet.split_once('/') else {
        return 0;
    };
    let (Ok(net), Ok(prefix)) = (net_part.parse::<Ipv4Addr>(), prefix_part.parse::<u32>()) else {
        return 0;
    };
    let mask = if prefix == 0 {
        0u32
    } else {
        u32::MAX << (32 - prefix)
    };
    let net_bits = u32::from(net) & mask;

    neigh_output
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .filter_map(|token| token.parse::<Ipv4Addr>().ok())
        .filter(|addr| u32::from(*addr) & mask == net_bits)
        .count() as u32
}

fn first_address_in(ip_addr_output: &str) -> Option<String> {
    ip_addr_output
        .split_whitespace()
        .skip_while(|token| *token != "inet")
        .nth(1)
        .map(|s| s.to_string())
}

pub struct GatewayProvider<R: CommandRunner> {
    runner: R,
    upstream_interface: String,
    downstream_interface: String,
    connection_profile: String,
}

impl<R: CommandRunner> GatewayProvider<R> {
    pub fn new(runner: R, upstream: &str, downstream: &str, profile: &str) -> Self {
        Self {
            runner,
            upstream_interface: upstream.to_string(),
            downstream_interface: downstream.to_string(),
            connection_profile: profile.to_string(),
        }
    }

    /// Wylacznie odczyt. Zwraca zwalidowany stan albo blad - nigdy stanu polowicznego.
    pub fn read_status(&self, now_unix_ms: u64) -> Result<GatewayStatus, GatewayProviderError> {
        // nmcli bywa niedostepny tam, gdzie `ip` dziala: w kontenerze na CBMS AppArmor
        // blokuje magistrale DBus. Brak nmcli NIE moze oznaczac braku bramy - stan
        // wyliczamy wtedy z samego `ip`, a niepotwierdzony profil jest oznaczony wprost.
        let nmcli_active = self.runner.active_connections().ok();
        let nmcli_profile = self.runner.profile_ipv4(&self.connection_profile).ok();
        let nmcli_available = nmcli_active.is_some() && nmcli_profile.is_some();

        let profile_active = nmcli_active
            .as_deref()
            .map(|active| {
                active.lines().any(|line| {
                    let mut parts = line.split(':');
                    parts.next() == Some(self.connection_profile.as_str())
                        && parts.next() == Some(self.downstream_interface.as_str())
                        && parts.next() == Some("activated")
                })
            })
            .unwrap_or(false);

        let addr_output = self
            .runner
            .interface_addresses(&self.downstream_interface)?;
        let live_cidr = first_address_in(&addr_output);

        let mut profile_lines = nmcli_profile.as_deref().unwrap_or_default().lines();
        let method = profile_lines.next().unwrap_or_default().trim().to_string();
        let configured_cidr = profile_lines.next().unwrap_or_default().trim().to_string();

        // Adres zywy jest zrodlem prawdy; konfiguracja profilu sluzy jako zapasowa.
        let cidr = live_cidr.clone().unwrap_or(configured_cidr);
        let subnet = subnet_of(&cidr)
            .ok_or_else(|| GatewayProviderError::Unparseable(format!("cidr: {cidr}")))?;

        let health = if nmcli_available {
            if !profile_active {
                GatewayHealth::Offline
            } else if live_cidr.is_none() || method != "shared" {
                GatewayHealth::Degraded
            } else {
                GatewayHealth::Ready
            }
        } else if live_cidr.is_some() {
            // Interfejs downstream ma swoj adres - brama stoi. Profilu nie potwierdzamy.
            GatewayHealth::Ready
        } else {
            GatewayHealth::Offline
        };

        let connection_profile = if nmcli_available {
            self.connection_profile.clone()
        } else {
            "unconfirmed:nmcli-unavailable".to_string()
        };

        let connected_clients = if profile_active || !nmcli_available {
            let neigh = self.runner.neighbours(&self.downstream_interface)?;
            count_downstream_clients(&neigh, &subnet)
        } else {
            0
        };

        let status = GatewayStatus {
            // Tryb hotspot nie jest obslugiwany na tym hoscie - karta nie ma
            // zatwierdzonej pracy jednoczesnej, wiec odczyt zawsze mowi Ethernet.
            mode: GatewayMode::Ethernet,
            health,
            upstream_interface: self.upstream_interface.clone(),
            downstream_interface: self.downstream_interface.clone(),
            downstream_cidr: cidr,
            downstream_subnet: subnet,
            connection_profile,
            connected_clients,
            last_verified_unix_ms: now_unix_ms,
        };
        status.validate()?;
        Ok(status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Utrwalone wyjscie z hosta CBMS, odczytane 2026-09-01. Zadnej sieci w tescie.
    const NMCLI_ACTIVE: &str = "DARKSTAR-WiFi:enp1s0:activated\n\
Vodafone480B38 1:wlp2s0:activated\n\
tailscale0:tailscale0:activated\n\
br-e0a0946cdec3:br-e0a0946cdec3:activated\n";

    const IP_ADDR: &str = "2: enp1s0    inet 192.168.2.1/24 brd 192.168.2.255 scope global noprefixroute enp1s0\\       valid_lft forever preferred_lft forever\n";

    const NMCLI_IPV4: &str = "shared\n192.168.2.1/24\n";

    const IP_NEIGH: &str = "192.168.2.2 lladdr a0:2d:db:d7:a9:20 REACHABLE\n\
192.168.1.254 lladdr a0:2d:db:d7:a9:20 STALE\n\
100.64.0.1 lladdr a0:2d:db:d7:a9:20 STALE\n";

    struct FixtureRunner;

    impl CommandRunner for FixtureRunner {
        fn active_connections(&self) -> Result<String, GatewayProviderError> {
            Ok(NMCLI_ACTIVE.into())
        }
        fn interface_addresses(&self, _iface: &str) -> Result<String, GatewayProviderError> {
            Ok(IP_ADDR.into())
        }
        fn profile_ipv4(&self, _profile: &str) -> Result<String, GatewayProviderError> {
            Ok(NMCLI_IPV4.into())
        }
        fn neighbours(&self, _iface: &str) -> Result<String, GatewayProviderError> {
            Ok(IP_NEIGH.into())
        }
    }

    #[test]
    fn reads_live_gateway_shape_from_host_output() {
        let provider = GatewayProvider::new(FixtureRunner, "wlp2s0", "enp1s0", "DARKSTAR-WiFi");
        let status = provider.read_status(1_788_300_000_000).unwrap();

        assert_eq!(status.upstream_interface, "wlp2s0");
        assert_eq!(status.downstream_interface, "enp1s0");
        assert_eq!(status.downstream_cidr, "192.168.2.1/24");
        assert_eq!(status.downstream_subnet, "192.168.2.0/24");
        assert_eq!(status.connection_profile, "DARKSTAR-WiFi");
        assert_eq!(status.mode, GatewayMode::Ethernet);
        assert_eq!(status.health, GatewayHealth::Ready);
    }

    #[test]
    fn counts_only_downstream_neighbours() {
        // Z trzech sasiadow tylko 192.168.2.2 nalezy do prywatnej podsieci bramy.
        assert_eq!(count_downstream_clients(IP_NEIGH, "192.168.2.0/24"), 1);
    }

    #[test]
    fn profile_that_is_not_active_reports_offline() {
        struct Inactive;
        impl CommandRunner for Inactive {
            fn active_connections(&self) -> Result<String, GatewayProviderError> {
                Ok("Vodafone480B38 1:wlp2s0:activated\n".into())
            }
            fn interface_addresses(&self, _i: &str) -> Result<String, GatewayProviderError> {
                Ok(IP_ADDR.into())
            }
            fn profile_ipv4(&self, _p: &str) -> Result<String, GatewayProviderError> {
                Ok(NMCLI_IPV4.into())
            }
            fn neighbours(&self, _i: &str) -> Result<String, GatewayProviderError> {
                Ok(String::new())
            }
        }
        let provider = GatewayProvider::new(Inactive, "wlp2s0", "enp1s0", "DARKSTAR-WiFi");
        let status = provider.read_status(1).unwrap();
        assert_eq!(status.health, GatewayHealth::Offline);
        assert_eq!(status.connected_clients, 0);
    }

    #[test]
    fn shared_method_without_address_is_degraded() {
        struct NoAddress;
        impl CommandRunner for NoAddress {
            fn active_connections(&self) -> Result<String, GatewayProviderError> {
                Ok(NMCLI_ACTIVE.into())
            }
            fn interface_addresses(&self, _i: &str) -> Result<String, GatewayProviderError> {
                Ok(String::new())
            }
            fn profile_ipv4(&self, _p: &str) -> Result<String, GatewayProviderError> {
                Ok(NMCLI_IPV4.into())
            }
            fn neighbours(&self, _i: &str) -> Result<String, GatewayProviderError> {
                Ok(String::new())
            }
        }
        let provider = GatewayProvider::new(NoAddress, "wlp2s0", "enp1s0", "DARKSTAR-WiFi");
        assert_eq!(
            provider.read_status(1).unwrap().health,
            GatewayHealth::Degraded
        );
    }

    #[test]
    fn abandoned_subnet_from_host_is_rejected_by_validation() {
        struct Abandoned;
        impl CommandRunner for Abandoned {
            fn active_connections(&self) -> Result<String, GatewayProviderError> {
                Ok(NMCLI_ACTIVE.into())
            }
            fn interface_addresses(&self, _i: &str) -> Result<String, GatewayProviderError> {
                Ok("2: enp1s0    inet 10.44.0.1/24 brd 10.44.0.255 scope global enp1s0\n".into())
            }
            fn profile_ipv4(&self, _p: &str) -> Result<String, GatewayProviderError> {
                Ok("shared\n10.44.0.1/24\n".into())
            }
            fn neighbours(&self, _i: &str) -> Result<String, GatewayProviderError> {
                Ok(String::new())
            }
        }
        let provider = GatewayProvider::new(Abandoned, "wlp2s0", "enp1s0", "DARKSTAR-WiFi");
        assert!(matches!(
            provider.read_status(1),
            Err(GatewayProviderError::Contract(
                GatewayContractError::AbandonedSubnet
            ))
        ));
    }

    #[test]
    fn nmcli_blocked_still_yields_status_from_ip() {
        // Na CBMS AppArmor blokuje kontenerowi magistrale DBus, wiec nmcli nie odpowiada,
        // a `ip` dziala normalnie. Brak nmcli nie moze oznaczac braku bramy.
        struct NoNmcli;
        impl CommandRunner for NoNmcli {
            fn active_connections(&self) -> Result<String, GatewayProviderError> {
                Err(GatewayProviderError::CommandFailed("nmcli blocked".into()))
            }
            fn interface_addresses(&self, _i: &str) -> Result<String, GatewayProviderError> {
                Ok(IP_ADDR.into())
            }
            fn profile_ipv4(&self, _p: &str) -> Result<String, GatewayProviderError> {
                Err(GatewayProviderError::CommandFailed("nmcli blocked".into()))
            }
            fn neighbours(&self, _i: &str) -> Result<String, GatewayProviderError> {
                Ok(IP_NEIGH.into())
            }
        }

        let provider = GatewayProvider::new(NoNmcli, "wlp2s0", "enp1s0", "DARKSTAR-WiFi");
        let status = provider.read_status(1).unwrap();

        assert_eq!(status.downstream_cidr, "192.168.2.1/24");
        assert_eq!(status.downstream_subnet, "192.168.2.0/24");
        assert_eq!(status.connected_clients, 1);
        // Adres jest, wiec brama stoi - ale profil niepotwierdzony i to ma byc widoczne.
        assert_eq!(status.health, GatewayHealth::Ready);
        assert_eq!(status.connection_profile, "unconfirmed:nmcli-unavailable");
    }

    #[test]
    fn without_nmcli_and_without_address_it_is_offline() {
        struct Nothing;
        impl CommandRunner for Nothing {
            fn active_connections(&self) -> Result<String, GatewayProviderError> {
                Err(GatewayProviderError::CommandFailed("nmcli blocked".into()))
            }
            fn interface_addresses(&self, _i: &str) -> Result<String, GatewayProviderError> {
                Ok(String::new())
            }
            fn profile_ipv4(&self, _p: &str) -> Result<String, GatewayProviderError> {
                Err(GatewayProviderError::CommandFailed("nmcli blocked".into()))
            }
            fn neighbours(&self, _i: &str) -> Result<String, GatewayProviderError> {
                Ok(String::new())
            }
        }
        let provider = GatewayProvider::new(Nothing, "wlp2s0", "enp1s0", "DARKSTAR-WiFi");
        assert!(matches!(
            provider.read_status(1),
            Err(GatewayProviderError::Unparseable(_))
        ));
    }

    #[test]
    fn subnet_is_derived_from_address_and_prefix() {
        assert_eq!(subnet_of("192.168.2.1/24").unwrap(), "192.168.2.0/24");
        assert_eq!(subnet_of("172.20.5.9/16").unwrap(), "172.20.0.0/16");
        assert!(subnet_of("nonsense").is_none());
    }
}
