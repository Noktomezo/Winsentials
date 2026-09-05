#![allow(
    clippy::too_many_lines,
    clippy::similar_names,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::wildcard_imports,
    clippy::struct_excessive_bools,
    clippy::too_many_arguments,
    clippy::ref_option,
    clippy::needless_pass_by_value
)]
rust_i18n::i18n!("../../locales");

pub mod components;
pub mod motion;
pub mod positioner;
pub mod theme;

pub use components::*;
pub use motion::*;
pub use positioner::*;
pub use theme::*;
