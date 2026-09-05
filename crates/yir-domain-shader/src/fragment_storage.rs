use yir_core::{
    provider_runtime_ipc::{DispatchArguments, DispatchUpload, MAX_UPLOAD_BYTES},
    ShaderBindingSet, Value,
};

pub const SHADER_FRAGMENT_STORAGE_CONTRACT: &str = "nuis-shader-fragment-storage-v1";
pub const FRAGMENT_STORAGE_CAPABILITY_MARKER: &str = "// nuis-fragment-storage-u32 ";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShaderFragmentStorageCapability {
    pub slot: usize,
    pub element_count: usize,
}

impl ShaderFragmentStorageCapability {
    pub fn validate(&self) -> Result<(), String> {
        if self.slot > 30 || !(1..=MAX_UPLOAD_BYTES / 4).contains(&self.element_count) {
            return Err("fragment storage slot or element budget is invalid".to_owned());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShaderFragmentStorage {
    pub capability: ShaderFragmentStorageCapability,
    pub upload: DispatchUpload,
}

impl ShaderFragmentStorage {
    pub fn from_bindings(bindings: &ShaderBindingSet) -> Result<Self, String> {
        let [binding] = bindings.bindings.as_slice() else {
            return Err("fragment storage requires exactly one binding".to_owned());
        };
        if !matches!(binding.kind.as_str(), "storage" | "storage_binding") {
            return Err("fragment storage requires a storage binding".to_owned());
        }
        let Value::OwnedBytes(elements) = binding.value.as_ref() else {
            return Err(
                "fragment u32 storage requires an owned integer buffer snapshot".to_owned(),
            );
        };
        let capability = ShaderFragmentStorageCapability {
            slot: binding.slot,
            element_count: elements.len(),
        };
        capability.validate()?;
        // Bytes currently snapshots i64 Buffer elements, not packed host memory.
        // Narrow explicitly, rejecting negative/out-of-range values before upload.
        let mut bytes = Vec::with_capacity(elements.len() * 4);
        for value in elements {
            let pixel = u32::try_from(*value)
                .map_err(|_| "fragment storage element is outside u32 range")?;
            bytes.extend_from_slice(&pixel.to_le_bytes());
        }
        Ok(Self {
            capability,
            upload: DispatchUpload::new("u32", vec![elements.len()], bytes)?,
        })
    }

    pub fn bind_dispatch(&self, arguments: &mut DispatchArguments) -> Result<(), String> {
        self.capability.validate()?;
        let mut candidate = arguments.clone();
        candidate.contract = SHADER_FRAGMENT_STORAGE_CONTRACT.to_owned();
        candidate.uploads.insert(
            format!("fragment.storage.{}", self.capability.slot),
            self.upload.clone(),
        );
        if Self::from_dispatch(&candidate)?.capability != self.capability {
            return Err("fragment storage layout differs from its upload".to_owned());
        }
        *arguments = candidate;
        Ok(())
    }

    pub fn from_dispatch(arguments: &DispatchArguments) -> Result<Self, String> {
        arguments.to_wire()?;
        if arguments.contract != SHADER_FRAGMENT_STORAGE_CONTRACT
            || !arguments.resources.is_empty()
            || arguments.uploads.len() != 1
        {
            return Err("unsupported fragment storage contract or resource count".to_owned());
        }
        let (name, upload) = arguments.uploads.first_key_value().unwrap();
        let slot = name
            .strip_prefix("fragment.storage.")
            .ok_or("invalid fragment storage name")?
            .parse::<usize>()
            .map_err(|_| "invalid fragment storage slot")?;
        let [element_count] = upload.shape.as_slice() else {
            return Err("fragment storage requires a contiguous u32 array".to_owned());
        };
        if name != &format!("fragment.storage.{slot}") || upload.element_type != "u32" {
            return Err("fragment storage name or element type is invalid".to_owned());
        }
        let capability = ShaderFragmentStorageCapability {
            slot,
            element_count: *element_count,
        };
        capability.validate()?;
        Ok(Self {
            capability,
            upload: upload.clone(),
        })
    }
}

pub fn fragment_storage_capability(
    source: &str,
) -> Result<Option<ShaderFragmentStorageCapability>, String> {
    let mut entries = source
        .lines()
        .filter_map(|line| line.strip_prefix(FRAGMENT_STORAGE_CAPABILITY_MARKER));
    let Some(entry) = entries.next() else {
        return Ok(None);
    };
    let (slot, count) = entry
        .split_once(':')
        .ok_or("invalid fragment storage capability")?;
    let result = ShaderFragmentStorageCapability {
        slot: slot
            .parse()
            .map_err(|_| "invalid storage capability slot")?,
        element_count: count
            .parse()
            .map_err(|_| "invalid storage capability count")?,
    };
    result.validate()?;
    if entries.next().is_some() || entry != format!("{}:{}", result.slot, result.element_count) {
        return Err("noncanonical or duplicate fragment storage capability".to_owned());
    }
    Ok(Some(result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use yir_core::{RenderPipeline, ShaderBinding};

    #[test]
    fn storage_snapshots_checked_u32_elements_without_exposing_buffer_memory() {
        let mut bindings = ShaderBindingSet {
            pipeline: RenderPipeline {
                shading_model: "image".to_owned(),
                topology: "triangle_strip".to_owned(),
            },
            bindings: vec![ShaderBinding {
                kind: "storage_binding".to_owned(),
                slot: 3,
                value: Box::new(Value::OwnedBytes(vec![0xff0000ff; 768])),
            }],
        };
        let storage = ShaderFragmentStorage::from_bindings(&bindings).unwrap();
        assert_eq!(storage.upload.payload().unwrap().len(), 3072);
        assert_eq!(&storage.upload.payload().unwrap()[..4], &[255, 0, 0, 255]);
        *bindings.bindings[0].value = Value::OwnedBytes(vec![0; 768]);
        assert_eq!(&storage.upload.payload().unwrap()[..4], &[255, 0, 0, 255]);
        let mut arguments = crate::ShaderDrawArguments {
            width: 160,
            height: 120,
            vertex_count: 3,
            instance_count: 1,
        }
        .to_dispatch();
        storage.bind_dispatch(&mut arguments).unwrap();
        let original = arguments.clone();
        let mut mismatched = storage.clone();
        mismatched.capability.element_count += 1;
        assert!(mismatched.bind_dispatch(&mut arguments).is_err());
        assert_eq!(
            arguments, original,
            "failed binding must not mutate dispatch"
        );
        assert!(arguments.to_wire().unwrap().len() < 256);
        assert_eq!(
            ShaderFragmentStorage::from_dispatch(&arguments).unwrap(),
            storage
        );
        assert!(
            ShaderFragmentStorage::from_dispatch(&arguments.descriptor())
                .unwrap()
                .upload
                .payload()
                .is_err()
        );
        for value in [
            Value::OwnedBytes(vec![]),
            Value::OwnedBytes(vec![-1]),
            Value::OwnedBytes(vec![i64::from(u32::MAX) + 1]),
            Value::Int(1),
        ] {
            *bindings.bindings[0].value = value;
            assert!(ShaderFragmentStorage::from_bindings(&bindings).is_err());
        }
        arguments
            .uploads
            .get_mut("fragment.storage.3")
            .unwrap()
            .element_type = "f32".to_owned();
        assert!(ShaderFragmentStorage::from_dispatch(&arguments).is_err());
    }

    #[test]
    fn storage_reflection_rejects_duplicate_noncanonical_or_unbounded_capabilities() {
        assert_eq!(
            fragment_storage_capability("// no resources").unwrap(),
            None
        );
        for suffix in [
            "31:768",
            "3:0",
            "3:4194305",
            "03:768",
            "3:0768",
            "3",
            "3:768\n// nuis-fragment-storage-u32 4:1",
        ] {
            assert!(fragment_storage_capability(&format!(
                "{FRAGMENT_STORAGE_CAPABILITY_MARKER}{suffix}"
            ))
            .is_err());
        }
    }
}
