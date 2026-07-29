#![forbid(unsafe_code)]

use std::net::Ipv4Addr;
use std::process::ExitCode;
use std::time::Duration;

use clap::{Parser, Subcommand, error::ErrorKind};
use l2linkscope_core::{
    DiscoveryError, DiscoveryErrorCode, Interface, JSON_SCHEMA_VERSION, ObservationKind,
    OperationalState,
};
use l2linkscope_linux::{DhcpV4ProbeOptions, LinuxError};
use serde::Serialize;

#[derive(Debug, Parser)]
#[command(
    name = "l2linkscope",
    version,
    about = "Observe and probe the local network without configuring it"
)]
struct Cli {
    /// Emit versioned machine-readable JSON.
    #[arg(long, global = true)]
    json: bool,

    /// Write acquisition progress diagnostics to standard error.
    #[arg(long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// List normalized Linux network-interface metadata.
    Interfaces,
    /// Run one explicitly requested, bounded active probe.
    Probe {
        #[command(subcommand)]
        protocol: ProbeCommand,
    },
}

#[derive(Debug, Subcommand)]
enum ProbeCommand {
    /// Send one DHCPv4 Discover and collect matching Offers without accepting a lease.
    Dhcp4 {
        /// Kernel name of the one interface on which to probe.
        interface: String,

        /// Bounded Offer collection window (250ms through 30s).
        #[arg(long, default_value = "5s", value_parser = parse_timeout)]
        timeout: Duration,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
enum ProcessExit {
    Success = 0,
    NoOffers = 1,
    InvalidArguments = 2,
    InterfaceNotFound = 3,
    UnsupportedInterface = 4,
    InsufficientPrivileges = 5,
    TransportFailure = 6,
    MalformedResponses = 7,
    Internal = 70,
}

impl From<ProcessExit> for ExitCode {
    fn from(value: ProcessExit) -> Self {
        Self::from(value as u8)
    }
}

#[derive(Serialize)]
struct InterfacesOutput<'a> {
    schema_version: &'static str,
    interfaces: &'a [Interface],
}

#[derive(Serialize)]
struct ErrorOutput {
    schema_version: &'static str,
    error: DiscoveryError,
}

fn main() -> ExitCode {
    match Cli::try_parse() {
        Ok(cli) => execute(cli).into(),
        Err(error) => {
            let success = matches!(
                error.kind(),
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
            );
            if let Err(print_error) = error.print() {
                eprintln!("error: could not write command-line help: {print_error}");
                return ProcessExit::Internal.into();
            }
            if success {
                ProcessExit::Success.into()
            } else {
                ProcessExit::InvalidArguments.into()
            }
        }
    }
}

fn execute(cli: Cli) -> ProcessExit {
    match cli.command {
        Command::Interfaces => run_interfaces(cli.json),
        Command::Probe {
            protocol: ProbeCommand::Dhcp4 { interface, timeout },
        } => run_dhcp4(&interface, timeout, cli.json, cli.verbose),
    }
}

fn run_interfaces(json: bool) -> ProcessExit {
    match l2linkscope_linux::interfaces() {
        Ok(interfaces) => {
            if json {
                let output = InterfacesOutput {
                    schema_version: JSON_SCHEMA_VERSION,
                    interfaces: &interfaces,
                };
                if let Err(error) = write_json_stdout(&output) {
                    eprintln!("error: {error}");
                    return ProcessExit::Internal;
                }
            } else {
                print_interfaces(&interfaces);
            }
            ProcessExit::Success
        }
        Err(error) => report_error(error, json),
    }
}

fn run_dhcp4(interface: &str, timeout: Duration, json: bool, verbose: bool) -> ProcessExit {
    if verbose {
        eprintln!(
            "sending one DHCPv4 Discover on {interface}; collecting for {} ms",
            timeout.as_millis()
        );
    }
    let options = match DhcpV4ProbeOptions::new(timeout) {
        Ok(options) => options,
        Err(error) => return report_error(error, json),
    };
    match l2linkscope_linux::probe_dhcp_v4(interface, options) {
        Ok(snapshot) => {
            let offer_count = snapshot.observations.len();
            if json {
                if let Err(error) = write_json_stdout(&snapshot) {
                    eprintln!("error: {error}");
                    return ProcessExit::Internal;
                }
            } else {
                print_probe(interface, timeout, &snapshot);
            }
            if offer_count == 0 {
                ProcessExit::NoOffers
            } else {
                ProcessExit::Success
            }
        }
        Err(error) => report_error(error, json),
    }
}

fn write_json_stdout(value: &impl Serialize) -> Result<(), serde_json::Error> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

fn report_error(error: LinuxError, json: bool) -> ProcessExit {
    let exit = exit_for_error(error.code());
    if json {
        let output = ErrorOutput {
            schema_version: JSON_SCHEMA_VERSION,
            error: error.as_discovery_error(),
        };
        match serde_json::to_string_pretty(&output) {
            Ok(value) => eprintln!("{value}"),
            Err(json_error) => {
                eprintln!("error: {error}");
                eprintln!("error: could not serialize structured error: {json_error}");
                return ProcessExit::Internal;
            }
        }
    } else {
        eprintln!("error: {error}");
    }
    exit
}

const fn exit_for_error(code: DiscoveryErrorCode) -> ProcessExit {
    match code {
        DiscoveryErrorCode::InterfaceNotFound => ProcessExit::InterfaceNotFound,
        DiscoveryErrorCode::UnsupportedInterface => ProcessExit::UnsupportedInterface,
        DiscoveryErrorCode::InsufficientPrivileges => ProcessExit::InsufficientPrivileges,
        DiscoveryErrorCode::TransportFailure => ProcessExit::TransportFailure,
        DiscoveryErrorCode::MalformedResponses => ProcessExit::MalformedResponses,
        DiscoveryErrorCode::InvalidInput => ProcessExit::InvalidArguments,
        DiscoveryErrorCode::Internal => ProcessExit::Internal,
    }
}

fn print_interfaces(interfaces: &[Interface]) {
    for (position, interface) in interfaces.iter().enumerate() {
        if position > 0 {
            println!();
        }
        println!("{}", display_untrusted(&interface.name));
        println!("  Index:       {}", interface.id.index);
        println!("  Link:        {}", display_link(interface));
        println!(
            "  MAC:         {}",
            interface
                .id
                .hardware_address
                .map_or_else(|| "unavailable".to_owned(), |value| value.to_string())
        );
        println!("  MTU:         {}", interface.mtu);
        println!(
            "  Addresses:   {}",
            if interface.addresses.is_empty() {
                "none".to_owned()
            } else {
                interface
                    .addresses
                    .iter()
                    .map(|value| format!("{}/{}", value.address, value.prefix_length))
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        );
        let probe = if interface.dhcp_v4_probe.supported {
            "supported".to_owned()
        } else {
            format!(
                "unsupported ({})",
                interface
                    .dhcp_v4_probe
                    .reason
                    .as_deref()
                    .unwrap_or("unspecified reason")
            )
        };
        println!("  DHCP probe:  {probe}");
    }
}

fn display_link(interface: &Interface) -> &'static str {
    match (interface.operational_state, interface.carrier_state) {
        (OperationalState::Up, l2linkscope_core::CarrierState::Present) => "connected",
        (OperationalState::Down | OperationalState::LowerLayerDown, _)
        | (_, l2linkscope_core::CarrierState::Absent) => "disconnected",
        _ => "unknown",
    }
}

fn print_probe(interface: &str, timeout: Duration, snapshot: &l2linkscope_core::DiscoverySnapshot) {
    if snapshot.observations.is_empty() {
        println!(
            "No DHCPv4 Offer was observed on {interface} during the {} probe.",
            display_probe_duration(timeout)
        );
        println!("No network configuration was changed.");
        return;
    }

    println!("DHCPv4 offers observed on {interface}");
    for (index, observation) in snapshot.observations.iter().enumerate() {
        let ObservationKind::DhcpV4Offer(offer) = observation.kind();
        println!();
        println!("Offer {}", index + 1);
        println!(
            "  Server:            {}",
            offer
                .server_identifier
                .map_or_else(|| "not advertised".to_owned(), |value| value.to_string())
        );
        println!("  Proposed address:  {}", offer.offered_address);
        if let Some((network, prefix)) = offer
            .subnet_mask
            .and_then(|mask| derived_network(offer.offered_address, mask))
        {
            println!("  Derived network:   {}/{}", network, prefix);
        }
        println!("  Routers:           {}", display_ipv4_list(&offer.routers));
        println!(
            "  DNS servers:       {}",
            display_ipv4_list(&offer.dns_servers)
        );
        if let Some(domain) = &offer.domain_name {
            println!("  Domain name:       {}", display_untrusted(domain));
        }
        if !offer.domain_search.is_empty() {
            println!(
                "  Domain search:     {}",
                offer
                    .domain_search
                    .iter()
                    .map(|name| display_untrusted(name))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
        if let Some(seconds) = offer.lease_time_seconds {
            println!(
                "  Lease duration:    {}",
                display_duration(u64::from(seconds))
            );
        }
        if let Some(mtu) = offer.interface_mtu {
            println!("  Advertised MTU:    {mtu}");
        }
        println!("  Evidence:          advertised by peer");
    }
    for warning in &snapshot.warnings {
        println!();
        println!("Warning: {}", warning.message);
    }
    println!();
    println!("This configuration was advertised but not accepted or applied.");
}

fn display_ipv4_list(values: &[Ipv4Addr]) -> String {
    if values.is_empty() {
        "not advertised".to_owned()
    } else {
        values
            .iter()
            .map(Ipv4Addr::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn derived_network(address: Ipv4Addr, mask: Ipv4Addr) -> Option<(Ipv4Addr, u32)> {
    let bits = u32::from(mask);
    if bits.leading_ones() + bits.trailing_zeros() != 32 {
        return None;
    }
    Some((Ipv4Addr::from(u32::from(address) & bits), bits.count_ones()))
}

fn display_untrusted(value: &str) -> String {
    value.chars().flat_map(char::escape_default).collect()
}

fn display_probe_duration(duration: Duration) -> String {
    if duration.as_millis() < 1_000 {
        format!("{} milliseconds", duration.as_millis())
    } else {
        display_duration(duration.as_secs())
    }
}

fn display_duration(seconds: u64) -> String {
    if seconds != 0 && seconds % 3_600 == 0 {
        let hours = seconds / 3_600;
        format!("{hours} hour{}", if hours == 1 { "" } else { "s" })
    } else if seconds != 0 && seconds % 60 == 0 {
        let minutes = seconds / 60;
        format!("{minutes} minute{}", if minutes == 1 { "" } else { "s" })
    } else {
        format!("{seconds} second{}", if seconds == 1 { "" } else { "s" })
    }
}

fn parse_timeout(value: &str) -> Result<Duration, String> {
    let duration = if let Some(milliseconds) = value.strip_suffix("ms") {
        milliseconds
            .parse::<u64>()
            .map(Duration::from_millis)
            .map_err(|_| "timeout must be an integer followed by ms or s".to_owned())?
    } else {
        let seconds = value.strip_suffix('s').unwrap_or(value);
        seconds
            .parse::<u64>()
            .map(Duration::from_secs)
            .map_err(|_| "timeout must be an integer followed by ms or s".to_owned())?
    };
    DhcpV4ProbeOptions::new(duration)
        .map(|options| options.timeout)
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;
    use std::time::Duration;

    use clap::{CommandFactory, Parser};
    use l2linkscope_core::DiscoveryErrorCode;

    use super::{
        Cli, Command, ProbeCommand, ProcessExit, derived_network, display_untrusted,
        exit_for_error, parse_timeout,
    };

    #[test]
    fn parses_interfaces_json_after_subcommand() {
        let cli = Cli::try_parse_from(["l2linkscope", "interfaces", "--json"])
            .unwrap_or_else(|error| error.exit());
        assert!(cli.json);
        assert!(matches!(cli.command, Command::Interfaces));
    }

    #[test]
    fn parses_bounded_dhcp_probe() {
        let cli = Cli::try_parse_from([
            "l2linkscope",
            "probe",
            "dhcp4",
            "enp3s0",
            "--timeout",
            "750ms",
        ])
        .unwrap_or_else(|error| error.exit());
        assert!(matches!(
            cli.command,
            Command::Probe {
                protocol: ProbeCommand::Dhcp4 { interface, timeout }
            } if interface == "enp3s0" && timeout == Duration::from_millis(750)
        ));
    }

    #[test]
    fn rejects_unbounded_timeout() {
        assert!(parse_timeout("31s").is_err());
        assert!(parse_timeout("249ms").is_err());
    }

    #[test]
    fn exit_categories_are_stable() {
        assert_eq!(
            exit_for_error(DiscoveryErrorCode::InterfaceNotFound),
            ProcessExit::InterfaceNotFound
        );
        assert_eq!(
            exit_for_error(DiscoveryErrorCode::InsufficientPrivileges),
            ProcessExit::InsufficientPrivileges
        );
        assert_eq!(
            exit_for_error(DiscoveryErrorCode::MalformedResponses),
            ProcessExit::MalformedResponses
        );
    }

    #[test]
    fn clap_definition_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn escapes_untrusted_terminal_control_characters() {
        assert_eq!(display_untrusted("safe\u{1b}[31m\n"), "safe\\u{1b}[31m\\n");
    }

    #[test]
    fn derives_only_from_contiguous_subnet_masks() {
        assert_eq!(
            derived_network(
                "192.0.2.117".parse().unwrap_or(Ipv4Addr::UNSPECIFIED),
                "255.255.255.0".parse().unwrap_or(Ipv4Addr::UNSPECIFIED)
            ),
            Some(("192.0.2.0".parse().unwrap_or(Ipv4Addr::UNSPECIFIED), 24))
        );
        assert!(
            derived_network(Ipv4Addr::new(192, 0, 2, 117), Ipv4Addr::new(255, 0, 255, 0)).is_none()
        );
    }
}
