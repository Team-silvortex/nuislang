mod codegen_wasm;
mod errors;
mod ir;
mod parser;

fn main() {
    let frontend = parser::frontend_name();
    let backend = codegen_wasm::backend_name();
    let module = ir::YirModule {
        version: "0.44.b-draft",
        profile: "aot",
    };
    let placeholder_error = errors::NuiscError {
        message: "prototype-only",
    };

    println!(
        "nuisc compiler prototype: topology-first scheduler frontend ({frontend} -> {backend}, yir={}, note={})",
        module.version,
        placeholder_error.message
    );
}
