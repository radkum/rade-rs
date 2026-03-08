//! Internal prelude for no_std compatibility
//! Re-exports common types from either std or alloc depending on features

pub use alloc::boxed::Box;
pub use alloc::format;
pub use alloc::string::{String, ToString};
pub use alloc::vec;
pub use alloc::vec::Vec;
