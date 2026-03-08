//! Internal prelude for no_std compatibility
//! Re-exports common types from either std or alloc depending on features

pub use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};
