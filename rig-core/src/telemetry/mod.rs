//! This module primarily concerns being able to orchestrate telemetry across a given pipeline or workflow.
//! This includes tracing, being able to send traces to an OpenTelemetry collector, setting up your
//! agents with the correct tracing style so you can emit the right traces for platforms like Langfuse,
//! and more.

use crate::completion::GetTokenUsage;
use serde::Serialize;

/// Invocation parameters for recording OpenTelemetry GenAI specification attributes.
/// This struct contains abstraction-friendly fields that can be populated from the
/// provider-agnostic `CompletionRequest` and provider-specific implementations.
#[derive(Debug, Clone, Default)]
pub struct InvocationParameters {
    /// Maximum number of tokens to generate
    pub max_tokens: Option<u64>,
    /// Temperature (randomness) for generation
    pub temperature: Option<f64>,
    /// Top-p (nucleus sampling)
    pub top_p: Option<f64>,
    /// Top-k sampling
    pub top_k: Option<u64>,
    /// Frequency penalty
    pub frequency_penalty: Option<f64>,
    /// Presence penalty
    pub presence_penalty: Option<f64>,
    /// Stop sequences (serialized as JSON array)
    pub stop_sequences: Option<Vec<String>>,
    /// Random seed
    pub seed: Option<i64>,
    /// Number of choices to generate
    pub choice_count: Option<u32>,
    /// Encoding formats (for embeddings)
    pub encoding_formats: Option<Vec<String>>,
    /// Additional provider-specific parameters (JSON-serializable)
    /// This is an escape hatch for provider-specific settings that don't fit
    /// into the standard attributes.
    pub additional_params: Option<serde_json::Value>,
}

impl InvocationParameters {
    /// Create a new empty InvocationParameters
    pub fn new() -> Self {
        Self::default()
    }

    /// Set max_tokens
    pub fn with_max_tokens(mut self, max_tokens: u64) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Set temperature
    pub fn with_temperature(mut self, temperature: f64) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Set top_p
    pub fn with_top_p(mut self, top_p: f64) -> Self {
        self.top_p = Some(top_p);
        self
    }

    /// Set top_k
    pub fn with_top_k(mut self, top_k: u64) -> Self {
        self.top_k = Some(top_k);
        self
    }

    /// Set frequency_penalty
    pub fn with_frequency_penalty(mut self, frequency_penalty: f64) -> Self {
        self.frequency_penalty = Some(frequency_penalty);
        self
    }

    /// Set presence_penalty
    pub fn with_presence_penalty(mut self, presence_penalty: f64) -> Self {
        self.presence_penalty = Some(presence_penalty);
        self
    }

    /// Set stop_sequences
    pub fn with_stop_sequences(mut self, stop_sequences: Vec<String>) -> Self {
        self.stop_sequences = Some(stop_sequences);
        self
    }

    /// Set seed
    pub fn with_seed(mut self, seed: i64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Set choice_count
    pub fn with_choice_count(mut self, choice_count: u32) -> Self {
        self.choice_count = Some(choice_count);
        self
    }

    /// Set encoding_formats
    pub fn with_encoding_formats(mut self, encoding_formats: Vec<String>) -> Self {
        self.encoding_formats = Some(encoding_formats);
        self
    }

    /// Set additional parameters
    pub fn with_additional_params(mut self, additional_params: serde_json::Value) -> Self {
        self.additional_params = Some(additional_params);
        self
    }
}

pub trait ProviderRequestExt {
    type InputMessage: Serialize;

    fn get_input_messages(&self) -> Vec<Self::InputMessage>;
    fn get_system_prompt(&self) -> Option<String>;
    fn get_model_name(&self) -> String;
    fn get_prompt(&self) -> Option<String>;

    /// Get invocation parameters for telemetry
    /// Providers should implement this to extract parameters from their request types
    fn get_invocation_parameters(&self) -> InvocationParameters {
        InvocationParameters::new()
    }
}

pub trait ProviderResponseExt {
    type OutputMessage: Serialize;
    type Usage: Serialize;

    fn get_response_id(&self) -> Option<String>;

    fn get_response_model_name(&self) -> Option<String>;

    fn get_output_messages(&self) -> Vec<Self::OutputMessage>;

    fn get_text_response(&self) -> Option<String>;

    fn get_usage(&self) -> Option<Self::Usage>;
}

/// A trait designed specifically to be used with Spans for the purpose of recording telemetry.
/// Nearly all methods
pub trait SpanCombinator {
    fn record_token_usage<U>(&self, usage: &U)
    where
        U: GetTokenUsage;

    fn record_response_metadata<R>(&self, response: &R)
    where
        R: ProviderResponseExt;

    fn record_model_input<T>(&self, messages: &T)
    where
        T: Serialize;

    fn record_model_output<T>(&self, messages: &T)
    where
        T: Serialize;

    fn record_invocation_parameters(&self, params: &InvocationParameters);
}

impl SpanCombinator for tracing::Span {
    fn record_token_usage<U>(&self, usage: &U)
    where
        U: GetTokenUsage,
    {
        if let Some(usage) = usage.token_usage() {
            self.record("gen_ai.usage.input_tokens", usage.input_tokens);
            self.record("gen_ai.usage.output_tokens", usage.output_tokens);
        }
    }

    fn record_response_metadata<R>(&self, response: &R)
    where
        R: ProviderResponseExt,
    {
        if let Some(id) = response.get_response_id() {
            self.record("gen_ai.response.id", id);
        }

        if let Some(model_name) = response.get_response_model_name() {
            self.record("gen_ai.response.model_name", model_name);
        }
    }

    fn record_model_input<T>(&self, input: &T)
    where
        T: Serialize,
    {
        let input_as_json_string =
            serde_json::to_string(input).expect("Serializing a Rust type to JSON should not break");

        self.record("gen_ai.input.messages", input_as_json_string);
    }

    fn record_model_output<T>(&self, input: &T)
    where
        T: Serialize,
    {
        let input_as_json_string =
            serde_json::to_string(input).expect("Serializing a Rust type to JSON should not break");

        self.record("gen_ai.input.messages", input_as_json_string);
    }

    fn record_invocation_parameters(&self, params: &InvocationParameters) {
        // Record standard invocation parameters following OpenTelemetry GenAI semantic conventions
        if let Some(max_tokens) = params.max_tokens {
            self.record("gen_ai.request.max_tokens", max_tokens);
        }

        if let Some(temperature) = params.temperature {
            // Only record finite values to avoid serialization issues
            if temperature.is_finite() {
                self.record("gen_ai.request.temperature", temperature);
            }
        }

        if let Some(top_p) = params.top_p {
            if top_p.is_finite() {
                self.record("gen_ai.request.top_p", top_p);
            }
        }

        if let Some(top_k) = params.top_k {
            self.record("gen_ai.request.top_k", top_k);
        }

        if let Some(frequency_penalty) = params.frequency_penalty {
            if frequency_penalty.is_finite() {
                self.record("gen_ai.request.frequency_penalty", frequency_penalty);
            }
        }

        if let Some(presence_penalty) = params.presence_penalty {
            if presence_penalty.is_finite() {
                self.record("gen_ai.request.presence_penalty", presence_penalty);
            }
        }

        if let Some(stop_sequences) = &params.stop_sequences {
            // Serialize stop sequences as JSON array
            if let Ok(stop_sequences_json) = serde_json::to_string(stop_sequences) {
                self.record("gen_ai.request.stop_sequences", stop_sequences_json);
            }
        }

        if let Some(seed) = params.seed {
            self.record("gen_ai.request.seed", seed);
        }

        if let Some(choice_count) = params.choice_count {
            self.record("gen_ai.request.choice.count", choice_count);
        }

        if let Some(encoding_formats) = &params.encoding_formats {
            // Serialize encoding formats as JSON array
            if let Ok(encoding_formats_json) = serde_json::to_string(encoding_formats) {
                self.record("gen_ai.request.encoding_formats", encoding_formats_json);
            }
        }

        // Record additional provider-specific parameters if present
        if let Some(additional_params) = &params.additional_params {
            if let Ok(params_json) = serde_json::to_string(additional_params) {
                self.record("gen_ai.request.parameters", params_json);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invocation_parameters_builder() {
        let params = InvocationParameters::new()
            .with_max_tokens(100)
            .with_temperature(0.7)
            .with_top_p(0.9)
            .with_top_k(50)
            .with_frequency_penalty(0.5)
            .with_presence_penalty(0.5)
            .with_seed(42)
            .with_choice_count(3);

        assert_eq!(params.max_tokens, Some(100));
        assert_eq!(params.temperature, Some(0.7));
        assert_eq!(params.top_p, Some(0.9));
        assert_eq!(params.top_k, Some(50));
        assert_eq!(params.frequency_penalty, Some(0.5));
        assert_eq!(params.presence_penalty, Some(0.5));
        assert_eq!(params.seed, Some(42));
        assert_eq!(params.choice_count, Some(3));
    }

    #[test]
    fn test_invocation_parameters_with_stop_sequences() {
        let params = InvocationParameters::new()
            .with_stop_sequences(vec!["STOP".to_string(), "END".to_string()]);

        assert_eq!(
            params.stop_sequences,
            Some(vec!["STOP".to_string(), "END".to_string()])
        );
    }

    #[test]
    fn test_invocation_parameters_with_encoding_formats() {
        let params = InvocationParameters::new()
            .with_encoding_formats(vec!["float".to_string(), "base64".to_string()]);

        assert_eq!(
            params.encoding_formats,
            Some(vec!["float".to_string(), "base64".to_string()])
        );
    }

    #[test]
    fn test_invocation_parameters_with_additional_params() {
        let additional = serde_json::json!({
            "custom_param": "value",
            "another_param": 123
        });

        let params = InvocationParameters::new().with_additional_params(additional.clone());

        assert_eq!(params.additional_params, Some(additional));
    }

    #[test]
    fn test_invocation_parameters_default() {
        let params = InvocationParameters::default();

        assert!(params.max_tokens.is_none());
        assert!(params.temperature.is_none());
        assert!(params.top_p.is_none());
        assert!(params.top_k.is_none());
        assert!(params.frequency_penalty.is_none());
        assert!(params.presence_penalty.is_none());
        assert!(params.stop_sequences.is_none());
        assert!(params.seed.is_none());
        assert!(params.choice_count.is_none());
        assert!(params.encoding_formats.is_none());
        assert!(params.additional_params.is_none());
    }

    // Test span recording of invocation parameters
    // Note: This is a basic test - in practice you'd want to use a real tracing subscriber
    // to verify the attributes are properly recorded
    #[test]
    fn test_record_invocation_parameters_with_finite_values() {
        let params = InvocationParameters::new()
            .with_max_tokens(100)
            .with_temperature(0.7)
            .with_top_p(0.9)
            .with_stop_sequences(vec!["STOP".to_string()]);

        let span = tracing::info_span!("test_span");
        span.record_invocation_parameters(&params);
        // If we get here without panicking, the basic recording worked
    }

    #[test]
    fn test_record_invocation_parameters_with_non_finite_values() {
        let mut params = InvocationParameters::new();
        params.temperature = Some(f64::INFINITY);
        params.top_p = Some(f64::NAN);

        let span = tracing::info_span!("test_span");
        // Should not panic even with non-finite values (they should be filtered out)
        span.record_invocation_parameters(&params);
    }
}
