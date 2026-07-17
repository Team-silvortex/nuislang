use super::lower_nir_to_yir_builtin_cpu;
use crate::frontend::parse_nuis_module;
use std::collections::BTreeMap;

#[test]
fn lowers_guard_return_with_async_survivor_binding_once() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Error {
            InvalidInput,
          }

          enum Result<T, E> {
            Ok(T),
            Err(E),
          }

          async fn work(seed: i64) -> i64 {
            return seed + 10;
          }

          fn fetch(seed: i64) -> Result<Task<i64>, Error> {
            let task: Task<i64> = spawn(work(seed));
            if seed > 0 {
              return Result.Ok(task);
            }
            return Result.Err(Error.InvalidInput);
          }

          async fn choose(seed: i64) -> Result<i64, Error> {
            let selected: Result<Task<i64>, Error> = fetch(seed);
            let value: i64 = await selected?;
            return Result.Ok(value + 1);
          }

          async fn main() -> Result<i64, Error> {
            return await choose(2);
          }
        }
        "#,
    )
    .unwrap();

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let guard_returns = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "guard_return")
        .count();
    let await_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "await")
        .count();
    let spawn_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task")
        .count();

    assert!(guard_returns >= 1, "expected Err path to stay guarded");
    assert_eq!(
        await_count, 2,
        "expected choose survivor task plus main helper call to await once each"
    );
    assert_eq!(spawn_count, 1, "expected fetch to spawn one task");
}

#[test]
fn lowers_try_result_variant_payload_guard_return_survivor() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Error {
            Missing,
          }

          enum Result<T, E> {
            Ok(T),
            Err(E),
          }

          fn fetch(seed: i64) -> Result<i64, Error> {
            if seed > 0 {
              return Result.Ok(seed);
            }
            return Result.Err(Error.Missing);
          }

          fn combine(lhs: Result<i64, Error>, rhs: Result<i64, Error>) -> Result<i64, Error> {
            let lhs_value: i64 = lhs?;
            let rhs_value: i64 = rhs?;
            return Result.Ok(lhs_value + rhs_value);
          }

          fn main() -> Result<i64, Error> {
            return combine(fetch(3), fetch(5));
          }
        }
        "#,
    )
    .unwrap();

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let guard_returns = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "guard_return")
        .count();
    let selects = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();
    let variant_fields = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "variant_field")
        .count();

    assert!(
        selects >= 2,
        "expected both `?` error paths to select between Err return and Ok survivor"
    );
    assert!(
        guard_returns <= 1,
        "expected pure Result `?` survivor paths to avoid whole-lane guard_return nodes"
    );
    assert!(
        variant_fields >= 2,
        "expected Result payload access to lower through variant fields"
    );
}

#[test]
fn lowers_dynamic_try_result_continuation_as_result_level_select() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "c" fn host_argv_count() -> i64;

          enum Error {
            Missing,
          }

          enum Result<T, E> {
            Ok(T),
            Err(E),
          }

          fn fetch(seed: i64) -> Result<i64, Error> {
            if seed > 0 {
              return Result.Ok(seed);
            }
            return Result.Err(Error.Missing);
          }

          fn combine(lhs: Result<i64, Error>, rhs: Result<i64, Error>) -> Result<i64, Error> {
            let lhs_value: i64 = lhs?;
            let rhs_value: i64 = rhs?;
            return Result.Ok(lhs_value + rhs_value);
          }

          fn main() -> i64 {
            let seed: i64 = host_argv_count() - 1;
            let result: Result<i64, Error> = combine(fetch(seed), fetch(3));
            match result {
              Result.Ok(_) => {
                return 1;
              }
              Result.Err(_) => {
                return 0;
              }
            }
          }
        }
        "#,
    )
    .unwrap();

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let nodes = yir
        .nodes
        .iter()
        .map(|node| (node.name.as_str(), node))
        .collect::<BTreeMap<_, _>>();
    let struct_vs_payload_selects = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .filter(|node| {
            let Some(then_node) = nodes.get(node.op.args[1].as_str()) else {
                return false;
            };
            let Some(else_node) = nodes.get(node.op.args[2].as_str()) else {
                return false;
            };
            matches!(
                (
                    then_node.op.instruction.as_str(),
                    else_node.op.instruction.as_str()
                ),
                ("struct", "variant_field") | ("variant_field", "struct")
            )
        })
        .count();
    let result_level_selects = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .filter(|node| {
            let then_is_result = nodes.get(node.op.args[1].as_str()).is_some_and(|branch| {
                branch.op.args.first().is_some_and(|arg| {
                    arg.starts_with("Result.Err") || arg.starts_with("Result.Ok")
                })
            });
            let else_is_result = nodes.get(node.op.args[2].as_str()).is_some_and(|branch| {
                branch.op.args.first().is_some_and(|arg| {
                    arg.starts_with("Result.Err") || arg.starts_with("Result.Ok")
                }) || branch.op.instruction == "select"
            });
            then_is_result && else_is_result
        })
        .count();

    assert_eq!(
        struct_vs_payload_selects, 0,
        "dynamic `?` continuation must not select between an Err struct and Ok payload"
    );
    assert!(
        result_level_selects >= 1,
        "expected dynamic `?` continuation to select whole Result variants"
    );
}
