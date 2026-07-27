# Artifact Provider Metadata Scope

`nuis-artifact-provider-metadata-scope-v1` is the optional trace projection
envelope for the open `nuis-artifact-provider-metadata-v1` table.

The compiler, LinkPlan, and generic runtime preserve provider values without
interpreting package-owned keys. The scope layer only decides which values are
visible to a runtime trace.

## Syntax

An unscoped entry remains globally visible:

```text
nuis.pixelmagic:filter-plan=pixelmagic.gray8.threshold-only
```

A scoped entry adds one or both selectors:

```text
@scope(domain=shader)|nuis.package:key=value
@scope(trace=hetero-trace:shader:metal:apple-silicon-gpu)|nuis.package:key=value
@scope(domain=shader,trace=hetero-trace:shader:metal:apple-silicon-gpu)|nuis.package:key=value
```

Selectors are conjunctive. An entry with both selectors is visible only when
both the domain family and trace identity match. Projection preserves manifest
order and removes the `@scope(...)|` envelope before package dispatch.

## Compatibility

Unscoped entries use the original behavior and remain visible to every trace.
This keeps existing projects valid while allowing new projects to isolate
requests for multiple units.

The runtime records:

* the scope protocol and validation status
* the selected domain and trace
* the source table count
* the projected provider metadata count and ordered values

Malformed envelopes, empty selectors, unknown selector keys, duplicate
selectors, and invalid selector values fail during project or build-manifest
validation. Runtime projection also fails closed if an unverified table reaches
it.

## Ownership

The scope layer does not understand PixelMagic plan IDs or other package
semantics. After projection, each package adapter receives its original
provider value and performs its own authorization and hash validation.
