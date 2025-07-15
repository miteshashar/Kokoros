//! Configuration system for the chunking interface
//!
//! Provides compile-time safe configuration with builder pattern
//! and comprehensive validation as specified in Issue 1A.

use crate::chunking::errors::{ChunkingError, ChunkingResult};
use std::marker::PhantomData;

/// Token configuration with safety margins
#[derive(Debug, Clone, PartialEq)]
pub struct TokenConfig {
    /// Safety margin: tokens reserved below model limit (default: 12)
    safety_margin: usize,
}

impl TokenConfig {
    /// Model token limit (Kokoro's 512-token limit)
    pub const MODEL_LIMIT: usize = 512;

    /// Create new token configuration with custom safety margin
    pub fn new(safety_margin: usize) -> Self {
        Self { safety_margin }
    }

    /// Conservative configuration with extra safety margin
    pub fn conservative() -> Self {
        Self::new(20)
    }

    /// Get the effective token limit (MODEL_LIMIT - safety_margin)
    pub fn effective_limit(&self) -> usize {
        Self::MODEL_LIMIT.saturating_sub(self.safety_margin)
    }

    /// Get the model's absolute limit
    pub fn model_limit(&self) -> usize {
        Self::MODEL_LIMIT
    }

    /// Get the safety margin
    pub fn safety_margin(&self) -> usize {
        self.safety_margin
    }

    /// Validate the configuration
    pub fn validate(&self) -> ChunkingResult<()> {
        if self.safety_margin >= Self::MODEL_LIMIT {
            return Err(ChunkingError::config_validation(&format!(
                "Safety margin {} must be less than model limit {}",
                self.safety_margin,
                Self::MODEL_LIMIT
            )));
        }
        Ok(())
    }
}

impl Default for TokenConfig {
    /// Default configuration preserving current behavior: 500 effective tokens (12 safety margin)
    fn default() -> Self {
        Self::new(12)
    }
}

/// Language configuration (English-only for Phase 1)
#[derive(Debug, Clone, PartialEq)]
pub struct LanguageConfig {
    language: Language,
    phoneme_validation: bool,
}

impl LanguageConfig {
    /// Create new language configuration
    pub fn new(language: Language) -> Self {
        Self {
            language,
            phoneme_validation: true, // Always enabled for accuracy
        }
    }

    /// Get the configured language
    pub fn language(&self) -> &Language {
        &self.language
    }

    /// Check if phoneme validation is enabled
    pub fn phoneme_validation(&self) -> bool {
        self.phoneme_validation
    }

    /// Validate the configuration
    pub fn validate(&self) -> ChunkingResult<()> {
        match self.language {
            Language::English => Ok(()),
            // Future languages would be validated here
        }
    }
}

impl Default for LanguageConfig {
    /// Default to English-only with phoneme validation
    fn default() -> Self {
        Self::new(Language::English)
    }
}

/// Supported languages (English-only for Phase 1)
#[derive(Debug, Clone, PartialEq)]
pub enum Language {
    /// English language (only supported language)
    /// Adding non-English languages requires:
    /// - Vocabulary expansion to include accented characters (é, ñ, ü, ç, etc.)
    /// - Character normalization pipeline  
    /// - Proper unicode handling in tokenization
    /// - Language-specific phonemizers (Pypinyin, MeCab, Indic NLP)
    /// - Dynamic phonemizer dispatch architecture
    /// - eSpeak-ng quality validation for European languages
    English,
}

impl Language {
    /// Get the language code for espeak-rs
    pub fn code(&self) -> &'static str {
        match self {
            Language::English => "en",
        }
    }

    /// Get the language name
    pub fn name(&self) -> &'static str {
        match self {
            Language::English => "English",
        }
    }
}

/// Advanced features configuration
#[derive(Debug, Clone, PartialEq)]
pub struct AdvancedFeatures {
    /// Intelligent break words at conjunctions
    break_words: bool,
    /// Numbered list detection (1., 2), a), etc.)
    numbered_lists: bool,
    /// Priority chunking at punctuation boundaries
    punctuation_priority: bool,
    /// Center-break word optimization
    center_break: bool,
    /// Custom break words override (optional)
    break_words_list: Option<Vec<String>>,
}

impl AdvancedFeatures {
    /// No advanced features (preserves current behavior)
    pub fn none() -> Self {
        Self {
            break_words: false,
            numbered_lists: false,
            punctuation_priority: false,
            center_break: false,
            break_words_list: None,
        }
    }

    /// Basic advanced features (break words + numbered lists)
    pub fn basic() -> Self {
        Self {
            break_words: true,
            numbered_lists: true,
            punctuation_priority: false,
            center_break: false,
            break_words_list: None,
        }
    }

    /// Full advanced features enabled
    pub fn full() -> Self {
        Self {
            break_words: true,
            numbered_lists: true,
            punctuation_priority: true,
            center_break: true,
            break_words_list: None,
        }
    }

    /// Custom advanced features configuration
    pub fn new(
        break_words: bool,
        numbered_lists: bool,
        punctuation_priority: bool,
        center_break: bool,
    ) -> Self {
        Self {
            break_words,
            numbered_lists,
            punctuation_priority,
            center_break,
            break_words_list: None,
        }
    }

    /// Set custom break words list
    pub fn with_break_words_list(mut self, words: Vec<String>) -> Self {
        self.break_words_list = Some(words);
        self
    }

    /// Check if break words feature is enabled
    pub fn break_words(&self) -> bool {
        self.break_words
    }

    /// Check if numbered lists feature is enabled
    pub fn numbered_lists(&self) -> bool {
        self.numbered_lists
    }

    /// Check if punctuation priority is enabled
    pub fn punctuation_priority(&self) -> bool {
        self.punctuation_priority
    }

    /// Check if center break is enabled
    pub fn center_break(&self) -> bool {
        self.center_break
    }

    /// Get custom break words list
    pub fn break_words_list(&self) -> Option<&[String]> {
        self.break_words_list.as_deref()
    }

    /// Validate feature combinations
    pub fn validate(&self) -> ChunkingResult<()> {
        // Validate feature conflicts
        if self.break_words && self.break_words_list.as_ref().map_or(false, |list| list.is_empty()) {
            return Err(ChunkingError::config_validation(
                "Break words enabled but break words list is empty"
            ));
        }
        Ok(())
    }
}

/// Main chunking configuration with strategy type parameter
#[derive(Debug, Clone, PartialEq)]
pub struct ChunkingConfig<S> {
    /// Strategy type marker
    pub strategy: PhantomData<S>,
    /// Token configuration
    pub token_config: TokenConfig,
    /// Language configuration
    pub language_config: LanguageConfig,
    /// Advanced features configuration
    pub advanced_features: AdvancedFeatures,
}

impl<S> ChunkingConfig<S> {
    /// Create a new configuration builder
    pub fn builder() -> ChunkingConfigBuilder<S, Uninitialized> {
        ChunkingConfigBuilder::new()
    }

    /// Get token configuration
    pub fn token_config(&self) -> &TokenConfig {
        &self.token_config
    }

    /// Get language configuration
    pub fn language_config(&self) -> &LanguageConfig {
        &self.language_config
    }

    /// Get advanced features configuration
    pub fn advanced_features(&self) -> &AdvancedFeatures {
        &self.advanced_features
    }

    /// Validate the entire configuration
    pub fn validate(&self) -> ChunkingResult<()> {
        self.token_config.validate()?;
        self.language_config.validate()?;
        self.advanced_features.validate()?;
        Ok(())
    }
}

// Builder pattern with type-safe state transitions

/// Builder state markers
pub struct Uninitialized;
pub struct TokenSet;
pub struct LanguageSet;
pub struct Complete;

/// Type-safe configuration builder
pub struct ChunkingConfigBuilder<S, State> {
    strategy: PhantomData<S>,
    state: PhantomData<State>,
    token_config: Option<TokenConfig>,
    language_config: Option<LanguageConfig>,
    advanced_features: Option<AdvancedFeatures>,
}

impl<S> ChunkingConfigBuilder<S, Uninitialized> {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            strategy: PhantomData,
            state: PhantomData,
            token_config: None,
            language_config: None,
            advanced_features: None,
        }
    }

    /// Set token configuration
    pub fn with_token_config(self, config: TokenConfig) -> ChunkingResult<ChunkingConfigBuilder<S, TokenSet>> {
        config.validate()?;
        Ok(ChunkingConfigBuilder {
            strategy: PhantomData,
            state: PhantomData,
            token_config: Some(config),
            language_config: self.language_config,
            advanced_features: self.advanced_features,
        })
    }
}

impl<S> ChunkingConfigBuilder<S, TokenSet> {
    /// Set language configuration
    pub fn with_language_config(self, config: LanguageConfig) -> ChunkingResult<ChunkingConfigBuilder<S, LanguageSet>> {
        config.validate()?;
        Ok(ChunkingConfigBuilder {
            strategy: PhantomData,
            state: PhantomData,
            token_config: self.token_config,
            language_config: Some(config),
            advanced_features: self.advanced_features,
        })
    }
}

impl<S> ChunkingConfigBuilder<S, LanguageSet> {
    /// Set advanced features (optional)
    pub fn with_advanced_features(mut self, features: AdvancedFeatures) -> ChunkingResult<Self> {
        features.validate()?;
        self.advanced_features = Some(features);
        Ok(self)
    }

    /// Build the final configuration
    pub fn build(self) -> ChunkingConfig<S> {
        ChunkingConfig {
            strategy: PhantomData,
            token_config: self.token_config.unwrap_or_default(),
            language_config: self.language_config.unwrap_or_default(),
            advanced_features: self.advanced_features.unwrap_or_else(AdvancedFeatures::none),
        }
    }
}

// Convenience methods for common configurations
impl<S> ChunkingConfigBuilder<S, Uninitialized> {
    /// Quick build with all defaults
    pub fn with_defaults(self) -> ChunkingConfig<S> {
        ChunkingConfig {
            strategy: PhantomData,
            token_config: TokenConfig::default(),
            language_config: LanguageConfig::default(),
            advanced_features: AdvancedFeatures::none(),
        }
    }
}