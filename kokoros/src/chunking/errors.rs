//! Error types for the chunking system
//!
//! Provides structured error hierarchy with comprehensive error messages
//! and graceful fallback strategies as specified in Issue 1A.

/// Comprehensive error type for chunking operations
#[derive(Debug, thiserror::Error)]
pub enum ChunkingError {
    /// Phoneme conversion failed using espeak-rs
    #[error("Phoneme conversion failed for text '{text}': {source}")]
    PhonemeConversion {
        text: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Token limit exceeded during validation
    #[error("Token limit exceeded: {actual} tokens > {limit} limit (safety margin: {safety_margin})")]
    TokenLimitExceeded {
        actual: usize,
        limit: usize,
        safety_margin: usize,
    },

    /// Configuration validation failed
    #[error("Configuration validation failed: {reason}")]
    ConfigValidation { reason: String },

    /// Text processing failed
    #[error("Text processing failed: {reason}")]
    TextProcessing { reason: String },

    /// Strategy implementation error
    #[error("Strategy '{strategy}' failed: {reason}")]
    StrategyError { strategy: String, reason: String },

    /// Language not supported
    #[error("Language '{language}' not supported. Currently only English is supported due to vocabulary constraints.")]
    UnsupportedLanguage { language: String },

    /// Input validation error
    #[error("Invalid input: {reason}")]
    InvalidInput { reason: String },
}

impl ChunkingError {
    /// Create a phoneme conversion error
    pub fn phoneme_conversion(text: &str, source: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self::PhonemeConversion {
            text: text.to_string(),
            source,
        }
    }

    /// Create a token limit exceeded error
    pub fn token_limit_exceeded(actual: usize, limit: usize, safety_margin: usize) -> Self {
        Self::TokenLimitExceeded {
            actual,
            limit,
            safety_margin,
        }
    }

    /// Create a configuration validation error
    pub fn config_validation(reason: &str) -> Self {
        Self::ConfigValidation {
            reason: reason.to_string(),
        }
    }

    /// Create a text processing error
    pub fn text_processing(reason: &str) -> Self {
        Self::TextProcessing {
            reason: reason.to_string(),
        }
    }

    /// Create a strategy error
    pub fn strategy_error(strategy: &str, reason: &str) -> Self {
        Self::StrategyError {
            strategy: strategy.to_string(),
            reason: reason.to_string(),
        }
    }

    /// Create an unsupported language error
    pub fn unsupported_language(language: &str) -> Self {
        Self::UnsupportedLanguage {
            language: language.to_string(),
        }
    }

    /// Create an invalid input error
    pub fn invalid_input(reason: &str) -> Self {
        Self::InvalidInput {
            reason: reason.to_string(),
        }
    }
}

/// Result type alias for chunking operations
pub type ChunkingResult<T> = Result<T, ChunkingError>;