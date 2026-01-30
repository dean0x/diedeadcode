//! Deadness analysis - finding unreachable code.

mod propagator;
mod transitive;

pub use propagator::find_dead_symbols;
