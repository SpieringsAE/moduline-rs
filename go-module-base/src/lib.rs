#![no_std]
mod go_module_internal;
pub use go_module_internal::{GoModule, GoModuleError, ModuleSetupError};
#[cfg(not(feature="async"))]
pub use go_module_internal::go_module;

#[cfg(feature = "async")]
pub use go_module_internal::go_module_async as go_module;