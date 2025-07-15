use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use spin_sdk::variables;

use crate::providers::{AuthKitProvider, OidcProvider, OidcProviderConfig, ProviderRegistry};

/// Gateway configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GatewayConfig {
    pub mcp_gateway_url: String,
    pub trace_id_header: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    pub providers: Vec<ProviderConfig>,
}

fn default_enabled() -> bool {
    true
}

/// Provider configuration enum
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProviderConfig {
    #[serde(rename = "authkit")]
    AuthKit {
        issuer: String,
        #[serde(default)]
        jwks_uri: Option<String>,
        #[serde(default)]
        audience: Option<String>,
    },
    Oidc {
        name: String,
        issuer: String,
        jwks_uri: String,
        #[serde(default)]
        audience: Option<String>,
        authorization_endpoint: String,
        token_endpoint: String,
        #[serde(default)]
        userinfo_endpoint: Option<String>,
        #[serde(default)]
        allowed_domains: Vec<String>,
    },
}

impl GatewayConfig {
    /// Load configuration from Spin variables
    pub fn from_spin_vars() -> Result<Self> {
        let config_json = variables::get("auth_config")
            .context("Missing required 'auth_config' variable")?;

        if config_json.is_empty() {
            // Return default config for tests
            return Ok(Self {
                mcp_gateway_url: "http://ftl-mcp-gateway.spin.internal/mcp-internal".to_string(),
                trace_id_header: "X-Trace-Id".to_string(),
                enabled: true,
                providers: vec![],
            });
        }

        serde_json::from_str(&config_json).context("Failed to parse auth_config JSON")
    }

    /// Build provider registry from configuration
    pub fn build_registry(&self) -> ProviderRegistry {
        let mut registry = ProviderRegistry::new();

        for provider_config in &self.providers {
            match provider_config {
                ProviderConfig::AuthKit {
                    issuer,
                    jwks_uri,
                    audience,
                } => {
                    let provider =
                        AuthKitProvider::new(issuer.clone(), jwks_uri.clone(), audience.clone());
                    registry.add_provider(Box::new(provider));
                }
                ProviderConfig::Oidc {
                    name,
                    issuer,
                    jwks_uri,
                    audience,
                    authorization_endpoint,
                    token_endpoint,
                    userinfo_endpoint,
                    allowed_domains,
                } => {
                    let config = OidcProviderConfig {
                        name: name.clone(),
                        issuer: issuer.clone(),
                        jwks_uri: jwks_uri.clone(),
                        audience: audience.clone(),
                        authorization_endpoint: authorization_endpoint.clone(),
                        token_endpoint: token_endpoint.clone(),
                        userinfo_endpoint: userinfo_endpoint.clone(),
                        allowed_domains: allowed_domains.clone(),
                    };
                    let provider = OidcProvider::new(config);
                    registry.add_provider(Box::new(provider));
                }
            }
        }

        registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_parsing() {
        let json = r#"{
            "mcp_gateway_url": "http://gateway.internal",
            "trace_id_header": "X-Request-ID",
            "enabled": true,
            "providers": [
                {
                    "type": "authkit",
                    "issuer": "https://example.authkit.app",
                    "audience": "my-api"
                },
                {
                    "type": "oidc",
                    "name": "auth0",
                    "issuer": "https://example.auth0.com",
                    "jwks_uri": "https://example.auth0.com/.well-known/jwks.json",
                    "authorization_endpoint": "https://example.auth0.com/authorize",
                    "token_endpoint": "https://example.auth0.com/oauth/token",
                    "allowed_domains": ["*.auth0.com"]
                }
            ]
        }"#;

        let config: GatewayConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.providers.len(), 2);
        assert_eq!(config.trace_id_header, "X-Request-ID");
        assert!(config.enabled);
    }

    #[test]
    fn test_config_parsing_with_auth_disabled() {
        let json = r#"{
            "mcp_gateway_url": "http://gateway.internal",
            "trace_id_header": "X-Request-ID",
            "enabled": false,
            "providers": []
        }"#;

        let config: GatewayConfig = serde_json::from_str(json).unwrap();
        assert!(!config.enabled);
        assert_eq!(config.providers.len(), 0);
    }

    #[test]
    fn test_config_parsing_defaults_enabled() {
        // Test that enabled defaults to true when not specified
        let json = r#"{
            "mcp_gateway_url": "http://gateway.internal",
            "trace_id_header": "X-Request-ID",
            "providers": []
        }"#;

        let config: GatewayConfig = serde_json::from_str(json).unwrap();
        assert!(config.enabled);
    }
}
