# Filesystem Project Companions

This folder contains narrow project-form path and filesystem companions.

Many of these are tiny surface checks. They are useful, but they are not all
first-stop examples.

Current role rule:

* only the `path/file/directory` trio should be treated as frontdoor
* most `path_*` entries are companion-only micro-probes
* the rest of the subtree is narrow runtime/filesystem coverage, not showcase
  onboarding

## Start Here

If you only want the shortest route through this subtree, start with:

* [path_runtime_demo](path_runtime_demo)
* [file_runtime_demo](file_runtime_demo)
* [directory_runtime_demo](directory_runtime_demo)

## Pick By Goal

* frontdoor trio:
  [path_runtime_demo](path_runtime_demo)
* path companion micro-probe cluster:
  [path_copy_demo](path_copy_demo),
  [path_rename_demo](path_rename_demo),
  [path_remove_demo](path_remove_demo),
  [path_parent_demo](path_parent_demo),
  [path_depth_demo](path_depth_demo),
  [path_filename_demo](path_filename_demo),
  [path_stem_demo](path_stem_demo),
  [path_extension_demo](path_extension_demo)
* file I/O:
  [file_read_demo](file_read_demo),
  [file_runtime_demo](file_runtime_demo),
  [file_write_demo](file_write_demo),
  [file_copy_demo](file_copy_demo),
  [file_roundtrip_demo](file_roundtrip_demo)
  as the core std filesystem contract probes,
  [file_output_demo](file_output_demo),
  [filesystem_report_demo](filesystem_report_demo)
  as the filesystem/text std contract consumer,
  [filesystem_report_file_demo](filesystem_report_file_demo)
  as the std filesystem contract consumer
* directory and stat surfaces:
  [directory_runtime_demo](directory_runtime_demo),
  [directory_create_demo](directory_create_demo),
  [directory_remove_demo](directory_remove_demo),
  [directory_stat_demo](directory_stat_demo),
  [stat_runtime_demo](stat_runtime_demo),
  [fs_metadata_runtime_demo](fs_metadata_runtime_demo)
* window/pipe/fabric/handle-table runtime edges:
  [window_runtime_demo](window_runtime_demo),
  [pipe_runtime_demo](pipe_runtime_demo),
  [fabric_runtime_demo](fabric_runtime_demo),
  [handle_table_runtime_demo](handle_table_runtime_demo)

## Reading Rule

* use one representative path example and one representative file/directory
  example before going wider
* treat the `path companion micro-probe cluster` as companion-only unless you
  are actively working on path semantics
* treat the many `path_*` projects as narrow capability probes grouped around
  one shared path layer, not as separate front-door tutorials
* most `path_*` projects are narrow coverage probes rather than broad teaching
  examples
* if this cluster grows further, it is the strongest candidate for a future
  grouped subrouter rather than more top-level README emphasis
* for repo-level routing, prefer
  [docs/current-mainline-map.md](../../../docs/current-mainline-map.md)
