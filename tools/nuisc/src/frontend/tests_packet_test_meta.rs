use super::parse_nuis_module;

#[test]
fn rejects_unknown_function_annotation() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          @mystery
          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("unknown annotation `@mystery`"));
}

#[test]
fn rejects_unknown_struct_annotation() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          @mystery
          struct Packet {
            id: i64,
          }

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("struct `Packet` uses unknown annotation `@mystery`"));
}

#[test]
fn rejects_packet_field_outside_packet_struct() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Packet {
            @packet_field
            id: i64,
          }

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains(
        "annotation `@packet_field` requires parent struct `Packet` to also declare `@packet`"
    ));
}

#[test]
fn rejects_empty_packet_struct() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          @packet
          struct Packet {}

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("annotation `@packet` requires at least one field"));
}

#[test]
fn rejects_packet_struct_without_packet_fields() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          @packet
          struct Packet {
            id: i64,
          }

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("annotation `@packet` requires at least one `@packet_field`"));
}

#[test]
fn rejects_ref_fields_inside_packet_struct() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          @packet
          struct Packet {
            @packet_field
            payload: ref i64,
          }

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains(
        "annotation `@packet_field` currently only supports payload-role fields (role=unsupported-shape)"
    ));
}

#[test]
fn rejects_optional_fields_inside_packet_struct() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          @packet
          struct Packet {
            @packet_field
            payload: i64?,
          }

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains(
        "annotation `@packet_field` currently only supports payload-role fields (role=unsupported-shape)"
    ));
}

#[test]
fn rejects_marker_fields_inside_packet_struct() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          @packet
          struct Packet {
            @packet_field
            payload: Marker<Tag>,
          }

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("annotation `@packet_field` currently only supports payload-role fields (role=control-plane)"));
}

#[test]
fn accepts_packet_control_field_for_marker_field() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          @packet
          struct Packet {
            @packet_field
            payload: i64,
            @packet_control_field
            tag: Marker<Tag>,
          }

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    assert_eq!(module.structs.len(), 1);
    assert_eq!(module.structs[0].fields.len(), 2);
    assert_eq!(
        module.structs[0].fields[1].annotations[0].name,
        "packet_control_field"
    );
}

#[test]
fn rejects_result_fields_inside_packet_struct() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          @packet
          struct Packet {
            @packet_field
            payload: DataResult<i64>,
          }

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("annotation `@packet_field` currently only supports payload-role fields (role=async-carrier)"));
}

#[test]
fn rejects_task_fields_inside_packet_struct() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          @packet
          struct Packet {
            @packet_field
            payload: Task<i64>,
          }

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("annotation `@packet_field` currently only supports payload-role fields (role=async-carrier)"));
}

#[test]
fn rejects_thread_fields_inside_packet_struct() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          @packet
          struct Packet {
            @packet_field
            payload: Thread<i64>,
          }

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("annotation `@packet_field` currently only supports payload-role fields (role=async-carrier)"));
}

#[test]
fn rejects_mutex_fields_inside_packet_struct() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          @packet
          struct Packet {
            @packet_field
            seed: i64,
            @packet_control_field
            payload: Mutex<i64>,
          }

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains("role=sync-resource")
            || error.contains("synchronization-resource fields like `Mutex`"),
        "{error}"
    );
}

#[test]
fn rejects_handle_table_fields_inside_packet_struct() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          @packet
          struct Packet {
            @packet_field
            payload: HandleTable<Bindings>,
          }

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("annotation `@packet_field` currently only supports payload-role fields (role=control-plane)"));
}

#[test]
fn rejects_packet_control_field_on_payload_role_field() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          @packet
          struct Packet {
            @packet_field
            payload: i64,
            @packet_control_field
            extra: bool,
          }

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains(
        "annotation `@packet_control_field` currently only supports control-plane-role fields (role=payload)"
    ));
}

#[test]
fn rejects_field_with_both_packet_slots() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          @packet
          struct Packet {
            @packet_field
            @packet_control_field
            payload: Marker<Tag>,
          }

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("cannot use both `@packet_field` and `@packet_control_field`"));
}

#[test]
fn rejects_conflicting_inline_annotations() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          @inline
          @noinline
          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("cannot use both `@inline` and `@noinline`"));
}

#[test]
fn rejects_malformed_export_annotation() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          @export("main")
          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("annotation `@export` expects `name = \"...\"`"));
}

#[test]
fn accepts_scalar_export_annotation_on_non_main_function() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          @export(name = "entry_main")
          fn helper(value: i64) -> i64 {
            return value;
          }

          fn main() -> i64 {
            return helper(0);
          }
        }
        "#,
    )
    .unwrap();

    assert!(module
        .functions
        .iter()
        .any(|function| function.name == "helper"));
}

#[test]
fn rejects_non_scalar_export_function_boundary() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          @export(name = "entry_main")
          fn helper(value: String) -> i64 {
            return 0;
          }

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("parameters currently require scalar `i64`"));
}

#[test]
fn rejects_export_annotation_with_non_c_symbol_name() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          @export(name = "entry.main")
          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("requires a C-style symbol name"));
}

#[test]
fn rejects_malformed_host_symbol_annotation() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          @host_symbol(name = "network.open_tcp")
          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("annotation `@host_symbol` expects `@host_symbol(\"...\")`"));
}

#[test]
fn rejects_unknown_std_host_symbol_annotation() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          @host_symbol("network.future_magic")
          fn open_magic(value: i64) -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("is not a recognized std-owned host symbol"));
}

#[test]
fn rejects_non_c_extern_host_symbol_bridge() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "nurs" @host_symbol("network.open_tcp") fn open_tcp(local_port: i64, remote_port: i64) -> i64;
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("require `extern \"c\"`"));
}
