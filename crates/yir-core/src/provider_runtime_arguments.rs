use std::collections::BTreeMap;

/// Bounded, canonical scalar inputs. Domain semantics belong to the registered adapter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DispatchArguments {
    pub contract: String,
    pub scalars: BTreeMap<String, u64>,
}

impl DispatchArguments {
    pub fn to_wire(&self) -> Result<String, String> {
        if !identifier(&self.contract) || self.scalars.is_empty() || self.scalars.len() > 8 {
            return Err("runtime dispatch argument contract or count is invalid".to_owned());
        }
        let mut wire = self.contract.clone();
        for (name, value) in &self.scalars {
            if !identifier(name) {
                return Err("runtime dispatch argument name is invalid".to_owned());
            }
            wire.push_str(&format!("|{name}:u64:{value}"));
        }
        if wire.len() > 256 {
            return Err("runtime dispatch arguments exceed wire limit".to_owned());
        }
        Ok(wire)
    }

    pub fn parse(wire: &str) -> Result<Self, String> {
        if wire.len() > 256 {
            return Err("runtime dispatch arguments exceed wire limit".to_owned());
        }
        let mut fields = wire.split('|');
        let contract = fields.next().unwrap_or_default().to_owned();
        let mut scalars = BTreeMap::new();
        for field in fields {
            let (name, value) = field
                .split_once(":u64:")
                .ok_or("runtime dispatch argument type is invalid")?;
            let value = value
                .parse::<u64>()
                .map_err(|_| "runtime dispatch argument value is invalid")?;
            if scalars.insert(name.to_owned(), value).is_some() {
                return Err("runtime dispatch argument is duplicated".to_owned());
            }
        }
        let arguments = Self { contract, scalars };
        if arguments.to_wire()? != wire {
            return Err("runtime dispatch arguments are not canonical".to_owned());
        }
        Ok(arguments)
    }
}

fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arguments_are_typed_bounded_and_canonical() {
        let wire = "example.v1|count:u64:3|size:u64:10";
        assert_eq!(
            DispatchArguments::parse(wire).unwrap().to_wire().unwrap(),
            wire
        );
        for invalid in [
            "example.v1",
            "example.v1|count:i64:3",
            "example.v1|count:u64:03",
            "example.v1|count:u64:3|count:u64:4",
            "example.v1|size:u64:10|count:u64:3",
            "example.v1|count:u64:18446744073709551616",
            "example.v1\n|count:u64:3",
        ] {
            assert!(DispatchArguments::parse(invalid).is_err(), "{invalid}");
        }
        assert!(DispatchArguments::parse(&"x".repeat(257)).is_err());
    }
}
