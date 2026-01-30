//! CLI command implementations.

pub mod analyze;
pub mod init;
pub mod watch;

pub use analyze::run_analyze;
pub use init::run_init;
pub use watch::run_watch;
