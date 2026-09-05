use std::collections::BTreeMap;

#[path = "provider_runtime_resource.rs"]
mod resource;
pub use resource::DispatchResource;

#[path = "provider_runtime_upload.rs"]
mod upload;
pub use upload::{DispatchUpload, MAX_UPLOAD_BYTES};

/// Bounded, canonical inputs. Domain semantics belong to the registered adapter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DispatchArguments {
    pub contract: String,
    pub scalars: BTreeMap<String, u64>,
    pub resources: BTreeMap<String, DispatchResource>,
    pub uploads: BTreeMap<String, DispatchUpload>,
}

impl DispatchArguments {
    pub fn to_wire(&self) -> Result<String, String> {
        if !identifier(&self.contract)
            || self.scalars.is_empty()
            || self.scalars.len() > 8
            || self.resources.len() + self.uploads.len() > 4
        {
            return Err("runtime dispatch argument contract or count is invalid".to_owned());
        }
        let mut wire = self.contract.clone();
        for (name, value) in &self.scalars {
            if !identifier(name) {
                return Err("runtime dispatch argument name is invalid".to_owned());
            }
            wire.push_str(&format!("|{name}:u64:{value}"));
        }
        for (name, resource) in &self.resources {
            if !identifier(name) || self.scalars.contains_key(name) {
                return Err("runtime resource name is invalid or duplicated".to_owned());
            }
            wire.push_str(&format!("|{name}:{}", resource.to_wire()?));
        }
        let mut upload_bytes = 0usize;
        for (name, upload) in &self.uploads {
            if !identifier(name)
                || self.scalars.contains_key(name)
                || self.resources.contains_key(name)
            {
                return Err("runtime upload name is invalid or duplicated".to_owned());
            }
            wire.push_str(&format!("|{name}:{}", upload.to_wire()?));
            upload_bytes = upload_bytes
                .checked_add(upload.byte_length)
                .filter(|bytes| *bytes <= MAX_UPLOAD_BYTES)
                .ok_or("runtime upload total exceeds byte budget")?;
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
        let mut resources = BTreeMap::new();
        let mut uploads = BTreeMap::new();
        for field in fields {
            if let Some((name, value)) = field.split_once(":immutable-upload-le:") {
                if uploads
                    .insert(name.to_owned(), DispatchUpload::parse(value)?)
                    .is_some()
                {
                    return Err("runtime upload is duplicated".to_owned());
                }
                continue;
            }
            if let Some((name, value)) = field.split_once(":immutable-le:") {
                if resources
                    .insert(name.to_owned(), DispatchResource::parse(value)?)
                    .is_some()
                {
                    return Err("runtime resource is duplicated".to_owned());
                }
                continue;
            }
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
        let arguments = Self {
            contract,
            scalars,
            resources,
            uploads,
        };
        if arguments.to_wire()? != wire {
            return Err("runtime dispatch arguments are not canonical".to_owned());
        }
        Ok(arguments)
    }

    /// Replay and replies retain identity, not another copy of the input payload.
    pub fn descriptor(&self) -> Self {
        Self {
            uploads: self
                .uploads
                .iter()
                .map(|(name, upload)| (name.clone(), upload.descriptor()))
                .collect(),
            ..self.clone()
        }
    }

    pub fn matches_identity(&self, other: &Self) -> Result<bool, String> {
        Ok(self.to_wire()? == other.to_wire()?)
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

    #[test]
    fn immutable_resource_roundtrip_binds_type_shape_bytes_and_hash() {
        let mut arguments = DispatchArguments::parse("example.v1|count:u64:3").unwrap();
        let resource = DispatchResource {
            element_type: "f32".to_owned(),
            shape: vec![4],
            bytes: [1.0f32, 0.0, 0.0, 1.0]
                .into_iter()
                .flat_map(f32::to_le_bytes)
                .collect(),
        };
        arguments
            .resources
            .insert("fragment.uniform.2".to_owned(), resource);
        let wire = arguments.to_wire().unwrap();
        assert_eq!(DispatchArguments::parse(&wire).unwrap(), arguments);
        for invalid in [
            wire.replace(":f32:4:", ":u8:4:"),
            wire.replace(":f32:4:", ":f32:5:"),
            wire.replace("0000803f", "00000000"),
            wire.replace("0000803f", "0000803F"),
            wire.replace(":f32:4:", ":f32:04:"),
            wire.replace(":immutable-le:", ":mutable-le:"),
            format!("{wire}|{}", wire.split('|').next_back().unwrap()),
        ] {
            assert!(DispatchArguments::parse(&invalid).is_err(), "{invalid}");
        }
        for shape in [vec![], vec![0], vec![usize::MAX, usize::MAX], vec![1; 5]] {
            arguments
                .resources
                .get_mut("fragment.uniform.2")
                .unwrap()
                .shape = shape;
            assert!(arguments.to_wire().is_err());
        }
    }
}
