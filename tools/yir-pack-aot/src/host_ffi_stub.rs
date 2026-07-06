use std::collections::BTreeMap;

use super::{HostFfiScalarType, HostFfiSignature};

pub(super) fn render_host_ffi_stubs(
    host_ffi_symbols: &BTreeMap<String, HostFfiSignature>,
) -> String {
    let mut out = String::new();
    for (symbol, signature) in host_ffi_symbols {
        if signature.abi == "libc" {
            continue;
        }
        out.push('\n');
        out.push_str(&render_host_ffi_stub(symbol, signature));
    }
    out
}

fn render_host_ffi_stub(symbol: &str, signature_info: &HostFfiSignature) -> String {
    let arg_count = signature_info.arg_count();
    let mut signature = String::new();
    if arg_count == 0 {
        signature.push_str("void");
    } else {
        for index in 0..arg_count {
            if index > 0 {
                signature.push_str(", ");
            }
            signature.push_str(&format!(
                "{} arg{index}",
                signature_info.arg_types[index].c_type()
            ));
        }
    }

    let body = if symbol.ends_with("color_bias") && arg_count >= 1 {
        "    return host_color_bias(arg0);".to_owned()
    } else if symbol.ends_with("speed_curve") && arg_count >= 1 {
        "    return host_speed_curve(arg0);".to_owned()
    } else if symbol.ends_with("radius_curve") && arg_count >= 1 {
        "    return host_radius_curve(arg0);".to_owned()
    } else if symbol.ends_with("mix_tick") && arg_count >= 2 {
        "    return host_mix_tick(arg0, arg1);".to_owned()
    } else if symbol == "host_network_connect_probe" && arg_count >= 3 {
        "    if (arg0 <= 0 || arg1 <= 0 || arg2 < 0) return 0;\n    return arg0 + arg1 + arg2;"
            .to_owned()
    } else if symbol == "host_network_open_tcp_stream" && arg_count >= 2 {
        "    if (arg0 <= 0 || arg1 < 0) return 0;\n    return arg0 + arg1 + 1;".to_owned()
    } else if matches!(
        symbol,
        "host_network_open_tcp_listener"
            | "host_network_bind_udp_datagram"
            | "host_network_accept_owned"
    ) && arg_count >= 3
    {
        "    if (arg0 <= 0 || arg1 < 0 || arg2 < 0) return 0;\n    return arg0 + arg1 + arg2 + 1;"
            .to_owned()
    } else if symbol == "host_network_open_udp_datagram" && arg_count >= 2 {
        "    if (arg0 <= 0 && arg1 <= 0) return 0;\n    return arg0 + arg1 + 1;".to_owned()
    } else if symbol == "host_network_close_owned" && arg_count >= 1 {
        "    return arg0 > 0 ? 1 : 0;".to_owned()
    } else if matches!(
        symbol,
        "host_network_send_owned" | "host_network_recv_owned"
    ) && arg_count >= 3
    {
        "    if (arg0 <= 0 || arg1 <= 0 || arg2 <= 0) return 0;\n    return arg0 + arg1 + arg2;"
            .to_owned()
    } else if symbol == "host_network_recv_http_status_owned" && arg_count >= 3 {
        "    if (arg0 <= 0 || arg1 <= 0 || arg2 <= 0) return 0;\n    return 200;".to_owned()
    } else if symbol == "host_network_accept_probe" && arg_count >= 3 {
        "    if (arg0 <= 0 || arg1 < 0 || arg2 < 0) return 0;\n    return arg0 + arg1 + arg2;"
            .to_owned()
    } else if symbol == "host_network_close" && arg_count >= 1 {
        "    return arg0 > 0 ? 1 : 0;".to_owned()
    } else if matches!(
        symbol,
        "host_network_send_probe" | "host_network_recv_probe"
    ) && arg_count >= 3
    {
        "    if (arg0 <= 0 || arg1 <= 0 || arg2 <= 0) return 0;\n    return arg0 + arg1 + arg2;"
            .to_owned()
    } else if arg_count == 0 {
        "    return 0;".to_owned()
    } else if arg_count == 1 {
        "    return arg0;".to_owned()
    } else {
        let mut expr = String::new();
        for index in 0..arg_count {
            if index > 0 {
                expr.push_str(" + ");
            }
            expr.push_str(&format!("arg{index}"));
        }
        format!("    return {expr};")
    };

    format!(
        "{} {symbol}({signature}) {{\n{body}\n}}\n",
        signature_info.return_type.c_type()
    )
}
