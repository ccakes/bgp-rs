use bgp_rs::*;
use std::net::IpAddr;

fn encode_as_message(message: Message) -> Vec<u8> {
    let mut data: Vec<u8> = vec![];
    message.encode(&mut data).expect("Encoding message");
    data
}

#[test]
fn test_message_too_large() {
    let mut routes = vec![];
    for subnet in 0..20 {
        for host in 0..20 {
            let addr: IpAddr = format!("2001:{}::{}", subnet, host).parse().unwrap();
            routes.push(NLRIEncoding::IP((addr, 128).into()));
        }
    }
    let message = Message::Update(Update {
        withdrawn_routes: vec![],
        attributes: vec![
            PathAttribute::ORIGIN(Origin::IGP),
            PathAttribute::AS_PATH(ASPath {
                segments: vec![Segment::AS_SEQUENCE(vec![64511])],
            }),
            PathAttribute::NEXT_HOP("10.0.14.1".parse().unwrap()),
            PathAttribute::MULTI_EXIT_DISC(0),
            PathAttribute::LOCAL_PREF(100),
            PathAttribute::CLUSTER_LIST(vec![167780868]),
            PathAttribute::ORIGINATOR_ID(167776001),
        ],
        announced_routes: routes,
    });
    let mut buf = vec![];
    let res = message.encode(&mut buf);
    assert!(res.is_err());
}

#[test]
fn test_encode_open() {
    let capabilities: Vec<OpenCapability> = vec![
        OpenCapability::MultiProtocol((AFI::IPV6, SAFI::Unicast)),
        OpenCapability::FourByteASN(65000),
        OpenCapability::RouteRefresh,
    ];
    let open = Open {
        version: 4,
        peer_asn: 65000,
        hold_timer: 60,
        identifier: 16843009, // 1.1.1.1
        parameters: vec![OpenParameter::Capabilities(capabilities)],
    };
    let mut data: Vec<u8> = vec![];
    open.encode(&mut data).expect("Encoding OPEN");
    #[rustfmt::skip]
    assert_eq!(
        data,
        vec![
            0x4, // Version
            0xfd, 0xe8, // ASN
            0, 0x3c, // Hold Timer
            0x01, 0x01, 0x01, 0x01, // Identifier
            16, // Length of Parameters
            0x02, 14, // Parameter type (Capability) and length
            0x01, 0x04, 0x00, 0x02, 0x00, 0x01, // IPv6 - Unicast
            0x41, 0x04, 0x00, 0x00, 0xfd, 0xe8, // 4-byte ASN
            0x02, 0x00 // Route Refresh
        ]
    );

    let message_data = encode_as_message(Message::Open(open));
    #[rustfmt::skip]
    assert_eq!(
        message_data[16..19],
        [0, 45, 1][..],
    );
}

#[test]
fn test_encode_open_too_large() {
    let capabilities: Vec<_> = (10..100).map(OpenCapability::FourByteASN).collect();
    let open = Open {
        version: 4,
        peer_asn: 65000,
        hold_timer: 60,
        identifier: 16843009, // 1.1.1.1
        parameters: vec![OpenParameter::Capabilities(capabilities)],
    };
    let mut data: Vec<u8> = vec![];
    let res = open.encode(&mut data);
    assert!(res.is_err());
}

#[cfg(feature = "flowspec")]
#[test]
fn test_encode_open_flowspec() {
    let capabilities: Vec<OpenCapability> = vec![
        OpenCapability::MultiProtocol((AFI::IPV6, SAFI::Unicast)),
        OpenCapability::MultiProtocol((AFI::IPV4, SAFI::Flowspec)),
        OpenCapability::FourByteASN(65000),
        OpenCapability::RouteRefresh,
    ];
    let open = Open {
        version: 4,
        peer_asn: 65000,
        hold_timer: 60,
        identifier: 16843009, // 1.1.1.1
        parameters: vec![OpenParameter::Capabilities(capabilities)],
    };
    let mut data: Vec<u8> = vec![];
    open.encode(&mut data).expect("Encoding OPEN");
    #[rustfmt::skip]
    assert_eq!(
        data,
        vec![
            0x4, // Version
            0xfd, 0xe8, // ASN
            0, 0x3c, // Hold Timer
            0x01, 0x01, 0x01, 0x01, // Identifier
            28,   // Parameter Length
            0x02, 0x06, 0x01, 0x04, 0x00, 0x02, 0x00, 0x01, // IPv6 - Unicast
            0x02, 0x06, 0x01, 0x04, 0x00, 0x01, 0x00, 0x85, // IPv4 - FlowSpec
            0x02, 0x06, 0x41, 0x04, 0x00, 0x00, 0xfd, 0xe8, // 4-byte ASN
            0x02, 0x02, 0x02, 0x00 // Route Refresh
        ]
    );

    let message_data = encode_as_message(Message::Open(open));
    #[rustfmt::skip]
    assert_eq!(
        message_data[16..19],
        [0, 57, 1][..],
    );
}

#[test]
fn test_encode_nlri() {
    let nlri = NLRIEncoding::IP(Prefix {
        protocol: AFI::IPV6,
        length: 17,
        prefix: vec![0x0a, 0x0a, 0x80, 0x00],
    });
    let mut data: Vec<u8> = vec![];
    nlri.encode(&mut data).expect("Encoding NLRI");
    assert_eq!(data, vec![17, 10, 10, 128]);

    let nlri = NLRIEncoding::IP(Prefix {
        protocol: AFI::IPV6,
        length: 64,
        prefix: vec![
            0x20, 0x01, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ],
    });
    let mut data: Vec<u8> = vec![];
    nlri.encode(&mut data).expect("Encoding NLRI");
    assert_eq!(data, vec![64, 32, 1, 0, 16, 0, 0, 0, 0]);
}

#[test]
fn test_encode_route_refresh() {
    let refresh = RouteRefresh {
        afi: AFI::IPV4,
        safi: SAFI::Unicast,
        subtype: 1u8,
    };
    let mut data: Vec<u8> = vec![];
    refresh.encode(&mut data).expect("Encoding Route Refresh");
    assert_eq!(data, vec![0, 1, 1, 1]);

    let message_data = encode_as_message(Message::RouteRefresh(refresh));
    #[rustfmt::skip]
    assert_eq!(
        message_data[16..19],
        [0, 23, 5][..],
    );
}

#[test]
fn test_encode_update_add_path() {
    let update = Update {
        withdrawn_routes: vec![],
        attributes: vec![
            PathAttribute::ORIGIN(Origin::IGP),
            PathAttribute::AS_PATH(ASPath {
                segments: vec![Segment::AS_SEQUENCE(vec![64511])],
            }),
            PathAttribute::NEXT_HOP("10.0.14.1".parse().unwrap()),
            PathAttribute::MULTI_EXIT_DISC(0),
            PathAttribute::LOCAL_PREF(100),
            PathAttribute::CLUSTER_LIST(vec![167780868]),
            PathAttribute::ORIGINATOR_ID(167776001),
        ],
        announced_routes: vec![
            NLRIEncoding::IP_WITH_PATH_ID((("5.5.5.5".parse().unwrap(), 32).into(), 1)),
            NLRIEncoding::IP_WITH_PATH_ID((("192.168.1.5".parse().unwrap(), 32).into(), 1)),
        ],
    };

    let mut data: Vec<u8> = vec![];
    update.encode(&mut data).expect("Encoding Update");
    #[rustfmt::skip]
    assert_eq!(
        data,
        vec![
            0, 0, // Withdrawn Routes Length
            0, 46, // Path Attribute Length
            64, 1, 1, 0, // ORIGIN
            64, 2, 4, 2, 1, 251, 255, // AS_PATH
            64, 3, 4, 10, 0, 14, 1,  // NEXT_HOP
            128, 4, 4, 0, 0, 0, 0, // MED
            64, 5, 4, 0, 0, 0, 100, // LOCAL_PREF
            128, 10, 4, 10, 0, 34, 4, // CLUSTER LIST
            128, 9, 4, 10, 0, 15, 1, // ORIGINATOR_ID
            // NLRI
            0, 0, 0, 1, 32, 5, 5, 5, 5, // 5.5.5.5/32 w/ Path ID 1
            0, 0, 0, 1, 32, 192, 168, 1, 5   // 192.168.1.5/32 w/ Path ID 1
        ]
    );

    let message_data = encode_as_message(Message::Update(update));
    #[rustfmt::skip]
    assert_eq!(
        message_data[16..19],
        [0, 87, 2][..],
    );
}

#[test]
fn test_encode_update_withdraw() {
    let update = Update {
        withdrawn_routes: vec![
            NLRIEncoding::IP(("5.5.5.5".parse().unwrap(), 32).into()),
            NLRIEncoding::IP(("192.168.1.5".parse().unwrap(), 32).into()),
        ],
        attributes: vec![
            PathAttribute::ORIGIN(Origin::IGP),
            PathAttribute::AS_PATH(ASPath {
                segments: vec![Segment::AS_SEQUENCE(vec![64511])],
            }),
            PathAttribute::MULTI_EXIT_DISC(0),
            PathAttribute::LOCAL_PREF(100),
            // IPv6 withdraw
            PathAttribute::MP_UNREACH_NLRI(MPUnreachNLRI {
                afi: AFI::IPV6,
                safi: SAFI::Unicast,
                withdrawn_routes: vec![
                    NLRIEncoding::IP(("3001:10:10::".parse().unwrap(), 56).into()),
                    NLRIEncoding::IP(("2620:20:20::".parse().unwrap(), 48).into()),
                ],
            }),
        ],
        announced_routes: vec![],
    };

    let mut data: Vec<u8> = vec![];
    update.encode(&mut data).expect("Encoding Update");
    #[rustfmt::skip]
    assert_eq!(
        data,
        vec![
            0, 10, // Withdrawn Routes Length
            32, 5, 5, 5, 5, 32, 192, 168, 1, 5, // Withdrawn prefixes
            0, 46, // Path Attribute Length
            64, 1, 1, 0, // ORIGIN
            64, 2, 4, 2, 1, 251, 255, // AS_PATH
            128, 4, 4, 0, 0, 0, 0, // MED
            64, 5, 4, 0, 0, 0, 100, // LOCAL_PREF
            // MPUnreachNlri
            128, 15, 18, 0, 2, 1,
            56, 48, 1, 0, 16, 0, 16, 0,
            48, 38, 32, 0, 32, 0, 32,
        ]
    );
}

#[test]
fn test_encode_nlri_ip_vpn_mpls() {
    let nlri = NLRIEncoding::IP_VPN_MPLS((100, ("5.5.5.5".parse().unwrap(), 32).into(), 3200));
    let mut data: Vec<u8> = vec![];
    nlri.encode(&mut data).unwrap();
    assert_eq!(
        data,
        vec![0, 0, 12, 128, 0, 0, 0, 0, 0, 0, 0, 100, 5, 5, 5, 5]
    );
}

#[test]
fn test_encode_keepalive() {
    let keepalive = Message::KeepAlive;
    let mut data: Vec<u8> = vec![];
    keepalive.encode(&mut data).expect("Encoding KeepAlive");
    assert_eq!(
        data,
        vec![
            // preamble
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0,
            19, // length
            4,  // type
        ]
    );

    let message_data = encode_as_message(Message::KeepAlive);
    #[rustfmt::skip]
    assert_eq!(
        message_data[16..19],
        [0, 19, 4][..],
    );
}

#[test]
fn test_encode_notification() {
    let notification = Notification {
        major_err_code: 6,
        minor_err_code: 3,
        data: vec![],
    };
    let mut data: Vec<u8> = vec![];
    notification
        .encode(&mut data)
        .expect("Encoding Notification");
    assert_eq!(data, vec![6, 3]);

    let msg = "Peer De-Configured".to_string();
    let notification = Notification {
        major_err_code: 6,
        minor_err_code: 3,
        data: msg.into_bytes(),
    };
    let mut data: Vec<u8> = vec![];
    notification
        .encode(&mut data)
        .expect("Encoding Notification");
    assert_eq!(
        data,
        vec![
            6, 3, 80, 101, 101, 114, 32, 68, 101, 45, 67, 111, 110, 102, 105, 103, 117, 114, 101,
            100
        ]
    );

    let message_data = encode_as_message(Message::Notification(notification));
    #[rustfmt::skip]
    assert_eq!(
        message_data[16..19],
        [0, 39, 3][..],
    );
}

#[cfg(feature = "flowspec")]
#[test]
fn test_encode_flowspec_filter_prefix() {
    let filters = vec![
        FlowspecFilter::DestinationPrefix(("3001:4:b::10".parse().unwrap(), 128).into()),
        FlowspecFilter::SourcePrefix(("3001:1:a::10".parse().unwrap(), 128).into()),
    ];
    let nlri = NLRIEncoding::FLOWSPEC(filters);
    let mut data: Vec<u8> = vec![];
    nlri.encode(&mut data).expect("Encoding Flowspec NLRI");
    #[rustfmt::skip]
    assert_eq!(
        data,
        vec![
            38, // NLRI length
            1, // Dest prefix type
            128, 0, // prefix length & offset
            0x30, 0x01, 0, 0x04, 0, 0x0b, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x10,
            2, // Source prefix type
            128, 0, // prefix length & offset
            0x30, 0x01, 0, 0x01, 0, 0x0a, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x10
        ]
    );
}

#[cfg(feature = "flowspec")]
#[test]
fn test_encode_flowspec_filter_ports() {
    let filters = vec![
        FlowspecFilter::Port(vec![(NumericOperator::EQ, 80), (NumericOperator::EQ, 8080)]),
        FlowspecFilter::DestinationPort(vec![
            (NumericOperator::GT, 8080),
            (NumericOperator::LT | NumericOperator::AND, 8088),
            (NumericOperator::EQ, 3128),
        ]),
        FlowspecFilter::SourcePort(vec![(NumericOperator::GT, 1024)]),
    ];
    let nlri = NLRIEncoding::FLOWSPEC(filters);
    let mut data: Vec<u8> = vec![];
    nlri.encode(&mut data).expect("Encoding Flowspec NLRI");
    #[rustfmt::skip]
    assert_eq!(
        data,
        vec![
            0x14, // NLRI Length
            // Port
            0x04, 0x01, 0x50, 0x91, 0x1f, 0x90,
            // Dest Port
            0x05, 0x12, 0x1f, 0x90, 0x54, 0x1f, 0x98, 0x91, 0x0c, 0x38,
            // Source Port
            0x06, 0x92, 0x04, 0x00
        ]
    );
}
