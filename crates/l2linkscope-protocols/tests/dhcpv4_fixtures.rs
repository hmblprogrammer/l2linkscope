use std::net::Ipv4Addr;

use l2linkscope_protocols::dhcpv4::{
    ClasslessStaticRoute, DhcpDiscover, OfferExpectation, ParseError, ParsedDhcpV4Offer,
    parse_offer,
};

const TRANSACTION_ID: u32 = 0x1234_5678;
const CLIENT_MAC: [u8; 6] = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55];

fn decode_hex_fixture(contents: &str) -> Vec<u8> {
    let compact = contents
        .lines()
        .map(|line| line.split('#').next().unwrap_or_default())
        .flat_map(str::split_whitespace)
        .collect::<String>();
    assert_eq!(
        compact.len() % 2,
        0,
        "fixture must contain pairs of hex digits"
    );
    compact
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let encoded = std::str::from_utf8(pair).unwrap_or_default();
            u8::from_str_radix(encoded, 16).unwrap_or_default()
        })
        .collect()
}

fn offer_from_options(options: &[u8]) -> Vec<u8> {
    let mut packet = vec![0_u8; 240];
    packet[0] = 2;
    packet[1] = 1;
    packet[2] = 6;
    packet[4..8].copy_from_slice(&TRANSACTION_ID.to_be_bytes());
    packet[16..20].copy_from_slice(&[192, 0, 2, 117]);
    packet[28..34].copy_from_slice(&CLIENT_MAC);
    packet[236..240].copy_from_slice(&[99, 130, 83, 99]);
    packet.extend_from_slice(options);
    packet
}

fn fixture(contents: &str) -> Vec<u8> {
    offer_from_options(&decode_hex_fixture(contents))
}

fn expectation() -> OfferExpectation {
    OfferExpectation {
        transaction_id: TRANSACTION_ID,
        client_hardware_address: CLIENT_MAC,
    }
}

fn successful_offer(result: Result<ParsedDhcpV4Offer, ParseError>) -> Vec<ParsedDhcpV4Offer> {
    let offers: Vec<_> = result.into_iter().collect();
    assert_eq!(offers.len(), 1, "expected one successfully parsed Offer");
    offers
}

#[test]
fn parses_valid_minimal_offer_fixture() {
    let packet = fixture(include_str!("fixtures/minimal-offer.options.hex"));
    let offers = successful_offer(parse_offer(&packet, expectation()));
    let offer = &offers[0];

    assert_eq!(offer.transaction_id, TRANSACTION_ID);
    assert_eq!(offer.client_hardware_address, CLIENT_MAC);
    assert_eq!(offer.offered_address, Ipv4Addr::new(192, 0, 2, 117));
    assert_eq!(offer.server_identifier, None);
    assert!(offer.routers.is_empty());
}

#[test]
fn parses_offer_with_common_network_options_fixture() {
    let packet = fixture(include_str!("fixtures/common-offer.options.hex"));
    let offers = successful_offer(parse_offer(&packet, expectation()));
    let offer = &offers[0];

    assert_eq!(offer.server_identifier, Some(Ipv4Addr::new(192, 0, 2, 1)));
    assert_eq!(offer.subnet_mask, Some(Ipv4Addr::new(255, 255, 255, 0)));
    assert_eq!(
        offer.routers,
        [Ipv4Addr::new(192, 0, 2, 1), Ipv4Addr::new(192, 0, 2, 254)]
    );
    assert_eq!(
        offer.dns_servers,
        [Ipv4Addr::new(192, 0, 2, 10), Ipv4Addr::new(192, 0, 2, 11)]
    );
    assert_eq!(offer.domain_name.as_deref(), Some("example.test"));
    assert_eq!(offer.domain_search, ["corp.example", "test.example"]);
    assert_eq!(offer.lease_time_seconds, Some(7_200));
    assert_eq!(offer.renewal_time_seconds, Some(3_600));
    assert_eq!(offer.rebinding_time_seconds, Some(6_300));
    assert_eq!(offer.interface_mtu, Some(1_500));
    assert_eq!(
        offer.classless_static_routes,
        [
            ClasslessStaticRoute {
                destination: Ipv4Addr::UNSPECIFIED,
                prefix_length: 0,
                router: Ipv4Addr::new(192, 0, 2, 1),
            },
            ClasslessStaticRoute {
                destination: Ipv4Addr::new(192, 0, 2, 0),
                prefix_length: 24,
                router: Ipv4Addr::new(192, 0, 2, 254),
            },
        ]
    );
}

#[test]
fn tolerates_unknown_and_padded_options() {
    let options = decode_hex_fixture("c8 03 aa bb cc 00 35 01 02 00 ff");
    let offer = parse_offer(&offer_from_options(&options), expectation());
    assert!(offer.is_ok());
}

#[test]
fn combines_duplicate_list_options() {
    let options = decode_hex_fixture(
        "35 01 02 03 04 c0 00 02 01 03 04 c0 00 02 fe \
         06 04 c0 00 02 0a 06 04 c0 00 02 0b ff",
    );
    let packet = offer_from_options(&options);
    let offers = successful_offer(parse_offer(&packet, expectation()));
    let offer = &offers[0];
    assert_eq!(offer.routers.len(), 2);
    assert_eq!(offer.dns_servers.len(), 2);
}

#[test]
fn permits_identical_duplicate_scalar_option() {
    let options = decode_hex_fixture("35 01 02 36 04 c0 00 02 01 36 04 c0 00 02 01 ff");
    let packet = offer_from_options(&options);
    let offers = successful_offer(parse_offer(&packet, expectation()));
    let offer = &offers[0];
    assert_eq!(offer.server_identifier, Some(Ipv4Addr::new(192, 0, 2, 1)));
}

#[test]
fn rejects_conflicting_duplicate_scalar_option() {
    let options = decode_hex_fixture("35 01 02 36 04 c0 00 02 01 36 04 c0 00 02 02 ff");
    let result = parse_offer(&offer_from_options(&options), expectation());
    assert_eq!(
        result.err(),
        Some(ParseError::ConflictingOption { code: 54 })
    );
}

#[test]
fn rejects_truncated_packet() {
    let error = parse_offer(&[0; 239], expectation()).err();
    assert_eq!(
        error,
        Some(ParseError::Truncated {
            actual: 239,
            minimum: 240
        })
    );
}

#[test]
fn rejects_invalid_option_length() {
    let options = decode_hex_fixture("35 01 02 01 03 ff ff 00 ff");
    let error = parse_offer(&offer_from_options(&options), expectation()).err();
    assert_eq!(
        error,
        Some(ParseError::InvalidOptionLength {
            code: 1,
            actual: 3,
            expected: "4 bytes",
        })
    );
}

#[test]
fn rejects_option_value_truncated_by_packet_end() {
    let options = decode_hex_fixture("35 01 02 36 04 c0 00");
    let error = parse_offer(&offer_from_options(&options), expectation()).err();
    assert!(matches!(
        error,
        Some(ParseError::TruncatedOption { code: Some(54), .. })
    ));
}

#[test]
fn rejects_missing_magic_cookie() {
    let mut packet = fixture(include_str!("fixtures/minimal-offer.options.hex"));
    packet[236..240].fill(0);
    assert_eq!(
        parse_offer(&packet, expectation()),
        Err(ParseError::MissingMagicCookie)
    );
}

#[test]
fn rejects_wrong_transaction_id() {
    let packet = fixture(include_str!("fixtures/minimal-offer.options.hex"));
    let expected = OfferExpectation {
        transaction_id: TRANSACTION_ID + 1,
        ..expectation()
    };
    assert!(matches!(
        parse_offer(&packet, expected),
        Err(ParseError::WrongTransactionId { .. })
    ));
}

#[test]
fn rejects_wrong_client_hardware_address() {
    let packet = fixture(include_str!("fixtures/minimal-offer.options.hex"));
    let expected = OfferExpectation {
        client_hardware_address: [0xff; 6],
        ..expectation()
    };
    assert!(matches!(
        parse_offer(&packet, expected),
        Err(ParseError::WrongClientHardwareAddress { .. })
    ));
}

#[test]
fn rejects_ack_presented_as_offer() {
    let options = decode_hex_fixture("35 01 05 ff");
    assert_eq!(
        parse_offer(&offer_from_options(&options), expectation()),
        Err(ParseError::NotOffer { actual: 5 })
    );
}

#[test]
fn rejects_malformed_classless_route_data() {
    let options = decode_hex_fixture("35 01 02 79 04 18 c0 00 02 ff");
    let error = parse_offer(&offer_from_options(&options), expectation()).err();
    assert!(matches!(
        error,
        Some(ParseError::InvalidClasslessRoute { .. })
    ));
}

#[test]
fn normalizes_classless_route_host_bits() {
    let options = decode_hex_fixture("35 01 02 79 09 19 c0 00 02 ff c0 00 02 01 ff");
    let packet = offer_from_options(&options);
    let offers = successful_offer(parse_offer(&packet, expectation()));
    let offer = &offers[0];
    assert_eq!(
        offer.classless_static_routes[0].destination,
        Ipv4Addr::new(192, 0, 2, 128)
    );
}

#[test]
fn rejects_domain_search_compression_loop() {
    let options = decode_hex_fixture("35 01 02 77 02 c0 00 ff");
    let error = parse_offer(&offer_from_options(&options), expectation()).err();
    assert!(matches!(
        error,
        Some(ParseError::InvalidDomainSearch { .. })
    ));
}

#[test]
fn discover_requests_expected_options_and_omits_hostname() {
    let packet = DhcpDiscover::new(TRANSACTION_ID, CLIENT_MAC).encode();
    let options = &packet[240..];
    assert!(options.windows(3).any(|window| window == [53, 1, 1]));
    assert!(
        options
            .windows(2)
            .any(|window| window[0] == 55 && window[1] == 11)
    );
    assert!(!options.windows(2).any(|window| window[0] == 12));
}
