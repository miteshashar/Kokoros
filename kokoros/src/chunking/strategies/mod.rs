//! Chunking strategies implementation
//!
//! Provides the strategy pattern for different chunking approaches.
//! Currently implements Basic and Phonetics strategies as specified in Issue 1A.

use crate::chunking::config::ChunkingConfig;
use crate::chunking::errors::ChunkingResult;

pub mod basic;
pub mod phonetics;

pub use basic::BasicStrategy;
pub use phonetics::PhoneticsStrategy;

/// Core trait for chunking strategies
pub trait ChunkingStrategy: Send + Sync + std::fmt::Debug {
    /// Chunk text according to this strategy
    fn chunk(&self, text: &str, config: &ChunkingConfig<Self>) -> ChunkingResult<Vec<String>>
    where
        Self: Sized;

    /// Validate that chunks meet token limits
    fn validate_tokens(&self, chunks: &[String], limit: usize) -> bool;

    /// Get strategy name for error reporting
    fn name(&self) -> &'static str;

    /// Estimate tokens for a text segment (strategy-specific)
    fn estimate_tokens(&self, text: &str, config: &ChunkingConfig<Self>) -> ChunkingResult<usize>
    where
        Self: Sized;
}