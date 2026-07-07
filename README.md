# csp — Rust constraint-satisfaction experiments

A set of small standalone **Rust** crates exploring constraint satisfaction (CSP).

## Crates
- **csp-engine** — a CSP solver with a small DSL to express variables and constraints,
  plus a cryptarithm solver. Uses `rayon` for parallel search and `rand`.
- **isa** — additional experimental crate.
- **color** — additional experimental crate.

## Build & run
Each crate is a standalone Cargo project (edition 2024):

```bash
cd csp-engine
cargo run
```
