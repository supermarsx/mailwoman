//! Geography and network-entry validation shared by settings and admission.
use crate::proxy::Cidr;
use std::net::IpAddr;

const ISO_3166: &str = "AD AE AF AG AI AL AM AO AQ AR AS AT AU AW AX AZ BA BB BD BE BF BG BH BI BJ BL BM BN BO BQ BR BS BT BV BW BY BZ CA CC CD CF CG CH CI CK CL CM CN CO CR CU CV CW CX CY CZ DE DJ DK DM DO DZ EC EE EG EH ER ES ET FI FJ FK FM FO FR GA GB GD GE GF GG GH GI GL GM GN GP GQ GR GS GT GU GW GY HK HM HN HR HT HU ID IE IL IM IN IO IQ IR IS IT JE JM JO JP KE KG KH KI KM KN KP KR KW KY KZ LA LB LC LI LK LR LS LT LU LV LY MA MC MD ME MF MG MH MK ML MM MN MO MP MQ MR MS MT MU MV MW MX MY MZ NA NC NE NF NG NI NL NO NP NR NU NZ OM PA PE PF PG PH PK PL PM PN PR PS PT PW PY QA RE RO RS RU RW SA SB SC SD SE SG SH SI SJ SK SL SM SN SO SR SS ST SV SX SY SZ TC TD TF TG TH TJ TK TL TM TN TO TR TT TV TW TZ UA UG UM US UY UZ VA VC VE VG VI VN VU WF WS YE YT ZA ZM ZW";

/// Whether `code` is an assigned ISO 3166-1 alpha-2 code (uppercase).
pub fn valid_country(code: &str) -> bool {
    code.len() == 2 && ISO_3166.split(' ').any(|c| c == code)
}
/// An allowlist entry: a CIDR network or a single address (an IPv6 single means
/// its /64). Private ranges are valid entries (an office); they have no country.
pub fn parse_ip_entry(raw: &str) -> Option<Cidr> {
    let raw = raw.trim();
    if raw.is_empty() || raw.len() > 64 {
        return None;
    }
    if !raw.contains('/')
        && let Ok(IpAddr::V6(v6)) = raw.trim_matches(['[', ']']).parse::<IpAddr>()
        && v6.to_ipv4_mapped().is_none()
    {
        return Cidr::parse(&format!("{v6}/64"));
    }
    Cidr::parse(raw)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn countries_must_be_assigned_iso_codes() {
        for ok in ["PT", "US", "GB", "DE"] {
            assert!(valid_country(ok), "{ok}");
        }
        for bad in ["pt", "XX", "ZZ", "USA", "P", "", "10", "AA"] {
            assert!(!valid_country(bad), "{bad}");
        }
        assert_eq!(ISO_3166.split(' ').count(), 249);
    }
}
