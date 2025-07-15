//! Basic chunking strategy
//!
//! Implements simple word-based chunking that preserves the logic from
//! `split_text_into_speech_chunks()` in kokoros-openai/src/lib.rs

use crate::chunking::config::ChunkingConfig;
use crate::chunking::errors::{ChunkingError, ChunkingResult};
use crate::chunking::strategies::ChunkingStrategy;

/// Basic word-based chunking strategy
/// 
/// This strategy implements the current `split_text_into_speech_chunks()` logic:
/// - Sentence boundary detection using punctuation
/// - Word-based fallback splitting for long content
/// - Simple punctuation handling without phoneme conversion
#[derive(Debug, Clone)]
pub struct BasicStrategy {
    /// Default words per chunk for basic strategy
    default_words_per_chunk: usize,
}

impl BasicStrategy {
    /// Create a new basic strategy
    pub fn new() -> Self {
        Self {
            default_words_per_chunk: 10, // Current default from kokoros-openai
        }
    }

    /// Create with custom words per chunk
    pub fn with_words_per_chunk(words_per_chunk: usize) -> Self {
        Self {
            default_words_per_chunk: words_per_chunk,
        }
    }

    /// Split text using simple word-based logic
    fn split_by_words(&self, text: &str, words_per_chunk: usize) -> Vec<String> {
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut word_count = 0;

        // Split by punctuation first
        for word in text.split_whitespace() {
            if !current_chunk.is_empty() {
                current_chunk.push(' ');
            }

            current_chunk.push_str(word);
            word_count += 1;

            // Check for sentence-ending punctuation (always break)
            let ends_with_sentence = word.ends_with('.')
                || word.ends_with('!')
                || word.ends_with('?')
                || word.ends_with(':')
                || word.ends_with(';');

            // Check for comma (conditional break)
            let ends_with_comma = word.ends_with(',');

            // Break conditions:
            // 1. Sentence-ending punctuation - always break
            // 2. Comma + reached word target
            if ends_with_sentence || (ends_with_comma && word_count >= words_per_chunk) {
                chunks.push(current_chunk.trim().to_string());
                current_chunk.clear();
                word_count = 0;
            }
        }

        // Add remaining chunk
        if !current_chunk.trim().is_empty() {
            chunks.push(current_chunk.trim().to_string());
        }

        // Filter out empty chunks
        chunks.into_iter().filter(|chunk| !chunk.is_empty()).collect()
    }

    /// Estimate tokens using simple character-to-token approximation
    fn estimate_tokens_simple(&self, text: &str) -> usize {
        // Simple approximation: average word length in characters for token estimation
        // This is much less accurate than phoneme-based but sufficient for basic strategy
        text.len() / 4 // Rough approximation: 4 characters per token
    }
}

impl Default for BasicStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl ChunkingStrategy for BasicStrategy {
    fn chunk(&self, text: &str, _config: &ChunkingConfig<Self>) -> ChunkingResult<Vec<String>> {
        if text.trim().is_empty() {
            return Ok(vec![]);
        }

        let chunks = self.split_by_words(text, self.default_words_per_chunk);
        
        if chunks.is_empty() {
            return Err(ChunkingError::text_processing("Failed to create any chunks from input text"));
        }

        Ok(chunks)
    }

    fn validate_tokens(&self, chunks: &[String], limit: usize) -> bool {
        chunks.iter().all(|chunk| {
            self.estimate_tokens_simple(chunk) <= limit
        })
    }

    fn name(&self) -> &'static str {
        "BasicStrategy"
    }

    fn estimate_tokens(&self, text: &str, _config: &ChunkingConfig<Self>) -> ChunkingResult<usize> {
        Ok(self.estimate_tokens_simple(text))
    }
}