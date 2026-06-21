# `std/filesystem`

This directory is the reading router for the current
`std filesystem/path/location` lane.

Keep the actual recipe sources in
[stdlib/std](/Users/Shared/chroot/dev/nuislang/stdlib/std) for now; this file
exists to give the lane a cluster-shaped front door before any higher-risk
filesystem reshuffle.

Canonical companions:

* `std` layering rule:
  [std-mainline-layering-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/std-mainline-layering-contract.md)
* shortest repo-wide route:
  [current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
* project companions:
  [examples/projects/filesystem/README.md](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/README.md)
* state/location companions:
  [examples/projects/state/README.md](/Users/Shared/chroot/dev/nuislang/examples/projects/state/README.md)

## Current Lane Shape

Read the current lane in this order:

```text
path naming core
-> path structure and name parts
-> file and directory runtime edge
-> mutate and output helpers
-> location roots
-> location bundle
```

Current rule:

* keep `path_*` helpers grouped under one shared path lane instead of treating
  every small probe as its own front door
* read `directory/stat/file` as the host filesystem edge
* read `cwd/temp/home/location` as host-owned path roots and bundle helpers
* keep `kv/cache/config` in the broader persistence lane, not in this router

## Source Router

### Path Naming Core

* [path_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_runtime_recipe.ns)

### Path Structure And Name Parts

* [path_parent_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_parent_recipe.ns)
* [path_depth_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_depth_recipe.ns)
* [path_filename_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_filename_recipe.ns)
* [path_stem_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_stem_recipe.ns)
* [path_extension_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_extension_recipe.ns)
* [path_extension_is_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_extension_is_recipe.ns)

### File And Directory Runtime Edge

* [fs_metadata_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/fs_metadata_runtime_recipe.ns)
* [directory_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/directory_runtime_recipe.ns)
* [stat_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/stat_runtime_recipe.ns)
* [directory_stat_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/directory_stat_recipe.ns)
* [file_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/file_runtime_recipe.ns)
* [file_read_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/file_read_recipe.ns)
* [file_write_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/file_write_recipe.ns)
* [file_copy_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/file_copy_recipe.ns)
* [file_roundtrip_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/file_roundtrip_recipe.ns)
* report bridge:
  [path_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_runtime_recipe.ns)
  ->
  [directory_stat_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/directory_stat_recipe.ns)
  ->
  [file_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/file_runtime_recipe.ns)
  ->
  [filesystem_report_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/filesystem_report_recipe.ns)
  ->
  [filesystem_io_report_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/filesystem_io_report_recipe.ns)
  ->
  [filesystem_report_file_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/filesystem_report_file_recipe.ns)
  ->
  [benchmark_report_file_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/benchmark_report_file_recipe.ns)

### Mutate And Output Helpers

* [directory_create_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/directory_create_recipe.ns)
* [directory_remove_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/directory_remove_recipe.ns)
* [path_copy_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_copy_recipe.ns)
* [path_rename_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_rename_recipe.ns)
* [path_remove_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_remove_recipe.ns)
* [file_read_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/file_read_recipe.ns)
* [file_write_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/file_write_recipe.ns)
* [file_copy_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/file_copy_recipe.ns)
* [file_roundtrip_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/file_roundtrip_recipe.ns)
* [file_output_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/file_output_recipe.ns)

### Location Roots

* [cwd_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/cwd_runtime_recipe.ns)
* [temp_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/temp_runtime_recipe.ns)
* [home_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/home_runtime_recipe.ns)

### Location Bundle

* [location_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/location_runtime_recipe.ns)

## Companion Validation Router

Use the FFI and project companions as grouped mirrors instead of browsing every
small path probe first.

Shortest grouped route:

* source-level anchors:
  [hello_path_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_path_runtime_facades.ns),
  [hello_directory_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_directory_runtime_facades.ns),
  [hello_file_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_file_runtime_facades.ns),
  [hello_cwd_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_cwd_runtime_facades.ns),
  [hello_location_runtime_facades.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_location_runtime_facades.ns)
* project-form anchors:
  [path_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/path_runtime_demo),
  [file_read_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/file_read_demo),
  [file_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/file_runtime_demo),
  [file_write_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/file_write_demo),
  [file_copy_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/file_copy_demo),
  [directory_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/directory_runtime_demo),
  [filesystem_report_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/filesystem_report_demo),
  [filesystem_report_file_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/filesystem_report_file_demo),
  [benchmark_report_file_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/benchmark_report_file_demo),
  [filesystem_io_report_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/filesystem_io_report_demo),
  [cwd_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/cwd_runtime_demo),
  [location_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/location_runtime_demo)

Wider grouped route:

* path mutate probes:
  [path_copy_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/path_copy_demo),
  [path_rename_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/path_rename_demo),
  [path_remove_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/path_remove_demo)
* directory/file probes:
  [file_read_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/file_read_demo),
  [file_write_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/file_write_demo),
  [file_copy_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/file_copy_demo),
  [file_roundtrip_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/file_roundtrip_demo),
  [file_output_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/file_output_demo),
  [directory_create_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/directory_create_demo),
  [directory_remove_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/directory_remove_demo),
  [directory_stat_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/directory_stat_demo),
  [stat_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/stat_runtime_demo),
  [fs_metadata_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/fs_metadata_runtime_demo)
* location roots:
  [temp_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/temp_runtime_demo),
  [home_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/home_runtime_demo)

## Current Reading Rule

If you only want one pass:

1. start with [path_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/path_runtime_recipe.ns)
2. widen to [directory_stat_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/directory_stat_recipe.ns)
3. then read [file_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/file_runtime_recipe.ns)
4. then [filesystem_report_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/filesystem_report_recipe.ns)
5. then [filesystem_io_report_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/filesystem_io_report_recipe.ns)
6. then [filesystem_report_file_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/filesystem_report_file_recipe.ns)
7. end with [location_runtime_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/location_runtime_recipe.ns)

Short rule:

* path names first
* file/directory edge second
* mutation helpers after the base edge is clear
* location helpers last
