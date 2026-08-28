use super::{project_with_modules, summarize_project_text_handle_rewrites};

#[test]
fn text_handle_rewrite_summary_lowers_the_resolved_closure_once() {
    let project = project_with_modules(vec![
        (
            "main.ns",
            r#"
            use cpu Helper;
            mod cpu Main {
              fn main() -> i64 {
                let buffer: ref Buffer = alloc_buffer(128, 0);
                let len: i64 = serialize_text_into("main", buffer, 0);
                let handle: i64 = deserialize_text_from(buffer, 0, len);
                return helper() + handle;
              }
            }
            "#,
        ),
        (
            "helper.ns",
            r#"
            mod cpu Helper {
              pub fn helper() -> i64 {
                let buffer: ref Buffer = alloc_buffer(128, 0);
                let len: i64 = serialize_text_into("helper", buffer, 0);
                return deserialize_text_from(buffer, 0, len);
              }
            }
            "#,
        ),
    ]);

    let summary = summarize_project_text_handle_rewrites(&project).unwrap();
    assert_eq!(summary.helper_hits, 1);
    assert_eq!(summary.local_hits, 1);
    assert_eq!(summary.total_hits(), 2);
}
