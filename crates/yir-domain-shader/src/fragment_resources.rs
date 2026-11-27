use crate::{
    ShaderFragmentStorage as Storage, ShaderFragmentUniform as Uniform,
    SHADER_FRAGMENT_STORAGE_CONTRACT, SHADER_FRAGMENT_UNIFORM_CONTRACT,
};
use yir_core::provider_runtime_ipc::DispatchArguments;

const MIXED: &str = "nuis-shader-fragment-uniform-storage-v1";

pub fn bind_fragment_resources(
    args: &mut DispatchArguments,
    uniform: Option<Uniform>,
    storage: Option<&Storage>,
) -> Result<(), String> {
    if !args.resources.is_empty() || !args.uploads.is_empty() {
        return Err("draw resources already bound".into());
    }
    match (uniform, storage) {
        (Some(u), Some(s)) => {
            if u.slot == s.capability.slot {
                return Err("fragment binding slots overlap".into());
            }
            let mut candidate = args.clone();
            let mut upload = args.clone();
            s.bind_dispatch(&mut upload)?;
            u.bind_dispatch(&mut candidate)?;
            candidate.uploads = upload.uploads;
            candidate.contract = MIXED.into();
            candidate.to_wire()?;
            *args = candidate;
            Ok(())
        }
        (Some(u), None) => u.bind_dispatch(args),
        (None, Some(s)) => s.bind_dispatch(args),
        (None, None) => Ok(()),
    }
}

pub fn decode_fragment_resources(
    args: &DispatchArguments,
) -> Result<(Option<Uniform>, Option<Storage>), String> {
    if args.contract == MIXED {
        args.to_wire()?;
        let mut part = args.clone();
        part.contract = SHADER_FRAGMENT_UNIFORM_CONTRACT.into();
        part.uploads.clear();
        let u = Uniform::from_dispatch(&part)?.ok_or("missing draw uniform")?;
        part.contract = SHADER_FRAGMENT_STORAGE_CONTRACT.into();
        part.resources.clear();
        part.uploads = args.uploads.clone();
        let s = Storage::from_dispatch(&part)?;
        if u.slot == s.capability.slot {
            return Err("fragment binding slots overlap".into());
        }
        Ok((Some(u), Some(s)))
    } else if args.contract == SHADER_FRAGMENT_STORAGE_CONTRACT {
        Ok((None, Some(Storage::from_dispatch(args)?)))
    } else {
        Ok((Uniform::from_dispatch(args)?, None))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use yir_core::provider_runtime_ipc::DispatchUpload;
    use yir_core::{DataWindow, ShaderBinding, StructValue, Value};

    fn packet_window() -> DataWindow {
        DataWindow {
            base: Box::new(Value::Struct(StructValue {
                type_name: "Packet".into(),
                fields: vec![
                    ("z".into(), Value::Int(-5)),
                    ("a".into(), Value::Int(1 << 24)),
                ],
            })),
            offset: 0,
            len: 1,
            immutable: true,
        }
    }

    fn uniform(window: DataWindow) -> Result<Uniform, String> {
        Uniform::from_binding(&ShaderBinding {
            kind: "uniform_binding".into(),
            slot: 2,
            value: Box::new(Value::DataWindow(window)),
        })
    }

    #[test]
    fn packet_fields_follow_declaration_order_and_reject_lossy_or_owned_values() {
        let words = uniform(packet_window()).unwrap().bytes;
        assert_eq!(
            words
                .chunks_exact(4)
                .map(|c| f32::from_le_bytes(c.try_into().unwrap()))
                .collect::<Vec<_>>(),
            [-5.0, 16777216.0, 0.0, 0.0]
        );
        for case in 0..8 {
            let mut w = packet_window();
            let Value::Struct(s) = w.base.as_mut() else {
                unreachable!()
            };
            match case {
                0 => w.immutable = false,
                1 => w.offset = 1,
                2 => w.len = 2,
                3 => s.fields[1].1 = Value::Int((1 << 24) + 1),
                4 => s.fields[1].1 = Value::OwnedBytes(vec![7]),
                5 => s.fields[1].0 = "z".into(),
                6 => s.fields.clear(),
                _ => s
                    .fields
                    .extend((0..3).map(|i| (i.to_string(), Value::Int(i)))),
            }
            assert!(uniform(w).is_err(), "case {case}");
        }
    }

    #[test]
    fn paired_dispatch_retains_both_resource_identities_and_rejects_aliases() {
        let mut args = crate::ShaderDrawArguments {
            width: 160,
            height: 120,
            vertex_count: 3,
            instance_count: 1,
        }
        .to_dispatch();
        let u = uniform(packet_window()).unwrap();
        let mut s = Storage {
            capability: crate::ShaderFragmentStorageCapability {
                slot: 2,
                element_count: 1,
            },
            upload: DispatchUpload::new("u32", vec![1], vec![7, 0, 0, 0]).unwrap(),
        };
        let before = args.clone();
        assert!(bind_fragment_resources(&mut args, Some(u), Some(&s)).is_err());
        assert_eq!(args, before);
        s.capability.slot = 3;
        bind_fragment_resources(&mut args, Some(u), Some(&s)).unwrap();
        assert_eq!(
            decode_fragment_resources(&args).unwrap(),
            (Some(u), Some(s))
        );
        let descriptor = DispatchArguments::parse(&args.to_wire().unwrap()).unwrap();
        assert_eq!(descriptor, args.descriptor());
        for case in 0..4 {
            let mut bad = descriptor.clone();
            match case {
                0 => bad.resources.clear(),
                1 => bad.uploads.clear(),
                2 => {
                    let upload = bad.uploads.remove("fragment.storage.3").unwrap();
                    bad.uploads.insert("fragment.storage.2".into(), upload);
                }
                _ => bad.contract = SHADER_FRAGMENT_UNIFORM_CONTRACT.into(),
            }
            assert!(decode_fragment_resources(&bad).is_err());
        }
    }
}
