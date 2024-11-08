#![allow(
    clippy::module_name_repetitions,
    clippy::future_not_send, // false positive in WASM (single threaded) context
)]
// clippy WARN level lints
#![warn(
    // missing_docs, // TODO: bring this back
    clippy::cargo,
    clippy::pedantic,
    clippy::nursery,
    clippy::dbg_macro,
    // clippy::unwrap_used, // TODO: bring this back
    clippy::integer_division,
    clippy::large_include_file,
    clippy::map_err_ignore,
    // clippy::missing_docs_in_private_items, // TODO: bring this back
    clippy::panic,
    clippy::todo,
    clippy::undocumented_unsafe_blocks,
    clippy::unimplemented,
    clippy::unreachable
)]
// clippy WARN level lints, that can be upgraded to DENY if preferred
#![warn(
    clippy::float_arithmetic,
    clippy::integer_arithmetic,
    clippy::modulo_arithmetic,
    clippy::as_conversions,
    clippy::assertions_on_result_states,
    clippy::clone_on_ref_ptr,
    clippy::create_dir,
    clippy::default_union_representation,
    clippy::deref_by_slicing,
    clippy::empty_drop,
    clippy::empty_structs_with_brackets,
    clippy::exit,
    clippy::filetype_is_file,
    clippy::float_cmp_const,
    // clippy::if_then_some_else_none, // messes with something in the document html element
    clippy::indexing_slicing,
    clippy::let_underscore_must_use,
    clippy::lossy_float_literal,
    clippy::pattern_type_mismatch,
    clippy::string_slice,
    clippy::try_err
)]
// clippy DENY level lints, they always have a quick fix that should be preferred
#![deny(
    clippy::wildcard_imports,
    clippy::multiple_inherent_impl,
    clippy::rc_buffer,
    clippy::rc_mutex,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::same_name_method,
    clippy::self_named_module_files,
    clippy::separated_literal_suffix,
    clippy::shadow_unrelated,
    clippy::str_to_string,
    clippy::string_add,
    clippy::string_to_string,
    clippy::unnecessary_self_imports,
    clippy::unneeded_field_pattern,
    clippy::unseparated_literal_suffix,
    clippy::verbose_file_reads
)]

mod app;
mod errors;
mod constants;
mod sleep;
mod wrappers;
pub mod crypto;
pub mod media_devices;
pub mod encoders;
pub mod utils;
pub mod components;
pub mod models;
pub mod stores;

pub use app::{App, Route};
// pub use config::CONFIG;
pub use crate::errors::error::{LiveDocumentError as Error, LiveDocumentResult as Result};
