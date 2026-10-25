# Examples

Start with a project when testing the complete toolchain path, a single `.ns`
source when checking language behavior, and handwritten YIR when checking
verifier/runtime/domain contracts. These roles are not interchangeable.

The source checkout does not ship generated example executables or bundles.
[examples/bins](bins/README.md) is a rebuild guide; build output belongs in ignored
`target/example-builds/`. Intentional input and inspection fixtures remain useful,
but they are not the source of truth for current executable behavior.

## Start Here

Use the [current mainline map](../docs/current-mainline-map.md),
[current snapshot](../docs/versioning/nuis-beta-0.17.0-snapshot.md) and
[validation checklist](../docs/versioning/nuis-beta-0.17.0-release-checklist.md)
for current direction and evidence boundaries. Earlier minor entries belong in
the [versioning history](../docs/versioning/README.md), not the shortest start path.

| Goal | Example | Boundary |
| --- | --- | --- |
| Host I/O | [filesystem_io_report_demo](projects/tooling/filesystem_io_report_demo), [stdin_runtime_demo](projects/tooling/stdin_runtime_demo). | Bounded native CLI input/output; not full streaming or arbitrary host compatibility. |
| Text or file processing | [cli_wc_demo](projects/tooling/cli_wc_demo), [cli_pgm_invert_demo](projects/tooling/cli_pgm_invert_demo). | Small checked inputs; the wc demo reads one 4096-byte chunk and uses ASCII separators. |
| Current image application | [ns_nova_image_showcase](projects/domains/ns_nova_image_showcase/README.md). | Real Metal pixel work and explicit AppKit sessions through an embedded-YIR host, not fully native CPU or private-Nsld application closure. |
| Smaller framework slice | [ns_nova_showcase](projects/domains/ns_nova_showcase). | Bounded frame/resource composition; see its own provider and platform requirements. |
| Linker pressure | [native_artifact_closure_demo](projects/tooling/native_artifact_closure_demo). | Inspect admission/finalizer evidence; a planned image is not automatically runnable. |
| Domain or control stress | [kernel_tensor_demo](projects/kernel_tensor_demo), [window_controls_demo](projects/window_controls_demo). | Registered hardware/profile requirements and source admission still apply. |

## Pick By Layer

| Layer | Router | Examples |
| --- | --- | --- |
| Source language | [ns](ns/README.md) | [hello_world.ns](ns/core/hello_world.ns), [hello_ref_struct.ns](ns/types/hello_ref_struct.ns), [GLM value path](ns/memory/hello_task_glm_value_path.ns). |
| Multi-file projects | [projects](projects/README.md) | Use the goal table above, then the local project README. |
| Handwritten YIR | [yir](yir/README.md) | [hello_yir.yir](yir/demos/hello_yir.yir), [data_fabric_demo.yir](yir/data/data_fabric_demo.yir), [kernel_tensor_demo.yir](yir/kernel/kernel_tensor_demo.yir). |
| Intentional rejection | [invalid](invalid/README.md) | Invalid syntax, type, memory and verifier fixtures are active tests, not broken demos. |
| Legacy material | [legacy](legacy/README.md) | Historical summaries, not current onboarding or compatibility claims. |

## Freshness Rule

The [freshness audit](../docs/examples-freshness-audit.md) distinguishes frontdoor,
companion, probe and legacy roles. Age alone does not justify deleting a fixture:
a regression for an old bug may still protect current behavior.

Promote runnable current projects into entry routes, retain feature companions
beside their owners, and label experiments or future sketches explicitly. Move
superseded onboarding out of the shortest path without deleting its historical
or regression purpose. File presence, metadata text and report output are not
substitutes for dispatched device evidence.
