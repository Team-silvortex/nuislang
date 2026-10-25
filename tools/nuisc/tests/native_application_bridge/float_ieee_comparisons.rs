use super::*;

fn source(ty: &str) -> String {
    format!("mod cpu Main {{
        struct State{{left:{ty},right:{ty},equal:bool,unequal:bool}}
        @noinline fn relay(value:State)->State{{return State{{left:value.left,right:value.right,equal:value.left==value.right,unequal:value.left!=value.right}};}}
        fn start(left:{ty},right:{ty})->State{{return State{{left:left,right:right,equal:left==right,unequal:left!=right}};}}
        fn step(state:State)->State{{return relay(state);}}
        fn stop(state:State)->State{{return state;}}
        fn main()->i64{{return 0;}}
    }}")
}

#[test]
fn f32_ieee_native_equality_matches_reference_for_nan_zeros_and_infinities() {
    let words = [
        0u64, 0x80000000, 0x3fc00000, 0xc0200000, 0x7f800000, 0xff800000, 0x7fc01234, 0xff800001, 1,
    ];
    let mut cases = Vec::new();
    for left in words {
        for right in words {
            let a = f32::from_bits(left as u32);
            let b = f32::from_bits(right as u32);
            let state = vec![left, right, u64::from(a == b), u64::from(a != b)];
            cases.push((vec![left, right], vec![state.clone(), state.clone(), state]));
        }
    }
    typed_record_inputs::check_flattened(&source("f32"), cases);
}

#[test]
fn f64_ieee_native_equality_matches_reference_for_nan_zeros_and_infinities() {
    let words = [
        0u64,
        0x8000000000000000,
        0x3ff8000000000000,
        0xc004000000000000,
        0x7ff0000000000000,
        0xfff0000000000000,
        0x7ff812345678abcd,
        0xfff0000000000001,
        1,
    ];
    let mut cases = Vec::new();
    for left in words {
        for right in words {
            let a = f64::from_bits(left);
            let b = f64::from_bits(right);
            let state = vec![left, right, u64::from(a == b), u64::from(a != b)];
            cases.push((vec![left, right], vec![state.clone(), state.clone(), state]));
        }
    }
    typed_record_inputs::check_flattened(&source("f64"), cases);
}
