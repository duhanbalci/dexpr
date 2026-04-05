mod debug_info;
pub mod error;
mod vm;

pub use debug_info::DebugInfo;
pub use error::VMError;
pub use vm::{ExternalFn, ExternalMethod, VM};
