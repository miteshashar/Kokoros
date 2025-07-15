//! Unified Chunking Interface for Kokoros TTS
//!
//! This module provides a composable, type-safe chunking system that preserves existing behavior
//! while enabling advanced features through a strategy pattern design.
//!
//! ## Key Features
//! - **Phoneme-to-Token Accuracy**: Preserves existing espeak-rs integration and token limits
//! - **Strategy Pattern**: Extensible design for Basic, Phonetics, and Advanced chunking
//! - **Compile-time Safety**: Type system prevents invalid configurations
//! - **Backward Compatibility**: Default configuration produces identical output to current implementation
//!
//! ## Example Usage
//! ```rust
//! // Simple usage (preserves current behavior exactly)
//! let engine = ChunkingEngine::default();
//! let chunks = engine.chunk("Your text here")?;
//!
//! // Advanced configuration
//! let config = ChunkingConfig::builder()
//!     .with_strategy(PhoneticsStrategy)
//!     .with_token_config(TokenConfig::conservative())
//!     .with_advanced_features(AdvancedFeatures::basic())
//!     .build();
//! let engine = ChunkingEngine::new(config);
//! let chunks = engine.chunk("Your text here")?;
//! ```

pub mod config;
pub mod engine;
pub mod errors;
pub mod strategies;

// Re-export main types for convenient usage
pub use config::{ChunkingConfig, TokenConfig, LanguageConfig, AdvancedFeatures};
pub use engine::ChunkingEngine;
pub use errors::ChunkingError;
pub use strategies::{ChunkingStrategy, BasicStrategy, PhoneticsStrategy};