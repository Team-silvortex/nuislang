use super::*;
use crate::call_parameters::CpuCallParameterKind::Scalar;

fn fixture() -> (
    Node,
    BTreeMap<String, LlvmValueRef>,
    BTreeMap<String, CpuHelperSignature>,
) {
    let node = Node {
        name: "exit".into(),
        resource: "cpu0".into(),
        op: yir_core::Operation::parse(
            "cpu.guard_call_owned_struct_return",
            [
                "ready",
                "helper",
                "State{pixels:Bytes}",
                "buffer",
                "width",
                "transfer_bytes_v1=1",
                "saved",
                "cleanup_bytes_v1=1",
                "spare",
            ]
            .map(str::to_owned)
            .to_vec(),
        )
        .unwrap(),
    };
    let registers = BTreeMap::from([
        (
            "ready".into(),
            LlvmValueRef::Bool {
                i1: "%cond".into(),
                i64: "%cond64".into(),
            },
        ),
        (
            "buffer".into(),
            LlvmValueRef::BorrowedBuffer {
                ptr: "%buffer".into(),
                len: "%len".into(),
            },
        ),
        ("width".into(), LlvmValueRef::I64("%width".into())),
        (
            "saved".into(),
            LlvmValueRef::OwnedBytes {
                blob: "%saved".into(),
            },
        ),
        (
            "spare".into(),
            LlvmValueRef::OwnedBytes {
                blob: "%spare".into(),
            },
        ),
    ]);
    let signatures = BTreeMap::from([(
        "helper".into(),
        CpuHelperSignature {
            params: vec![
                Scalar(CpuCallScalarKind::BorrowedBuffer),
                Scalar(CpuCallScalarKind::I64),
                Scalar(CpuCallScalarKind::OwnedBytes),
            ],
            implicit_parameters: vec![],
            mutex_permit_params: vec![None; 3],
            ret: CpuCallScalarKind::I64,
            owned_struct_return: true,
            owned_struct_layout: Some(
                yir_core::parse_owned_struct_layout("State{pixels:Bytes}").unwrap(),
            ),
            native_value_return: None,
            owned_external_buffer_return: None,
        },
    )]);
    (node, registers, signatures)
}

#[test]
fn transfers_are_typed_pointer_arguments_and_cleanup_is_inside_the_taken_block() {
    let (node, registers, signatures) = fixture();
    let layout = yir_core::parse_owned_struct_layout(&node.op.args[2]).unwrap();
    let mut body = vec![];
    lower(
        &node,
        &mut body,
        &registers,
        &BTreeMap::new(),
        &signatures,
        CpuCallScalarKind::I64,
        Some(&layout),
        &mut 0,
        &mut 0,
    )
    .unwrap();
    let text = body.join("\n");
    let branch = text.find("br i1").unwrap();
    let cleanup = text
        .find("call void @nuis_scheduler_owned_blob_drop_v1(ptr %spare)")
        .unwrap();
    let call = text
        .find("call i64 @nuis_fn_helper(ptr %buffer, i64 %len, i64 %width, ptr %saved)")
        .unwrap();
    let returned = text.find("ret i64").unwrap();
    assert!(branch < cleanup && cleanup < call && call < returned);
    assert_eq!(text.matches("owned_blob_drop_v1").count(), 1);
    assert!(!text.contains("owned_blob_copy_v1"));
    assert!(body.last().unwrap().starts_with("guard_owned_call_cont"));
    assert!(
        matches!(registers.get("saved"), Some(LlvmValueRef::OwnedBytes { blob }) if blob == "%saved")
    );
}

#[test]
fn invalid_transfer_preflight_emits_no_cleanup_call_or_branch() {
    for case in 0..6 {
        let (node, mut registers, mut signatures) = fixture();
        match case {
            0 => {
                registers.remove("saved");
            }
            1 => {
                registers.insert("saved".into(), LlvmValueRef::Ptr("%saved".into()));
            }
            2 => {
                registers.insert(
                    "spare".into(),
                    LlvmValueRef::OwnedBytes {
                        blob: "%saved".into(),
                    },
                );
            }
            3 => {
                signatures.get_mut("helper").unwrap().params[2] = Scalar(CpuCallScalarKind::I64);
            }
            4 => {
                signatures
                    .get_mut("helper")
                    .unwrap()
                    .mutex_permit_params
                    .clear();
            }
            _ => {
                signatures
                    .get_mut("helper")
                    .unwrap()
                    .implicit_parameters
                    .push("ptr %context".into());
            }
        }
        let layout = yir_core::parse_owned_struct_layout(&node.op.args[2]).unwrap();
        let mut body = vec!["sentinel".into()];
        let mut reg = 11;
        let mut block = 17;
        assert!(
            lower(
                &node,
                &mut body,
                &registers,
                &BTreeMap::new(),
                &signatures,
                CpuCallScalarKind::I64,
                Some(&layout),
                &mut reg,
                &mut block
            )
            .is_err(),
            "{case}"
        );
        assert_eq!(body, ["sentinel"]);
        assert_eq!((reg, block), (11, 17));
    }
}

#[test]
fn owned_continuation_forwards_result_and_failure_without_a_branch_or_repacking() {
    let (mut node, registers, mut signatures) = fixture();
    node.op.instruction = "tail_call_owned_struct_return".into();
    node.op.args.remove(0);
    let layout = yir_core::parse_owned_struct_layout(&node.op.args[1]).unwrap();
    let mut body = vec![];
    lower(
        &node,
        &mut body,
        &registers,
        &BTreeMap::new(),
        &signatures,
        CpuCallScalarKind::I64,
        Some(&layout),
        &mut 0,
        &mut 0,
    )
    .unwrap();
    let text = body.join("\n");
    assert!(!text.contains("br i1"));
    assert!(!text.contains("aggregate_alloc"));
    assert!(!text.contains("aggregate_read"));
    assert!(
        text.find("owned_blob_drop_v1").unwrap() < text.find("call i64 @nuis_fn_helper").unwrap()
    );
    assert!(body.last().unwrap().trim().starts_with("ret i64"));
    signatures.get_mut("helper").unwrap().params[2] = Scalar(CpuCallScalarKind::I64);
    let mut body = vec!["sentinel".into()];
    assert!(lower(
        &node,
        &mut body,
        &registers,
        &BTreeMap::new(),
        &signatures,
        CpuCallScalarKind::I64,
        Some(&layout),
        &mut 0,
        &mut 0
    )
    .is_err());
    assert_eq!(body, ["sentinel"]);
}

#[test]
fn guarded_and_tail_calls_preserve_bool_captures_without_scalar_or_owner_coercion() {
    for tail in [false, true] {
        for case in 0..6 {
            let (mut node, mut registers, mut signatures) = fixture();
            node.op.args.insert(5, "flag".into());
            let flag = LlvmValueRef::Bool {
                i1: "%flag".into(),
                i64: "%flag64".into(),
            };
            registers.insert("flag".into(), flag.clone());
            let signature = signatures.get_mut("helper").unwrap();
            signature.params.insert(2, Scalar(CpuCallScalarKind::Bool));
            signature.mutex_permit_params.insert(2, None);
            match case {
                1 => {
                    registers.insert("flag".into(), LlvmValueRef::I64("1".into()));
                }
                2 => {
                    registers.insert("width".into(), flag);
                }
                3 => signature.params[2] = Scalar(CpuCallScalarKind::OwnedBytes),
                4 => signature.params[2] = Scalar(CpuCallScalarKind::I64),
                5 => signature.params[3] = Scalar(CpuCallScalarKind::Bool),
                _ => {}
            }
            let layout = yir_core::parse_owned_struct_layout(&node.op.args[2]).unwrap();
            if tail {
                node.op.instruction = "tail_call_owned_struct_return".into();
                node.op.args.remove(0);
            }
            let mut body = vec!["sentinel".into()];
            let mut reg = 11;
            let mut block = 17;
            let result = lower(
                &node,
                &mut body,
                &registers,
                &BTreeMap::new(),
                &signatures,
                CpuCallScalarKind::I64,
                Some(&layout),
                &mut reg,
                &mut block,
            );
            if case == 0 {
                assert!(result.unwrap());
                let text = body.join("\n");
                assert!(text.contains("call i64 @nuis_fn_helper(ptr %buffer, i64 %len, i64 %width, i1 %flag, ptr %saved)"));
                assert_eq!(text.contains("br i1"), !tail);
                assert_eq!(text.matches("owned_blob_drop_v1").count(), 1);
                assert!(!text.contains("aggregate_alloc"));
            } else {
                assert!(result.is_err(), "tail={tail} case={case}");
                assert_eq!(body, ["sentinel"]);
                assert_eq!((reg, block), (11, 17));
            }
        }
    }
}
