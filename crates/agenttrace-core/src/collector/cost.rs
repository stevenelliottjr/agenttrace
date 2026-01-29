//! Cost calculator for LLM calls
//!
//! Calculates the cost of LLM API calls based on token usage and model pricing.

use std::collections::HashMap;

use crate::models::Span;

/// Pricing information for a model (per million tokens)
#[derive(Debug, Clone)]
pub struct ModelPricing {
    /// Cost per million input tokens
    pub input_per_million: f64,
    /// Cost per million output tokens
    pub output_per_million: f64,
    /// Cost per million cached input tokens (if applicable)
    pub cached_input_per_million: Option<f64>,
}

/// Cost calculator with model pricing database
pub struct CostCalculator {
    pricing: HashMap<String, ModelPricing>,
}

impl Default for CostCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl CostCalculator {
    /// Create a new cost calculator with default pricing
    pub fn new() -> Self {
        let mut pricing = HashMap::new();

        // Anthropic Claude models (as of Jan 2025)
        pricing.insert(
            "claude-3-opus".to_string(),
            ModelPricing {
                input_per_million: 15.0,
                output_per_million: 75.0,
                cached_input_per_million: Some(1.5),
            },
        );
        pricing.insert(
            "claude-3-5-sonnet".to_string(),
            ModelPricing {
                input_per_million: 3.0,
                output_per_million: 15.0,
                cached_input_per_million: Some(0.3),
            },
        );
        pricing.insert(
            "claude-3-5-haiku".to_string(),
            ModelPricing {
                input_per_million: 0.80,
                output_per_million: 4.0,
                cached_input_per_million: Some(0.08),
            },
        );
        pricing.insert(
            "claude-sonnet-4".to_string(),
            ModelPricing {
                input_per_million: 3.0,
                output_per_million: 15.0,
                cached_input_per_million: Some(0.3),
            },
        );
        pricing.insert(
            "claude-opus-4".to_string(),
            ModelPricing {
                input_per_million: 15.0,
                output_per_million: 75.0,
                cached_input_per_million: Some(1.5),
            },
        );

        // OpenAI models (as of Jan 2025)
        pricing.insert(
            "gpt-4".to_string(),
            ModelPricing {
                input_per_million: 30.0,
                output_per_million: 60.0,
                cached_input_per_million: None,
            },
        );
        pricing.insert(
            "gpt-4-turbo".to_string(),
            ModelPricing {
                input_per_million: 10.0,
                output_per_million: 30.0,
                cached_input_per_million: None,
            },
        );
        pricing.insert(
            "gpt-4o".to_string(),
            ModelPricing {
                input_per_million: 2.50,
                output_per_million: 10.0,
                cached_input_per_million: Some(1.25),
            },
        );
        pricing.insert(
            "gpt-4o-mini".to_string(),
            ModelPricing {
                input_per_million: 0.15,
                output_per_million: 0.60,
                cached_input_per_million: Some(0.075),
            },
        );
        pricing.insert(
            "o1".to_string(),
            ModelPricing {
                input_per_million: 15.0,
                output_per_million: 60.0,
                cached_input_per_million: Some(7.5),
            },
        );
        pricing.insert(
            "o1-mini".to_string(),
            ModelPricing {
                input_per_million: 3.0,
                output_per_million: 12.0,
                cached_input_per_million: Some(1.5),
            },
        );
        pricing.insert(
            "o1-pro".to_string(),
            ModelPricing {
                input_per_million: 150.0,
                output_per_million: 600.0,
                cached_input_per_million: None,
            },
        );
        pricing.insert(
            "gpt-3.5-turbo".to_string(),
            ModelPricing {
                input_per_million: 0.50,
                output_per_million: 1.50,
                cached_input_per_million: None,
            },
        );

        // Google models
        pricing.insert(
            "gemini-1.5-pro".to_string(),
            ModelPricing {
                input_per_million: 1.25,
                output_per_million: 5.0,
                cached_input_per_million: Some(0.3125),
            },
        );
        pricing.insert(
            "gemini-1.5-flash".to_string(),
            ModelPricing {
                input_per_million: 0.075,
                output_per_million: 0.30,
                cached_input_per_million: Some(0.01875),
            },
        );
        pricing.insert(
            "gemini-2.0-flash".to_string(),
            ModelPricing {
                input_per_million: 0.10,
                output_per_million: 0.40,
                cached_input_per_million: Some(0.025),
            },
        );

        // Mistral models
        pricing.insert(
            "mistral-large".to_string(),
            ModelPricing {
                input_per_million: 2.0,
                output_per_million: 6.0,
                cached_input_per_million: None,
            },
        );
        pricing.insert(
            "mistral-small".to_string(),
            ModelPricing {
                input_per_million: 0.2,
                output_per_million: 0.6,
                cached_input_per_million: None,
            },
        );

        Self { pricing }
    }

    /// Calculate cost for a span
    pub fn calculate(&self, span: &mut Span) {
        // Only calculate for LLM calls with token usage
        if !span.is_llm_call() {
            return;
        }

        let model_name = match &span.model_name {
            Some(name) => name,
            None => return,
        };

        // Find matching pricing
        let pricing = self.find_pricing(model_name);
        let pricing = match pricing {
            Some(p) => p,
            None => {
                // Unknown model, can't calculate cost
                tracing::debug!("Unknown model for cost calculation: {}", model_name);
                return;
            }
        };

        let tokens_in = span.tokens_in.unwrap_or(0) as f64;
        let tokens_out = span.tokens_out.unwrap_or(0) as f64;
        let tokens_reasoning = span.tokens_reasoning.unwrap_or(0) as f64;

        // Calculate input cost
        let input_cost = (tokens_in / 1_000_000.0) * pricing.input_per_million;

        // Calculate output cost (reasoning tokens count as output)
        let output_cost = ((tokens_out + tokens_reasoning) / 1_000_000.0) * pricing.output_per_million;

        span.cost_usd = Some(input_cost + output_cost);
    }

    /// Find pricing for a model by matching model name prefix
    fn find_pricing(&self, model_name: &str) -> Option<&ModelPricing> {
        // Try exact match first
        if let Some(pricing) = self.pricing.get(model_name) {
            return Some(pricing);
        }

        // Try prefix match (e.g., "claude-3-5-sonnet-20241022" matches "claude-3-5-sonnet")
        for (key, pricing) in &self.pricing {
            if model_name.starts_with(key) {
                return Some(pricing);
            }
        }

        // Try contains match for versioned models
        for (key, pricing) in &self.pricing {
            if model_name.contains(key) {
                return Some(pricing);
            }
        }

        None
    }

    /// Add or update pricing for a model
    pub fn set_pricing(&mut self, model: String, pricing: ModelPricing) {
        self.pricing.insert(model, pricing);
    }

    /// Get pricing for a model
    pub fn get_pricing(&self, model: &str) -> Option<&ModelPricing> {
        self.find_pricing(model)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn create_test_span(model: &str, tokens_in: i32, tokens_out: i32) -> Span {
        Span {
            id: Uuid::new_v4(),
            span_id: "test-span".to_string(),
            trace_id: "test-trace".to_string(),
            parent_span_id: None,
            operation_name: "llm_call".to_string(),
            service_name: "test".to_string(),
            span_kind: crate::models::SpanKind::Internal,
            started_at: Utc::now(),
            ended_at: Some(Utc::now()),
            duration_ms: Some(100.0),
            status: crate::models::SpanStatus::Ok,
            status_message: None,
            model_name: Some(model.to_string()),
            model_provider: Some("anthropic".to_string()),
            tokens_in: Some(tokens_in),
            tokens_out: Some(tokens_out),
            tokens_reasoning: None,
            cost_usd: None,
            tool_name: None,
            tool_input: None,
            tool_output: None,
            tool_duration_ms: None,
            prompt_preview: None,
            completion_preview: None,
            attributes: serde_json::json!({}),
            events: vec![],
            links: vec![],
        }
    }

    #[test]
    fn test_cost_calculation_claude_sonnet() {
        let calculator = CostCalculator::new();
        let mut span = create_test_span("claude-sonnet-4-20250514", 1000, 500);

        calculator.calculate(&mut span);

        // 1000 input tokens at $3/M = $0.003
        // 500 output tokens at $15/M = $0.0075
        // Total = $0.0105
        assert!(span.cost_usd.is_some());
        let cost = span.cost_usd.unwrap();
        assert!((cost - 0.0105).abs() < 0.0001);
    }

    #[test]
    fn test_cost_calculation_gpt4o() {
        let calculator = CostCalculator::new();
        let mut span = create_test_span("gpt-4o", 1000000, 500000);

        calculator.calculate(&mut span);

        // 1M input tokens at $2.50/M = $2.50
        // 500K output tokens at $10/M = $5.00
        // Total = $7.50
        assert!(span.cost_usd.is_some());
        let cost = span.cost_usd.unwrap();
        assert!((cost - 7.50).abs() < 0.01);
    }

    #[test]
    fn test_unknown_model() {
        let calculator = CostCalculator::new();
        let mut span = create_test_span("unknown-model-xyz", 1000, 500);

        calculator.calculate(&mut span);

        // Should not set cost for unknown model
        assert!(span.cost_usd.is_none());
    }
}
