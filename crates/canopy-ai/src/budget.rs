//! Budget tracking for AI API usage

use super::bridge::Confidence;

/// Budget configuration and tracking
#[derive(Debug, Clone)]
pub struct Budget {
    /// Total tokens available for the session
    pub total_tokens: u32,
    /// Tokens used so far
    pub tokens_used: u32,
    /// Maximum tokens per analysis request
    pub max_tokens_per_request: u32,
    /// Confidence threshold for auto-accepting AI results
    pub auto_accept_threshold: Confidence,
    /// Whether to use caching to reduce API calls
    pub enable_caching: bool,
}

impl Budget {
    /// Create a new budget with the specified total tokens
    pub fn new(total_tokens: u32) -> Self {
        Self {
            total_tokens,
            tokens_used: 0,
            max_tokens_per_request: 4000,
            auto_accept_threshold: 0.8,
            enable_caching: true,
        }
    }
    
    /// Check if there's enough budget for an estimated token cost
    pub fn has_budget(&self, estimated_tokens: u32) -> bool {
        self.tokens_used + estimated_tokens <= self.total_tokens
    }
    
    /// Record token usage
    pub fn use_tokens(&mut self, tokens: u32) {
        self.tokens_used += tokens;
    }
    
    /// Get remaining tokens
    pub fn remaining(&self) -> u32 {
        self.total_tokens.saturating_sub(self.tokens_used)
    }
    
    /// Get usage percentage
    pub fn usage_percentage(&self) -> f32 {
        if self.total_tokens == 0 {
            return 0.0;
        }
        (self.tokens_used as f32 / self.total_tokens as f32) * 100.0
    }
    
    /// Check if budget is exhausted
    pub fn is_exhausted(&self) -> bool {
        self.tokens_used >= self.total_tokens
    }
    
    /// Check if a confidence score meets the auto-accept threshold
    pub fn should_auto_accept(&self, confidence: Confidence) -> bool {
        confidence >= self.auto_accept_threshold
    }
    
    /// Estimate tokens for a request based on prompt length
    /// Rough estimate: ~4 characters per token
    pub fn estimate_tokens(prompt_length: usize) -> u32 {
        ((prompt_length / 4) as u32).saturating_add(500) // Base overhead
    }
}

impl Default for Budget {
    fn default() -> Self {
        Self::new(100_000) // Default 100k tokens
    }
}

/// Budget warning levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetWarning {
    /// Budget is healthy (< 50%)
    Healthy,
    /// Budget is getting low (50-75%)
    Warning,
    /// Budget is critically low (75-90%)
    Critical,
    /// Budget is nearly exhausted (> 90%)
    Exhausted,
}

impl Budget {
    /// Get the current warning level
    pub fn warning_level(&self) -> BudgetWarning {
        let percentage = self.usage_percentage();
        match percentage {
            p if p < 50.0 => BudgetWarning::Healthy,
            p if p < 75.0 => BudgetWarning::Warning,
            p if p < 90.0 => BudgetWarning::Critical,
            _ => BudgetWarning::Exhausted,
        }
    }
}