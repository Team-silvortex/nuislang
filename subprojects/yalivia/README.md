# yalivia

`yalivia` is hosted inside the `nuislang` tree for now, but it is not part of
the main Rust workspace and should not be treated as an internal crate/module
of `nuisc`.

Current intended role:

* lifecycle-oriented runtime companion project
* future execution loop / hook / RPC integration surface
* consumer of stable-enough `nuis` artifact and protocol boundaries

Current non-goals:

* do not merge `yalivia` into `tools/nuisc`
* do not make `yalivia` a hidden submodule of `crates/nuis-runtime`
* do not couple `yalivia` directly to unstable compiler internals when a
  contract surface can exist instead

Suggested local layout:

* `docs/`
  lifecycle model, RPC protocol notes, integration contracts
* `protocols/`
  transport and message schema drafts
* `runtime/`
  runtime loop, host integration, hook execution
* `tests/`
  contract/integration probes

Current status:

* repository shell only
* hosted under `subprojects/` to keep it close during rapid design changes
* still preserves its own nested `.git` history
