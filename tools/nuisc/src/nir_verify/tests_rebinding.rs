use super::verify_nir_module;
use crate::frontend::parse_nuis_module;

#[test]
fn fresh_binding_clears_prior_moved_state_for_same_name() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() {
            let buffer: ref Buffer = alloc_buffer(1, 7);
            let iteration: Bytes = copy_bytes(buffer);
            drop_bytes(iteration);
            let iteration: Bytes = copy_bytes(buffer);
            drop_bytes(iteration);
            free(buffer);
            return;
          }
        }
        "#,
    )
    .unwrap();

    verify_nir_module(&module).expect("a fresh same-name binding must have a fresh GLM identity");
}
