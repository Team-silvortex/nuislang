# Invalid Examples

These examples are supposed to fail verification or front-end checks.

Use this folder for:

* negative verifier coverage
* front-end structure failures
* task / ownership / payload boundary failures
* handwritten `YIR` invalid-shape probes

Canonical short map:

* [docs/current-mainline-map.md](../../docs/current-mainline-map.md)
  Use that file first when you want the shortest current route.

Subfolders:

* [ns/core](ns/core/README.md)
  invalid front-end structure and unit-binding examples
* [ns/memory](ns/memory/README.md)
  invalid front-end ownership/lifetime examples
* [ns](ns)
  invalid front-end examples
* [projects](projects)
  invalid multi-mod project examples
* [yir](yir)
  invalid handwritten `YIR` examples

Reading rule:

* use this README as a pure invalid-example router
* use the local invalid subdirectory README when you want the specific failure
  family
* use [docs/reference/cpu-task-payload-matrix.md](../../docs/reference/cpu-task-payload-matrix.md)
  when you want the current task payload allow/reject split behind the invalid
  memory cases
* use [docs/repo-cleanup-candidates.md](../../docs/repo-cleanup-candidates.md)
  when you want the current cleanup/archiving policy
