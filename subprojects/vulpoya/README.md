# vulpoya

`vulpoya` is hosted inside the `nuislang` tree for now, but it is not part of
the main Rust workspace and should not be treated as just another verifier
module inside `nuisc`.

Current intended role:

* independent analyzer / verifier companion project
* secondary review over `YIR`, `GLM`, contracts, and exported compiler facts
* future tooling peer in the same ecosystem, not a compiler pass disguised as a
  separate name

Current non-goals:

* do not collapse `vulpoya` into `tools/nuisc`
* do not make it responsible for the compiler's primary correctness layer
* do not bind it directly to unstable frontend implementation details when a
  semantic contract surface can exist instead

Suggested local layout:

* `docs/`
  analysis model, verifier scope, review pipeline
* `contracts/`
  imported semantic facts and validation schemas
* `analysis/`
  rule engines, dataflow checks, proof-oriented passes
* `tests/`
  fixture packs and secondary-review regressions

Current status:

* repository shell only
* hosted under `subprojects/` to keep protocol design close to `nuislang`
* still preserves its own nested `.git` history
