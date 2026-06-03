use std::collections::BTreeSet;

use nuis_semantics::model::{
    AstAttributeValue, AstExpr, AstFunction, AstModule, AstStmt, AstTypeRef,
};

pub(crate) fn validate_export_annotations(module: &AstModule) -> Result<(), String> {
    let mut seen_export_names = BTreeSet::new();
    for function in &module.functions {
        let export_name = function_export_name(function)?;
        let Some(export_name) = export_name else {
            continue;
        };
        if module.domain != "cpu" {
            return Err(format!(
                "function `{}::{}` can only use `@export(name = \"...\")` inside `mod cpu` in the current MVP",
                module.unit, function.name
            ));
        }
        if function.name != "main" {
            return Err(format!(
                "function `{}` uses `@export(name = \"{}\")`, but only `fn main()` can be exported in the current MVP",
                function.name, export_name
            ));
        }
        if !function.params.is_empty() {
            return Err(format!(
                "function `{}` uses `@export(name = \"{}\")`, but exported `fn main()` cannot take parameters in the current MVP",
                function.name, export_name
            ));
        }
        if !seen_export_names.insert(export_name.clone()) {
            return Err(format!(
                "module `{} {}` repeats exported symbol `{}`",
                module.domain, module.unit, export_name
            ));
        }
    }
    Ok(())
}

fn function_export_name(function: &AstFunction) -> Result<Option<String>, String> {
    let Some(attribute) = function
        .attributes
        .iter()
        .find(|attribute| attribute.name == "export")
    else {
        return Ok(None);
    };
    let Some(arg) = attribute.args.first() else {
        return Ok(None);
    };
    match &arg.value {
        AstAttributeValue::String(value) => Ok(Some(value.clone())),
        _ => Err(format!(
            "function `{}` annotation `@export` expects `name = \"...\"`",
            function.name
        )),
    }
}

pub(crate) fn function_host_symbol_name(function: &AstFunction) -> Result<Option<String>, String> {
    let Some(attribute) = function
        .attributes
        .iter()
        .find(|attribute| attribute.name == "host_symbol")
    else {
        return Ok(None);
    };
    let Some(arg) = attribute.args.first() else {
        return Ok(None);
    };
    match &arg.value {
        AstAttributeValue::String(value) => resolve_std_host_symbol(value)
            .map(|value| Some(value.to_owned()))
            .ok_or_else(|| {
                format!(
                    "function `{}` annotation `@host_symbol(\"{}\")` is not a recognized std-owned host symbol",
                    function.name, value
                )
            }),
        _ => Err(format!(
            "function `{}` annotation `@host_symbol` expects a string literal",
            function.name
        )),
    }
}

pub(crate) fn extern_function_symbol_name(
    function: &nuis_semantics::model::AstExternFunction,
) -> Result<String, String> {
    match &function.host_symbol {
        Some(symbol) => resolve_std_host_symbol(symbol)
            .map(str::to_owned)
            .ok_or_else(|| {
                format!(
                    "extern function `{}` host symbol `{}` is not a recognized std-owned host symbol",
                    function.name, symbol
                )
            }),
        None => Ok(match &function.interface {
            Some(interface) => format!("{interface}__{}", function.name),
            None => function.name.clone(),
        }),
    }
}

pub(crate) fn resolve_std_host_symbol(symbol: &str) -> Option<&'static str> {
    match symbol {
        "network.connect" => Some("host_network_connect_probe"),
        "network.open_tcp" => Some("host_network_open_tcp_stream"),
        "network.open_tcp_listener" => Some("host_network_open_tcp_listener"),
        "network.open_udp" => Some("host_network_open_udp_datagram"),
        "network.bind_udp" => Some("host_network_bind_udp_datagram"),
        "network.accept" => Some("host_network_accept_probe"),
        "network.accept_owned" => Some("host_network_accept_owned"),
        "network.close" => Some("host_network_close"),
        "network.close_owned" => Some("host_network_close_owned"),
        "network.send_owned" => Some("host_network_send_owned"),
        "network.recv_owned" => Some("host_network_recv_owned"),
        "network.recv_http_status_owned" => Some("host_network_recv_http_status_owned"),
        "network.send" => Some("host_network_send_probe"),
        "network.recv" => Some("host_network_recv_probe"),
        _ => None,
    }
}

pub(crate) fn validate_host_symbol_bridge_annotations(module: &AstModule) -> Result<(), String> {
    for function in &module.functions {
        let Some(attribute) = function
            .attributes
            .iter()
            .find(|attribute| attribute.name == "host_symbol")
        else {
            continue;
        };
        let Some(arg) = attribute.args.first() else {
            continue;
        };
        let AstAttributeValue::String(logical_symbol) = &arg.value else {
            continue;
        };
        if resolve_std_host_symbol(logical_symbol).is_none() {
            return Err(format!(
                "function `{}` annotation `@host_symbol(\"{}\")` is not a recognized std-owned host symbol",
                function.name, logical_symbol
            ));
        }
        if module.domain != "cpu" {
            return Err(format!(
                "function `{}::{}` can only use `@host_symbol(\"{}\")` inside `mod cpu` in the current MVP; prefer `extern \"c\" @host_symbol(...) fn ...;` for the stable host-boundary form",
                module.unit, function.name, logical_symbol
            ));
        }
        if function.is_async {
            return Err(format!(
                "function `{}` uses `@host_symbol(\"{}\")`, but async host bridge stubs are not supported in the current MVP; prefer `extern \"c\" @host_symbol(...) fn ...;` for stable host-boundary declarations",
                function.name, logical_symbol
            ));
        }
        if !function.generic_params.is_empty() {
            return Err(format!(
                "function `{}` uses `@host_symbol(\"{}\")`, but generic host bridge stubs are not supported in the current MVP; prefer `extern \"c\" @host_symbol(...) fn ...;` for stable host-boundary declarations",
                function.name, logical_symbol
            ));
        }
        if !function
            .params
            .iter()
            .all(|param| is_i64_type_ref(&param.ty))
        {
            return Err(format!(
                "function `{}` uses `@host_symbol(\"{}\")`, but host bridge stubs currently require all parameters to be `i64`; prefer the `extern \"c\" @host_symbol(...) fn ...;` form for the stable host-boundary path",
                function.name, logical_symbol
            ));
        }
        let Some(return_type) = &function.return_type else {
            return Err(format!(
                "function `{}` uses `@host_symbol(\"{}\")`, but host bridge stubs currently require `-> i64`; prefer the `extern \"c\" @host_symbol(...) fn ...;` form for the stable host-boundary path",
                function.name, logical_symbol
            ));
        };
        if !is_i64_type_ref(return_type) {
            return Err(format!(
                "function `{}` uses `@host_symbol(\"{}\")`, but host bridge stubs currently require `-> i64`; prefer the `extern \"c\" @host_symbol(...) fn ...;` form for the stable host-boundary path",
                function.name, logical_symbol
            ));
        }
        if !matches!(
            function.body.as_slice(),
            [AstStmt::Return(Some(AstExpr::Int(0)))]
        ) {
            return Err(format!(
                "function `{}` uses `@host_symbol(\"{}\")`, but host bridge stubs currently require a trivial `return 0;` body; prefer `extern \"c\" @host_symbol(...) fn ...;` for the stable host-boundary form",
                function.name, logical_symbol
            ));
        }
    }
    Ok(())
}

pub(crate) fn validate_extern_host_symbols(module: &AstModule) -> Result<(), String> {
    for function in &module.externs {
        validate_extern_host_symbol(module, function)?;
    }
    for interface in &module.extern_interfaces {
        for function in &interface.methods {
            validate_extern_host_symbol(module, function)?;
        }
    }
    Ok(())
}

fn validate_extern_host_symbol(
    module: &AstModule,
    function: &nuis_semantics::model::AstExternFunction,
) -> Result<(), String> {
    let Some(logical_symbol) = &function.host_symbol else {
        return Ok(());
    };
    if resolve_std_host_symbol(logical_symbol).is_none() {
        return Err(format!(
            "extern function `{}` annotation `@host_symbol(\"{}\")` is not a recognized std-owned host symbol",
            function.name, logical_symbol
        ));
    }
    if module.domain != "cpu" {
        return Err(format!(
            "extern function `{}::{}` can only use `@host_symbol(\"{}\")` inside `mod cpu` in the current MVP",
            module.unit, function.name, logical_symbol
        ));
    }
    if function.abi != "c" {
        return Err(format!(
            "extern function `{}` uses `@host_symbol(\"{}\")`, but std-owned host symbol externs currently require `extern \"c\"`",
            function.name, logical_symbol
        ));
    }
    if !function
        .params
        .iter()
        .all(|param| is_i64_type_ref(&param.ty))
        || !is_i64_type_ref(&function.return_type)
    {
        return Err(format!(
            "extern function `{}` uses `@host_symbol(\"{}\")`, but std-owned host symbol externs currently require only `i64` parameters and `-> i64`",
            function.name, logical_symbol
        ));
    }
    Ok(())
}

fn is_i64_type_ref(ty: &AstTypeRef) -> bool {
    ty.name == "i64" && ty.generic_args.is_empty() && !ty.is_optional && !ty.is_ref
}
