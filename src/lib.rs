//! diedeadcode - Conservative TypeScript dead code detection.
//!
//! This crate provides dead code detection with transitive analysis
//! and confidence scoring for TypeScript/JavaScript codebases.

pub mod analysis;
pub mod cli;
pub mod config;
pub mod core;
pub mod plugins;

pub use analysis::Analyzer;
pub use config::Config;
pub use core::{AnalysisResult, CallGraph, Confidence, DeadSymbol};
