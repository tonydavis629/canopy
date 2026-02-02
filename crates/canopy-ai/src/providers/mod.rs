//! AI provider implementations

pub mod openai;
pub mod anthropic;
pub mod local;

use super::bridge::AIProvider;
use anyhow::Result;

/// Factory function to create AI providers
pub fn create_provider(provider_name: &str, api_key: Option<String>) -> Result<Box<dyn AIProvider>> {
    match provider_name {
        "openai" => Ok(Box::new(openai::OpenAIProvider::new(api_key))),
        "anthropic" => Ok(Box::new(anthropic::AnthropicProvider::new(api_key))),
        "local" => Ok(Box::new(local::LocalProvider::new())),
        _ => anyhow::bail!("Unknown AI provider: {}", provider_name),
    }
}