use std::collections::BTreeSet;

/// Versioned call-edge ownership. A guarded call preserves its fallthrough;
/// a terminal continuation consumes these owners unconditionally.
pub const CLEANUP_PREFIX: &str = "cleanup_bytes_v1=";
pub const TRANSFER_PREFIX: &str = "transfer_bytes_v1=";

pub struct GuardedOwnedCall<'a> {
    pub condition: &'a str,
    pub callee: &'a str,
    pub layout: &'a str,
    pub inputs: &'a [String],
    pub transfers: &'a [String],
    pub cleanup: &'a [String],
}

pub fn parse(args: &[String]) -> Result<GuardedOwnedCall<'_>, String> {
    if !(4..=46).contains(&args.len()) {
        return Err("guarded owned call has invalid arity".into());
    }
    parse_parts(&args[0], &args[1..])
}

/// A terminal continuation consumes its owners unconditionally; no boolean seed
/// or fabricated Result distinguishes it from an ordinary guarded exit.
pub fn parse_tail(args: &[String]) -> Result<GuardedOwnedCall<'_>, String> {
    if !(3..=45).contains(&args.len()) {
        return Err("owned tail call has invalid arity".into());
    }
    parse_parts("", args)
}

fn parse_parts<'a>(condition: &'a str, args: &'a [String]) -> Result<GuardedOwnedCall<'a>, String> {
    let split = args[2..]
        .iter()
        .position(|s| s.contains('='))
        .map_or(args.len(), |i| i + 2);
    let inputs = &args[2..split];
    let mut rest = &args[split..];
    let transfers = take_trailer(&mut rest, TRANSFER_PREFIX)?;
    let cleanup = take_trailer(&mut rest, CLEANUP_PREFIX)?;
    if !rest.is_empty() || transfers.len() + cleanup.len() > 8 {
        return Err("invalid guarded Bytes ownership trailers".into());
    }
    if !(1..=33).contains(&inputs.len())
        || args[0].is_empty()
        || !args[0]
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_')
        || crate::parse_owned_struct_layout(&args[1]).is_err()
    {
        return Err("invalid guarded owned helper signature metadata".into());
    }
    let mut seen = BTreeSet::new();
    if transfers.iter().chain(cleanup).any(|owner| {
        owner.is_empty()
            || owner == condition
            || inputs.contains(owner)
            || owner.contains('=')
            || !seen.insert(owner)
    }) {
        return Err("guarded Bytes ownership aliases a capture or repeats an owner".into());
    }
    Ok(GuardedOwnedCall {
        condition,
        callee: &args[0],
        layout: &args[1],
        inputs,
        transfers,
        cleanup,
    })
}

fn take_trailer<'a>(rest: &mut &'a [String], prefix: &str) -> Result<&'a [String], String> {
    let Some(marker) = rest.first().filter(|s| s.starts_with(prefix)) else {
        return Ok(&rest[..0]);
    };
    let count = marker[prefix.len()..]
        .parse::<usize>()
        .map_err(|_| "invalid guarded Bytes ownership count")?;
    if !(1..=8).contains(&count) || *marker != format!("{prefix}{count}") || rest.len() <= count {
        return Err("invalid guarded Bytes ownership extent".into());
    }
    let owners = &rest[1..=count];
    *rest = &rest[count + 1..];
    Ok(owners)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_continuation_has_no_condition_and_keeps_exact_owner_trailers() {
        let args = [
            "next",
            "State{pixels:Bytes}",
            "buffer",
            "width",
            "transfer_bytes_v1=1",
            "saved",
            "cleanup_bytes_v1=1",
            "spare",
        ]
        .map(str::to_owned);
        let call = parse_tail(&args).unwrap();
        assert_eq!(call.condition, "");
        assert_eq!(call.callee, "next");
        assert_eq!(call.inputs, ["buffer", "width"]);
        assert_eq!(call.transfers, ["saved"]);
        assert_eq!(call.cleanup, ["spare"]);
        for count in 0..3 {
            assert!(parse_tail(&args[..count]).is_err());
        }
        let mut bad = args.to_vec();
        bad[7] = "saved".into();
        assert!(parse_tail(&bad).is_err());
        bad[7] = "buffer".into();
        assert!(parse_tail(&bad).is_err());
    }

    #[test]
    fn cleanup_trailer_is_bounded_canonical_and_disjoint_from_captures() {
        let base = ["ready", "helper", "State{pixels:Bytes}", "buffer", "width"].map(str::to_owned);
        assert!(parse(&base).unwrap().cleanup.is_empty());
        let mut valid = base.to_vec();
        valid.extend(["cleanup_bytes_v1=2", "first", "second"].map(str::to_owned));
        let parsed = parse(&valid).unwrap();
        assert_eq!(parsed.inputs, &base[3..]);
        assert_eq!(parsed.cleanup, ["first", "second"]);
        for suffix in [
            vec!["cleanup_bytes_v1=0"],
            vec!["cleanup_bytes_v1=01", "first"],
            vec!["cleanup_bytes_v1=9"],
            vec!["cleanup_bytes_v1=2", "first"],
            vec!["cleanup_bytes_v1=2", "first", "first"],
            vec!["cleanup_bytes_v1=1", "buffer"],
            vec!["cleanup_bytes_v1=1", "ready"],
            vec!["cleanup_bytes_v1=1", "cleanup_bytes_v1=1"],
        ] {
            let mut bad = base.to_vec();
            bad.extend(suffix.into_iter().map(str::to_owned));
            assert!(parse(&bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn transfer_and_cleanup_form_a_bounded_disjoint_canonical_partition() {
        let base = ["ready", "helper", "State{pixels:Bytes}", "buffer", "width"].map(str::to_owned);
        let make = |suffix: &[&str]| {
            base.iter()
                .cloned()
                .chain(suffix.iter().map(|s| (*s).to_owned()))
                .collect::<Vec<_>>()
        };
        let args = make(&[
            "transfer_bytes_v1=1",
            "saved",
            "cleanup_bytes_v1=1",
            "spare",
        ]);
        let call = parse(&args).unwrap();
        assert_eq!(call.transfers, ["saved"]);
        assert_eq!(call.cleanup, ["spare"]);
        assert_eq!(call.inputs, &base[3..]);
        assert!(parse(&make(&["transfer_bytes_v1=1", "saved"]))
            .unwrap()
            .cleanup
            .is_empty());
        for suffix in [
            vec!["transfer_bytes_v1=0"],
            vec!["transfer_bytes_v1=01", "saved"],
            vec!["transfer_bytes_v1=2", "saved"],
            vec!["transfer_bytes_v1=2", "saved", "saved"],
            vec!["transfer_bytes_v1=1", "buffer"],
            vec!["transfer_bytes_v1=1", "ready"],
            vec![
                "transfer_bytes_v1=1",
                "saved",
                "cleanup_bytes_v1=1",
                "saved",
            ],
            vec![
                "cleanup_bytes_v1=1",
                "spare",
                "transfer_bytes_v1=1",
                "saved",
            ],
            vec!["transfer_bytes_v2=1", "saved"],
            vec![
                "transfer_bytes_v1=8",
                "a",
                "b",
                "c",
                "d",
                "e",
                "f",
                "g",
                "h",
                "cleanup_bytes_v1=1",
                "i",
            ],
        ] {
            assert!(parse(&make(&suffix)).is_err(), "{suffix:?}");
        }
    }
}
