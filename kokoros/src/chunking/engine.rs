//! Chunking engine implementation
//!
//! Provides the main ChunkingEngine that orchestrates strategies and configuration
//! to deliver unified chunking interface as specified in Issue 1A.

use crate::chunking::config::{ChunkingConfig, TokenConfig, LanguageConfig, AdvancedFeatures};
use crate::chunking::errors::{ChunkingError, ChunkingResult};
use crate::chunking::strategies::{ChunkingStrategy, PhoneticsStrategy};
use std::marker::PhantomData;

/// Main chunking engine that provides unified interface
/// 
/// The engine coordinates strategy execution with configuration validation
/// and provides the single entry point for all chunking operations.
#[derive(Debug)]
pub struct ChunkingEngine<S: ChunkingStrategy> {
    config: ChunkingConfig<S>,
    strategy: S,
}

impl<S: ChunkingStrategy> ChunkingEngine<S> {
    /// Create a new chunking engine with the given configuration
    pub fn new(config: ChunkingConfig<S>, strategy: S) -> ChunkingResult<Self> {
        config.validate()?;
        Ok(Self { config, strategy })
    }

    /// Chunk text using the configured strategy
    pub fn chunk(&self, text: &str) -> ChunkingResult<Vec<String>> {
        // Input validation
        if text.is_empty() {
            return Ok(vec![]);
        }

        let sanitized = text.trim();
        if sanitized.is_empty() {
            return Ok(vec![]);
        }

        // Execute chunking strategy
        let chunks = self.strategy.chunk(sanitized, &self.config)?;

        // Validate results
        self.validate_chunks(&chunks)?;

        Ok(chunks)
    }

    /// Estimate tokens for text using the configured strategy
    pub fn estimate_tokens(&self, text: &str) -> ChunkingResult<usize> {
        if text.trim().is_empty() {
            return Ok(0);
        }
        
        self.strategy.estimate_tokens(text, &self.config)
    }

    /// Get the configuration
    pub fn config(&self) -> &ChunkingConfig<S> {
        &self.config
    }

    /// Get the strategy
    pub fn strategy(&self) -> &S {
        &self.strategy
    }

    /// Validate chunks meet all requirements
    fn validate_chunks(&self, chunks: &[String]) -> ChunkingResult<()> {
        let token_limit = self.config.token_config().effective_limit();

        for (index, chunk) in chunks.iter().enumerate() {
            // Check for empty chunks
            if chunk.trim().is_empty() {
                return Err(ChunkingError::text_processing(&format!(
                    "Chunk {} is empty after processing", index
                )));
            }

            // Validate token limit
            let tokens = self.strategy.estimate_tokens(chunk, &self.config)?;
            if tokens > token_limit {
                return Err(ChunkingError::token_limit_exceeded(
                    tokens,
                    token_limit,
                    self.config.token_config().safety_margin(),
                ));
            }
        }

        Ok(())
    }
}

// Default implementation for PhoneticsStrategy (preserves current behavior exactly)
impl Default for ChunkingEngine<PhoneticsStrategy> {
    /// Create default engine that preserves current `split_text_into_chunks()` behavior
    fn default() -> Self {
        let config = ChunkingConfig {
            strategy: PhantomData,
            token_config: TokenConfig::default(),      // 12 safety margin → 500 effective tokens
            language_config: LanguageConfig::default(), // English-only
            advanced_features: AdvancedFeatures::none(), // No advanced features for compatibility
        };
        
        let strategy = PhoneticsStrategy::new();
        
        Self::new(config, strategy).expect("Default configuration should always be valid")
    }
}

// Convenience constructors
impl ChunkingEngine<PhoneticsStrategy> {
    /// Create engine with phonetics strategy and default configuration
    pub fn with_phonetics() -> Self {
        Self::default()
    }

    /// Create engine with phonetics strategy and custom token limit
    pub fn with_phonetics_and_limit(safety_margin: usize) -> ChunkingResult<Self> {
        let config = ChunkingConfig::builder()
            .with_token_config(TokenConfig::new(safety_margin))?
            .with_language_config(LanguageConfig::default())?
            .build();
        
        let strategy = PhoneticsStrategy::new();
        Self::new(config, strategy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunking::strategies::BasicStrategy;

    #[test]
    fn test_default_engine_creation() {
        let engine = ChunkingEngine::default();
        assert_eq!(engine.config().token_config().effective_limit(), 500);
        assert_eq!(engine.config().token_config().safety_margin(), 12);
    }

    #[test]
    fn test_empty_text_handling() {
        let engine = ChunkingEngine::default();
        let result = engine.chunk("").unwrap();
        assert!(result.is_empty());

        let result = engine.chunk("   ").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_basic_strategy_engine() {
        let config = ChunkingConfig::builder()
            .with_token_config(TokenConfig::default()).unwrap()
            .with_language_config(LanguageConfig::default()).unwrap()
            .build();
        
        let strategy = BasicStrategy::new();
        let engine = ChunkingEngine::new(config, strategy).unwrap();
        
        let chunks = engine.chunk("Hello world. This is a test.").unwrap();
        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_token_estimation() {
        let engine = ChunkingEngine::default();
        let tokens = engine.estimate_tokens("Hello world").unwrap();
        assert!(tokens > 0);
    }
}