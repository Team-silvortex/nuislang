pub(super) fn c_string_literal(raw: &str) -> String {
    raw.replace('\\', "\\\\").replace('"', "\\\"")
}

pub(super) fn bytes_to_c_array(bytes: &[u8]) -> String {
    let mut out = String::new();
    for (index, byte) in bytes.iter().enumerate() {
        if index % 12 == 0 {
            out.push_str("\n    ");
        }
        out.push_str(&format!("0x{byte:02X}, "));
    }
    out.push('\n');
    out
}
