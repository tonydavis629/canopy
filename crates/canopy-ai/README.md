# canopy-ai Module Documentation

## Overview

The `canopy-ai` module provides AI-powered semantic analysis of code relationships. It uses large language models (LLMs) to infer connections between code elements that aren't immediately obvious from static analysis alone.

## Features

### AI Providers
- **OpenAI** - GPT-4 and GPT-3.5 models via OpenRouter
- **Anthropic** - Claude models via OpenRouter
- **Local** - Heuristic-based analysis without AI

### Semantic Analysis
- **Relationship Inference** - Detects calls, dependencies, uses, etc.
- **Code Summarization** - Generates natural language descriptions
- **Question Answering** - Answers questions about the codebase

### Confidence Scoring
All AI-inferred relationships include confidence scores (0.0-1.0) to indicate reliability.

## Configuration

```toml
[ai]
provider = "openai"  # or "anthropic" or "local"
api_key = "your-api-key"
enabled = true
confidence_threshold = 0.7
```

## Usage

```rust
use canopy_ai::providers::create_provider;
use canopy_ai::bridge::{AIProvider, SemanticAnalysisRequest};

// Create AI provider
let provider = create_provider("openai", Some(api_key))?;

// Analyze relationships
let request = SemanticAnalysisRequest {
    source_node: source_function,
    candidate_nodes: candidate_functions,
    context: analysis_context,
    relationship_types: vec![SemanticRelationship::Calls, SemanticRelationship::DependsOn],
};

let result = provider.analyze_semantic_relationships(request).await?;
```

## Budget Management

The AI module includes budget tracking to prevent excessive API usage:

```rust
use canopy_ai::bridge::AIBudget;

let mut budget = AIBudget::new(100_000); // 100k tokens
if budget.has_budget(estimated_tokens) {
    // Perform AI analysis
    budget.use_tokens(actual_tokens);
}
```

## Caching

AI results are cached to avoid redundant API calls:
- Cache key includes source code hash and request parameters
- Cache is invalidated when source files change
- Cache stored in `.openclaw/canopy/cache/` directory

## Testing

Run unit tests:
```bash
cargo test -p canopy-ai
```

## Best Practices

1. **Use confidence thresholds** - Only accept high-confidence relationships automatically
2. **Cache results** - Enable caching to reduce API costs
3. **Monitor budget** - Set appropriate token limits
4. **Validate results** - AI suggestions should be reviewed by developers