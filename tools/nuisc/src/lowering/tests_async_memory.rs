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
