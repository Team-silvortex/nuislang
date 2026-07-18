use super::lower_nir_to_yir_builtin_cpu;
use crate::frontend::parse_nuis_module;

fn path_exists(yir: &yir_core::YirModule, from: &str, to: &str) -> bool {
    let mut frontier = vec![from.to_owned()];
    let mut seen = std::collections::BTreeSet::new();
    while let Some(current) = frontier.pop() {
        if current == to {
            return true;
        }
        if !seen.insert(current.clone()) {
            continue;
        }
        for edge in &yir.edges {
            if edge.from == current {
                frontier.push(edge.to.clone());
            }
        }
    }
    false
}

#[test]
fn lowers_explicit_buffer_copy_to_owned_bytes_yir() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(3, 7);
            let bytes: Bytes = copy_bytes(buffer);
            free(buffer);
            return 0;
          }
        }
        "#,
    )
    .expect("parse explicit owned Buffer copy");
    let yir = lower_nir_to_yir_builtin_cpu(&module).expect("lower owned Buffer copy");

    let copy = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "copy_buffer_owned")
        .expect("cpu.copy_buffer_owned node");
    let free = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "free")
        .expect("source Buffer free node");
    assert!(path_exists(&yir, &copy.name, &free.name));
}

#[test]
fn sequences_memory_lifecycle_around_task_result_observation() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn amplify(seed: i64) -> i64 {
            return seed + 30;
          }

          fn main() -> i64 {
            let head: ref Node = move(alloc_node(5, null()));
            let head_ref: ref Node = borrow(head);
            let seed: i64 = load_value(head_ref);
            borrow_end(head_ref);

            let scratch: ref Buffer = alloc_buffer(2, 0);
            store_at(scratch, 0, seed);

            let task: Task<i64> = spawn(amplify(seed));
            let observed: i64 = join(task);
            store_at(scratch, 1, observed);
            let replay: i64 = load_at(scratch, 0) + load_at(scratch, 1);
            free(scratch);
            free(head);
            return replay;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let borrow_end = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "borrow_end")
        .unwrap();
    let spawn = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task")
        .unwrap();
    let join = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "join")
        .unwrap();
    let stores = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "store_at")
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let frees = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "free")
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let load_ats = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "load_at")
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();

    assert!(path_exists(&yir, &borrow_end.name, &spawn.name));
    assert!(path_exists(&yir, &spawn.name, &join.name));
    assert!(stores
        .iter()
        .any(|store| path_exists(&yir, &join.name, store)));
    assert!(load_ats
        .iter()
        .all(|load| frees.iter().any(|free| path_exists(&yir, load, free))));
    assert!(frees
        .iter()
        .all(|free| stores.iter().any(|store| path_exists(&yir, store, free))));
}

#[test]
fn lowers_slice_backed_buffer_access_sequence() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(8, 0);
            let view: Slice<i64> = slice(buffer, 2, 3);
            view[1] = 9;
            let value: i64 = view[1];
            let size: i64 = view.len;
            let total: i64 = value + size;
            free(buffer);
            return total;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "struct"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "field"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "store_at"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "load_at"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "free"));
}

#[test]
fn lowers_subslice_backed_buffer_access_sequence() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(8, 0);
            let view: Slice<i64> = slice<i64>(buffer, 2, 5);
            let inner: Slice<i64> = subslice<i64>(view, 1, 2);
            let base: ref Buffer = slice_buffer(inner);
            let offset: i64 = slice_start(inner);
            let size: i64 = slice_len(inner);
            inner[0] = 7;
            let value: i64 = inner[0];
            free(buffer);
            return buffer_len(base) + offset + size + value;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let struct_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "struct")
        .count();
    let field_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "field")
        .count();
    assert!(struct_count >= 2, "expected slice + subslice struct nodes");
    assert!(field_count >= 3, "expected subslice field projections");
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "store_at"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "load_at"));
}

#[test]
fn lowers_slice_i32_backed_buffer_access_sequence_with_cast_nodes() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i32 {
            let buffer: ref Buffer = alloc_buffer(8, 0);
            let view: Slice<i32> = slice<i32>(buffer, 1, 2);
            let value: i32 = i32_from_i64(7);
            view[0] = value;
            let replay: i32 = view[0];
            free(buffer);
            return replay;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "cast_i32_to_i64"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "cast_i64_to_i32"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "store_at"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "load_at"));
}

#[test]
fn lowers_slice_bool_backed_buffer_access_sequence_with_cast_nodes() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> bool {
            let buffer: ref Buffer = alloc_buffer(8, 0);
            let view: Slice<bool> = slice<bool>(buffer, 1, 2);
            let value: bool = true;
            view[0] = value;
            let replay: bool = view[0];
            free(buffer);
            return replay;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "cast_bool_to_i64"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "cast_i64_to_bool"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "store_at"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "load_at"));
}

#[test]
fn lowers_slice_f32_backed_buffer_access_sequence_with_cast_nodes() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> f32 {
            let buffer: ref Buffer = alloc_buffer(8, 0);
            let view: Slice<f32> = slice<f32>(buffer, 1, 2);
            let value: f32 = 1.5;
            view[0] = value;
            let replay: f32 = view[0];
            free(buffer);
            return replay;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "cast_f32_to_i64"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "cast_i64_to_f32"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "store_at"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "load_at"));
}

#[test]
fn lowers_slice_f64_backed_buffer_access_sequence_with_cast_nodes() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> f64 {
            let buffer: ref Buffer = alloc_buffer(8, 0);
            let view: Slice<f64> = slice<f64>(buffer, 1, 2);
            let value: f64 = 1.5;
            view[0] = value;
            let replay: f64 = view[0];
            free(buffer);
            return replay;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "cast_f64_to_i64"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "cast_i64_to_f64"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "store_at"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "load_at"));
}

#[test]
fn lowers_bytes_and_subbytes_backed_buffer_access_sequence() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type Byte = i64;
          type ByteSlice = Slice<Byte>;

          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(8, 0);
            let view: ByteSlice = bytes(buffer, 1, 4);
            let inner: ByteSlice = subbytes(view, 1, 2);
            inner[0] = 72;
            let replay: Byte = inner[0];
            free(buffer);
            return replay + inner.len;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let struct_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "struct")
        .count();
    let field_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "field")
        .count();

    assert!(struct_count >= 2, "expected bytes + subbytes struct nodes");
    assert!(field_count >= 3, "expected subbytes field projections");
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "store_at"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "load_at"));
}

#[test]
fn sequences_network_session_packet_staging_memory_lifecycle() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_network_open_tcp_stream(
            remote_port: i64,
            connect_timeout_ms: i64
          ) -> i64;
          extern "c" fn host_network_send_owned(
            handle: i64,
            stream_window: i64,
            send_window: i64
          ) -> i64;
          extern "c" fn host_network_recv_owned(
            handle: i64,
            stream_window: i64,
            recv_window: i64
          ) -> i64;
          extern "c" fn host_network_recv_http_status_owned(
            handle: i64,
            stream_window: i64,
            recv_window: i64
          ) -> i64;
          extern "c" fn host_network_close_owned(handle: i64) -> i64;

          async fn session_step(seed: i64) -> i64 {
            let remote_port: i64 = network_profile_remote_port("NetworkUnit");
            let connect_timeout_ms: i64 = network_profile_connect_timeout("NetworkUnit");
            let stream_window: i64 = network_profile_stream_window("NetworkUnit");
            let recv_window: i64 = network_profile_recv_window("NetworkUnit");
            let send_window: i64 = network_profile_send_window("NetworkUnit");
            let handle: i64 = host_network_open_tcp_stream(remote_port, connect_timeout_ms);
            let send_result: NetworkResult<i64> =
              network_result(host_network_send_owned(handle, stream_window, send_window));
            let status_result: NetworkResult<i64> = network_result(
              host_network_recv_http_status_owned(handle, stream_window, recv_window)
            );
            let recv_result: NetworkResult<i64> =
              network_result(host_network_recv_owned(handle, stream_window, recv_window));
            let close_value: i64 = host_network_close_owned(handle);

            let scratch: ref Buffer = alloc_buffer(4, 0);
            store_at(scratch, 0, network_value(send_result));
            store_at(scratch, 1, network_value(status_result));
            store_at(scratch, 2, network_value(recv_result));
            store_at(scratch, 3, close_value);

            let staged_send: i64 = load_at(scratch, 0);
            let staged_status: i64 = load_at(scratch, 1);
            let staged_recv: i64 = load_at(scratch, 2);
            let staged_close: i64 = load_at(scratch, 3);
            let staged_total: i64 =
              staged_send + staged_status + staged_recv + staged_close;
            free(scratch);

            if network_send_ready(send_result) || network_recv_ready(status_result) {
              return seed + staged_total;
            }
            return seed + staged_close;
          }

          async fn main() -> i64 {
            return await session_step(0);
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let alloc = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "alloc_buffer")
        .unwrap();
    let stores = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "store_at")
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let loads = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "load_at")
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let free = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "free")
        .unwrap();
    let network_observes = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "network" && node.op.instruction == "observe")
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let network_values = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "network" && node.op.instruction == "value")
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "network" && node.op.instruction == "is_send_ready"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "network" && node.op.instruction == "is_recv_ready"));
    assert_eq!(stores.len(), 4);
    assert_eq!(loads.len(), 4);
    assert_eq!(network_observes.len(), 3);
    assert_eq!(network_values.len(), 3);

    assert!(stores
        .iter()
        .all(|store| path_exists(&yir, &alloc.name, store)));
    assert!(loads
        .iter()
        .all(|load| stores.iter().any(|store| path_exists(&yir, store, load))));
    assert!(loads.iter().all(|load| path_exists(&yir, load, &free.name)));
    assert!(network_values
        .iter()
        .all(|value| stores.iter().any(|store| path_exists(&yir, value, store))));
    assert!(network_observes
        .iter()
        .all(|observe| path_exists(&yir, observe, &free.name)));
}
