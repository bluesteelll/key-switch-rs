//! Configuration: `config.toml` loading, parsing, and conversion into the
//! runtime `Binding` representation.
//!
//! Layout:
//!   - `schema.rs`  — serde structs for the on-disk shape
//!   - `parsing.rs` — combo string and WM_* name parsers
//!   - `loader.rs`  — file I/O, default-config generation, conversion
//!
//! Only `loader` is re-exported; the rest are internal implementation detail.

mod loader;
mod parsing;
mod schema;

pub use loader::{default_config_path, load};
