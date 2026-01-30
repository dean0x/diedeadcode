//! Confidence scoring for dead code detection.

mod patterns;
mod scorer;

pub use scorer::score_dead_symbols;
