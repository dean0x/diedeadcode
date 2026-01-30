//! Built-in framework detectors.

mod express;
mod jest;
mod nextjs;
mod vitest;

pub use express::ExpressDetector;
pub use jest::JestDetector;
pub use nextjs::NextJsDetector;
pub use vitest::VitestDetector;
