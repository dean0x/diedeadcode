//! Call graph construction from TypeScript/JavaScript files.

mod builder;
mod references;
mod symbols;

pub use builder::build_call_graph;
