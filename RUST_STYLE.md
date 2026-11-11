# Rust Style

House standard: [Google Rust Style Guide](https://google.github.io/styleguide/rustguide.html).
Applies to all Rust code in this repository.

## Enforcement

- `cargo fmt --check` — required (`rustfmt.toml` at repo root)
- `cargo clippy -- -D warnings` — required
- Both run in: `.github/workflows` (run locally via `act`, per AGENTS.md CI expectations)

## rustfmt

`rustfmt.toml` carries the stable-compatible subset of the guide:
`max_width = 100`, `reorder_modules = true`, `use_field_init_shorthand = true`,
`use_try_shorthand = true`.

Deliberately skipped (nightly-only, unavailable on our stable toolchain):
`imports_granularity`, `group_imports`, `format_code_in_doc_comments`,
`format_macro_matchers`, `format_strings`, `wrap_comments`, `normalize_comments`,
`normalize_doc_attributes`, `reorder_impl_items`, `match_block_trailing_comma`,
`hex_literal_case`, `short_array_element_width_threshold`.

## Deviations

- `#[rustfmt::skip]` only for manual alignment or generated code (per the guide).
- Any clippy allow must be explicit, minimally scoped, and justified in a comment.
