use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    str::FromStr,
};

use crate::http::{
    utils::{self, split_at_next},
    StatusCode,
};

/// Authority as defined in [RFC3986 Section
/// 3.2](https://datatracker.ietf.org/doc/html/rfc3986#section-3.2).
///
/// ```text
/// authority = [ userinfo "@" ] host [ ":" port ]
/// port = *DIGIT
/// ```
/// For information on `userinfo` or `host`, see [`UserInfo`]
/// or [`Host`] respectively.
#[derive(Debug, PartialEq)]
pub struct Authority {
    user_info: Option<UserInfo>,
    host: Host,
    port: Option<u16>,
}

impl Authority {
    /// Derive a [`Authority`] from a slice of bytes.
    ///
    /// Returns a [`StatusCode::BAD_REQUEST`] when the slice of bytes does not match the ABNF
    /// syntax of [`Authority`].
    pub fn from_bytes(src: &[u8]) -> Result<Self, StatusCode> {
        let (user_info, rest) = if let Some((info_bytes, rest)) = split_at_next(src, b'@') {
            (Some(UserInfo::from_bytes(info_bytes)?), rest)
        } else {
            (None, src)
        };

        if let Some(last_colon) = rest.iter().rposition(|b| *b == b':') {
            // might be a port
            // rest len - 5 as maximum 4 octets for the port and
            // 1 octet for the last_colon position
            if last_colon >= rest.len() - 5 {
                // last colon position is within the last 4 digits
                // which could be a valid port so try to parse Host
                // from slice before last colon then if successful
                // parse last octets as digits for port
                if let Ok(host) = Host::from_bytes(&rest[..last_colon]) {
                    // valid host so last octets should be port digits
                    if last_colon == rest.len() - 1 {
                        // empty port which is valid as syntax is:
                        // port = *DIGIT
                        return Ok(Authority {
                            user_info,
                            host,
                            port: Some(0),
                        });
                    }

                    let mut port = 0u16;
                    for digit in &rest[last_colon + 1..] {
                        if digit.is_ascii_digit() {
                            port = (port * 10) + (digit - b'0') as u16;
                        } else {
                            return Err(StatusCode::BAD_REQUEST);
                        }
                    }
                    return Ok(Authority {
                        user_info,
                        host,
                        port: Some(port),
                    });
                }
            }
        }

        // fallback to no port and only host
        let host = Host::from_bytes(rest)?;
        Ok(Authority {
            user_info,
            host,
            port: None,
        })
    }
}

/// A subcompont of [`Authority`]
///
/// ```text
/// userinfo = *( unreserved / pct-encoded / sub-delims / ":" )
///
/// unreserved = ALPHA / DIGIT / "-" / "." / "_" / "~"
/// pct-encoded = "%" HEXDIG HEXDIG
/// sub-delims = "!" / "$" / "&" / "'" / "(" / ")" / "*" / "+" / "," / ";" / "="
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct UserInfo(String);

impl UserInfo {
    /// Derive a [`UserInfo`] from a slice of bytes.
    ///
    /// Returns a [`StatusCode::BAD_REQUEST`] when the slice of bytes does not match the ABNF
    /// syntax of [`UserInfo`].
    pub fn from_bytes(src: &[u8]) -> Result<Self, StatusCode> {
        // SAFETY:
        // unreserved and sub-delims and ':' are all valid ascii characters
        // so the safety requirement of parse_pct_encoded_ext is satisfied.
        unsafe {
            utils::abnf::parse_pct_encoded_ext(src, |b| {
                utils::abnf::is_unreserved(b) || utils::abnf::is_sub_delims(b) || b == b':'
            })
        }
        .filter(|ui| ui.len() == src.len())
        .map(Self)
        .ok_or(StatusCode::BAD_REQUEST)
    }
}

/// Host type as defined in [RFC3986 Section
/// 3.2.2](https://datatracker.ietf.org/doc/html/rfc3986#section-3.2.2)
///
/// ```text
/// host = IP-literal / IPv4address / reg-name
///
/// IP-literal = "[" ( IPv6address / IPvFuture ) "]"
/// IPvFuture = "v" 1*HEXDIG "." 1*( unreserved / sub-delims / ":" )
/// IPv6address = // Implemented in Rust std::net::Ipv6Addr::from_str
/// IPv4address = // Implemented in Rust std::net::Ipv4Addr::from_str
///
/// reg-name = *( unreserved / pct-encoded / sub-delims )
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum Host {
    /// Contains a [`IpAddr`] abstraction over either a IPv4address or a
    /// IPv6address.
    IpvN(IpAddr),
    /// Contains a IpvFuture address - the number being the version.
    IpvFuture((u16, String)),
    /// Domain name string of the host.
    Domain(String),
}

impl Host {
    /// Derive a [`Host`] from a slice of bytes.
    ///
    /// Returns a [`StatusCode::BAD_REQUEST`] when the slice of bytes does not match the ABNF
    /// syntax of [`Host`].
    pub fn from_bytes(src: &[u8]) -> Result<Self, StatusCode> {
        if src.is_empty() {
            return Err(StatusCode::BAD_REQUEST);
        }

        match src {
            // IP-literal
            [b'[', b'v', ..] => {
                let ipv_raw = ipv_future_from_bytes(src)?;
                Ok(Host::IpvFuture(ipv_raw))
            }
            [b'[', rest @ .., b']'] => {
                let s = String::from_utf8_lossy(rest);
                if let Ok(addr) = Ipv6Addr::from_str(&s) {
                    Ok(Host::IpvN(addr.into()))
                } else {
                    Err(StatusCode::BAD_REQUEST)
                }
            }
            // IPv4address first then fall back on reg-name
            _ => {
                let c = String::from_utf8_lossy(src);
                if let Ok(addr) = Ipv4Addr::from_str(&c) {
                    Ok(Host::IpvN(addr.into()))
                } else {
                    // fall back to reg-name
                    utils::abnf::parse_reg_name(src)
                        .filter(|s| s.len() == src.len())
                        .ok_or(StatusCode::BAD_REQUEST)
                        .map(Host::Domain)
                }
            }
        }
    }
}

/// Parse sequence of octets to the components of IpvFuture
///
/// ```text
/// IPvFuture = "v" 1*HEXDIG "." 1*( unreserved / sub-delims / ":" )
/// ```
fn ipv_future_from_bytes(src: &[u8]) -> Result<(u16, String), StatusCode> {
    if let [b'[', b'v', rest @ .., b']'] = src {
        if let Some((version, [b'.', rest @ ..])) = utils::abnf::parse_hex_u16(rest) {
            // SAFETY:
            // unreserved and sub-delims and ':' are valid ascii characters
            // so the safety requirements of parse_seq are satisfied.
            let name = unsafe {
                utils::abnf::parse_seq(rest, |b| {
                    utils::abnf::is_unreserved(b) || utils::abnf::is_sub_delims(b) || b == b':'
                })
            }
            .filter(|s| !s.is_empty())
            .ok_or(StatusCode::BAD_REQUEST)?;
            return Ok((version, name));
        }
    }
    Err(StatusCode::BAD_REQUEST)
}

#[cfg(test)]
mod authority_tests {
    use std::net::Ipv4Addr;

    use super::{Authority, Host, StatusCode};

    fn assert_is_bad_request(bytes: &[u8]) {
        assert_eq!(Err(StatusCode::BAD_REQUEST), Authority::from_bytes(bytes));
    }

    #[test]
    fn empty_is_a_bad_request() {
        assert_is_bad_request(&[])
    }

    #[test]
    fn domain_name_with_too_large_port_is_a_bad_request() {
        assert_is_bad_request(b"example.com:50000");
    }

    #[test]
    fn ipv_future_is_valid_with_port() {
        assert_eq!(
            Ok(Authority {
                user_info: None,
                host: Host::IpvFuture((4, "2000:db8:ff00:32:1000".to_owned())),
                port: Some(8080),
            }),
            Authority::from_bytes(b"[v4.2000:db8:ff00:32:1000]:8080")
        );
    }

    #[test]
    fn ipv_future_is_valid_without_port() {
        assert_eq!(
            Ok(Authority {
                user_info: None,
                host: Host::IpvFuture((4, "2000:db8:ff00:32:1000".to_owned())),
                port: None,
            }),
            Authority::from_bytes(b"[v4.2000:db8:ff00:32:1000]")
        );
    }

    #[test]
    fn example_is_valid() {
        assert_eq!(
            Ok(Authority {
                user_info: None,
                host: Host::Domain("example.com".to_owned()),
                port: Some(8042)
            }),
            Authority::from_bytes(b"example.com:8042")
        );
    }

    #[test]
    fn domain_name_with_empty_port_is_valid() {
        assert_eq!(
            Ok(Authority {
                user_info: None,
                host: Host::Domain("example.com".to_owned()),
                port: Some(0),
            }),
            Authority::from_bytes(b"example.com:")
        );
    }

    #[test]
    fn ipv4_addr_with_port_is_valid() {
        assert_eq!(
            Ok(Authority {
                user_info: None,
                host: Host::IpvN(Ipv4Addr::LOCALHOST.into()),
                port: Some(80),
            }),
            Authority::from_bytes(b"127.0.0.1:80")
        );
    }
}

#[cfg(test)]
mod user_info_tests {
    use super::super::StatusCode;
    use super::UserInfo;

    fn assert_is_bad_request(bytes: &[u8]) {
        assert_eq!(Err(StatusCode::BAD_REQUEST), UserInfo::from_bytes(bytes));
    }

    fn assert_valid_user_info(user_info: &str) {
        assert_eq!(
            Ok(UserInfo(user_info.to_owned())),
            UserInfo::from_bytes(user_info.as_bytes())
        );
    }

    #[test]
    fn invalid_pct_encoded_is_a_bad_request() {
        assert_is_bad_request(b"%");
        assert_is_bad_request(b"%F");
        // HEXDIG is only valid when using uppercase letters!
        assert_is_bad_request(b"%1a");
    }

    #[test]
    fn invalid_ascii_char_is_a_bad_request() {
        assert_is_bad_request(b"@");
    }

    #[test]
    fn empty_userinfo_is_valid() {
        assert_valid_user_info("");
    }

    #[test]
    fn multiple_unreserved_chars_is_valid() {
        assert_valid_user_info("A9-3F.l6_o2~");
    }

    #[test]
    fn multiple_sub_delims_is_valid() {
        assert_valid_user_info("!*$+&,';(=)");
    }

    #[test]
    fn multiple_pct_encoded_is_valid() {
        assert_valid_user_info("%2F%9A%11%FF");
    }

    #[test]
    fn multiple_user_info_parts_is_valid() {
        assert_valid_user_info("%2B!*A22=(%108");
    }
}

#[cfg(test)]
mod host_tests {
    use super::{ipv_future_from_bytes, Host, StatusCode};
    use std::{
        fmt::Debug,
        net::{Ipv4Addr, Ipv6Addr},
        str::FromStr,
    };

    fn assert_is_bad_request<T>(right: Result<T, StatusCode>)
    where
        T: Debug + PartialEq,
    {
        assert_eq!(Err(StatusCode::BAD_REQUEST), right);
    }

    #[test]
    fn ipv_future_without_known_prefix_is_a_bad_request() {
        // known prefix is "[v"
        assert_is_bad_request(ipv_future_from_bytes(&[]));
        assert_is_bad_request(ipv_future_from_bytes(b"[a2.Hi]"));
        assert_is_bad_request(ipv_future_from_bytes(b"@vF.something]"));
        // Note prefix is case sensitive
        assert_is_bad_request(ipv_future_from_bytes(b"[V9.2001:db8:ff00:42:8329]"));
    }

    #[test]
    fn ipv_future_without_hex_dig_is_a_bad_request() {
        assert_is_bad_request(ipv_future_from_bytes(b"[v.2001:db8:ff00:32:1111]"));
        // G is not a valid hexdig
        assert_is_bad_request(ipv_future_from_bytes(b"[vG.2001:db8:ff00:32:1111]"));
    }

    #[test]
    fn ipv_future_version_that_overflows_u16_is_a_bad_request() {
        assert_is_bad_request(ipv_future_from_bytes(b"[vFFFF1.2001:db8:ff00:32:1122]"));
    }

    #[test]
    fn ipv_future_without_period_seperator_is_a_bad_request() {
        assert_is_bad_request(ipv_future_from_bytes(b"[vFF2001:db8:ff00:32:1111]"));
    }

    #[test]
    fn ipv_future_basic_success() {
        assert_eq!(
            Ok((9u16, "2022:dn8:aa23:74:2232".to_owned())),
            ipv_future_from_bytes(b"[v9.2022:dn8:aa23:74:2232]")
        );
    }

    #[test]
    fn ipv_future_accepts_mutliple_hex_dig_in_version() {
        assert_eq!(
            Ok((255u16, "2001:db7:ff00:32:4444".to_owned())),
            ipv_future_from_bytes(b"[vFF.2001:db7:ff00:32:4444]")
        );
    }

    #[test]
    fn empty_is_a_bad_request() {
        assert_is_bad_request(Host::from_bytes(&[]));
    }

    #[test]
    fn reserved_char_prefix_is_a_bad_request() {
        assert_is_bad_request(Host::from_bytes(b"@example.org"));
    }

    #[test]
    fn ipv_future_is_a_host() {
        assert_eq!(
            Ok(Host::IpvFuture((4, "2000:db8:ff00:32:1000".to_owned()))),
            Host::from_bytes(b"[v4.2000:db8:ff00:32:1000]")
        );
    }

    #[test]
    fn ipv6_addr_is_a_host() {
        let ipv6_addr = Ipv6Addr::from_str("2001:db8:aaaa:bbbb:cccc:dddd:eeee:0001").unwrap();
        assert_eq!(
            Ok(Host::IpvN(ipv6_addr.into())),
            Host::from_bytes(b"[2001:db8:aaaa:bbbb:cccc:dddd:eeee:0001]")
        );
    }

    #[test]
    fn ipv4_addr_is_a_host() {
        let ipv4_addr = Ipv4Addr::from_str("127.0.0.1").unwrap();
        assert_eq!(
            Ok(Host::IpvN(ipv4_addr.into())),
            Host::from_bytes(b"127.0.0.1")
        )
    }

    #[test]
    fn domain_name_is_a_host() {
        assert_eq!(
            Ok(Host::Domain("example.com".to_owned())),
            Host::from_bytes(b"example.com")
        );
    }
}
