//! Framework detection plugins.

pub mod detector;
pub mod frameworks;
pub mod registry;

pub use detector::detect_frameworks;
pub use registry::FrameworkDetector;
