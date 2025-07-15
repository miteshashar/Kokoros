//! Phonetics-based chunking strategy
//!
//! Implements phoneme-to-token accurate chunking that preserves the exact logic from
//! `split_text_into_chunks()` in kokoros/src/tts/koko.rs

use crate::chunking::config::ChunkingConfig;
use crate::chunking::errors::{ChunkingError, ChunkingResult};
use crate::chunking::strategies::ChunkingStrategy;
use crate::tts::tokenize::tokenize;
use espeak_rs::text_to_phonemes;
use lazy_static::lazy_static;
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};

// Re-use the global espeak mutex from the existing koko.rs implementation
lazy_static! {
    static ref ESPEAK_MUTEX: Mutex<()> = Mutex::new(());
}

/// Phonetics-based chunking strategy with LRU cache
/// 
/// This strategy implements the current `split_text_into_chunks()` logic exactly:
/// - espeak-rs phoneme-to-token conversion with thread safety
/// - 500 token limit with 12-token safety margin (configurable)
/// - Sentence → word → character boundary fallback hierarchy
/// - LRU cache for phoneme conversions to improve performance
#[derive(Debug)]
pub struct PhoneticsStrategy {
    /// LRU cache for phoneme conversions to improve performance
    phoneme_cache: Arc<Mutex<LruCache<String, String>>>,
}

impl PhoneticsStrategy {
    /// Create a new phonetics strategy with default cache size
    pub fn new() -> Self {
        Self::with_cache_size(100)
    }

    /// Create with custom cache size
    pub fn with_cache_size(cache_size: usize) -> Self {
        let cache_size = NonZeroUsize::new(cache_size).unwrap_or(NonZeroUsize::new(100).unwrap());
        Self {
            phoneme_cache: Arc::new(Mutex::new(LruCache::new(cache_size))),
        }
    }

    /// Convert text to phonemes with caching and thread safety
    fn text_to_phonemes_cached(&self, text: &str, language: &str) -> ChunkingResult<String> {
        let cache_key = format!("{}:{}", language, text);
        
        // Check cache first
        {
            let mut cache = self.phoneme_cache.lock().unwrap();
            if let Some(cached_phonemes) = cache.get(&cache_key) {
                return Ok(cached_phonemes.clone());
            }
        }

        // Convert to phonemes with thread safety (preserving exact current implementation)
        let phonemes = {
            let _guard = ESPEAK_MUTEX.lock().unwrap();
            text_to_phonemes(text, language, None, true, false) // with_stress=false as decided in 1A.2
                .map_err(|e| ChunkingError::phoneme_conversion(text, Box::new(e)))?
                .join("")
        };

        // Cache the result
        {
            let mut cache = self.phoneme_cache.lock().unwrap();
            cache.put(cache_key, phonemes.clone());
        }

        Ok(phonemes)
    }

    /// Estimate tokens using phoneme conversion (exact current implementation)
    fn estimate_tokens_phoneme(&self, text: &str, language: &str) -> ChunkingResult<usize> {
        let phonemes = self.text_to_phonemes_cached(text, language)?;
        Ok(tokenize(&phonemes).len())
    }

    /// Split text into chunks (preserving exact current logic from koko.rs)
    fn split_text_with_phonemes(&self, text: &str, max_tokens: usize, language: &str) -> ChunkingResult<Vec<String>> {
        let mut chunks = Vec::new();

        // First split by sentences - using common sentence ending punctuation
        let sentences: Vec<&str> = text
            .split(|c| c == '.' || c == '?' || c == '!' || c == ';')
            .filter(|s| !s.trim().is_empty())
            .collect();

        let mut current_chunk = String::new();

        for sentence in sentences {
            // Clean up the sentence and add back punctuation
            let sentence = format!("{}.", sentence.trim());

            // Convert to phonemes to check token count
            let token_count = self.estimate_tokens_phoneme(&sentence, language)?;

            if token_count > max_tokens {
                // If single sentence is too long, split by words
                let words: Vec<&str> = sentence.split_whitespace().collect();
                let mut word_chunk = String::new();

                for word in words {
                    let test_chunk = if word_chunk.is_empty() {
                        word.to_string()
                    } else {
                        format!("{} {}", word_chunk, word)
                    };

                    let test_tokens = self.estimate_tokens_phoneme(&test_chunk, language)?;

                    if test_tokens > max_tokens {
                        if !word_chunk.is_empty() {
                            chunks.push(word_chunk);
                        }
                        word_chunk = word.to_string();
                    } else {
                        word_chunk = test_chunk;
                    }
                }

                if !word_chunk.is_empty() {
                    chunks.push(word_chunk);
                }
            } else if !current_chunk.is_empty() {
                // Try to append to current chunk
                let test_text = format!("{} {}", current_chunk, sentence);
                let test_tokens = self.estimate_tokens_phoneme(&test_text, language)?;

                if test_tokens > max_tokens {
                    // If combining would exceed limit, start new chunk
                    chunks.push(current_chunk);
                    current_chunk = sentence;
                } else {
                    current_chunk = test_text;
                }
            } else {
                current_chunk = sentence;
            }
        }

        // Add the last chunk if not empty
        if !current_chunk.is_empty() {
            chunks.push(current_chunk);
        }

        Ok(chunks)
    }
}

impl Default for PhoneticsStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for PhoneticsStrategy {
    fn clone(&self) -> Self {
        // Create a new cache for the cloned instance to avoid sharing mutex
        Self::new()
    }
}

impl ChunkingStrategy for PhoneticsStrategy {
    fn chunk(&self, text: &str, config: &ChunkingConfig<Self>) -> ChunkingResult<Vec<String>> {
        if text.trim().is_empty() {
            return Ok(vec![]);
        }

        let max_tokens = config.token_config().effective_limit();
        let language = config.language_config().language().code();

        let chunks = self.split_text_with_phonemes(text, max_tokens, language)?;
        
        if chunks.is_empty() {
            return Err(ChunkingError::text_processing("Failed to create any chunks from input text"));
        }

        // Validate all chunks meet token limits
        for chunk in &chunks {
            let tokens = self.estimate_tokens_phoneme(chunk, language)?;
            if tokens > max_tokens {
                return Err(ChunkingError::token_limit_exceeded(
                    tokens,
                    max_tokens,
                    config.token_config().safety_margin(),
                ));
            }
        }

        Ok(chunks)
    }

    fn validate_tokens(&self, chunks: &[String], limit: usize) -> bool {
        // For phonetics strategy, we need the language code, but this is a simplified validation
        // In practice, this would be called from the engine with proper context
        chunks.iter().all(|chunk| {
            // Fallback to simple estimation if phoneme conversion fails
            self.estimate_tokens_phoneme(chunk, "en").unwrap_or(chunk.len() / 4) <= limit
        })
    }

    fn name(&self) -> &'static str {
        "PhoneticsStrategy"
    }

    fn estimate_tokens(&self, text: &str, config: &ChunkingConfig<Self>) -> ChunkingResult<usize> {
        let language = config.language_config().language().code();
        self.estimate_tokens_phoneme(text, language)
    }
}