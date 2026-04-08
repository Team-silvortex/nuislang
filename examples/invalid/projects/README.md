# Invalid Project Examples

These project layouts are supposed to fail project-level checks.

Recommended checks:

```bash
cargo run -p nuis -- check examples/invalid/projects/bad_links_missing_data_use
cargo run -p nuis -- check examples/invalid/projects/bad_links_missing_data_plane
cargo run -p nuis -- check examples/invalid/projects/bad_links_missing_downlink
cargo run -p nuis -- check examples/invalid/projects/bad_shader_profile_missing_packet_shape
cargo run -p nuis -- check examples/invalid/projects/bad_data_profile_missing_payload_class
```
