#![no_std]
mod go_module_internal;
#[cfg(not(feature = "async"))]
pub use go_module_internal::go_module;
pub use go_module_internal::*;

#[cfg(feature = "async")]
pub use go_module_internal::go_module_async as go_module;
