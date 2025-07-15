use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use spin_sdk::variables;

use crate::providers::{AuthKitProvider, OidcProvider, OidcProviderConfig, ProviderRegistry};

/// Gateway configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GatewayConfig {
    pub mcp_gateway_url: String,
    pub trace_id_header: String,
    pub enabled: bool,
    pub provider: Option<ProviderConfig>,
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
        // Read core settings
        let enabled = variables::get("auth_enabled")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);

        let mcp_gateway_url = variables::get("auth_gateway_url")
            .unwrap_or_else(|_| "http://ftl-mcp-gateway.spin.internal/mcp-internal".to_string());

        let trace_id_header =
            variables::get("auth_trace_header").unwrap_or_else(|_| "X-Trace-Id".to_string());

        // Read provider configuration
        let provider_type = variables::get("auth_provider_type").unwrap_or_default();

        let provider = if provider_type.is_empty() {
            None
        } else {
            Some(Self::load_provider_config(&provider_type)?)
        };

        Ok(Self {
            mcp_gateway_url,
            trace_id_header,
            enabled,
            provider,
        })
    }

    /// Ensure URL uses HTTPS protocol. Adds https:// if no protocol specified.
    /// Returns error if http:// is explicitly used.
    fn ensure_https_url(url: String) -> Result<String> {
        if url.starts_with("http://") {
            anyhow::bail!(
                "Auth provider URLs must use HTTPS. HTTP is not allowed for security reasons. \
                If you meant to use HTTPS, either provide just the domain (e.g., \"example.authkit.app\") \
                or the full HTTPS URL (e.g., \"https://example.authkit.app\")."
            )
        } else if url.starts_with("https://") {
            Ok(url)
        } else {
            Ok(format!("https://{url}"))
        }
    }

    /// Load provider configuration from variables
    fn load_provider_config(provider_type: &str) -> Result<ProviderConfig> {
        let issuer = variables::get("auth_provider_issuer")
            .context("auth_provider_issuer is required when auth_provider_type is set")?;
        let issuer = Self::ensure_https_url(issuer)?;

        let audience = variables::get("auth_provider_audience")
            .ok()
            .filter(|s| !s.is_empty());

        match provider_type {
            "authkit" => {
                let jwks_uri = variables::get("auth_provider_jwks_uri")
                    .ok()
                    .filter(|s| !s.is_empty());

                Ok(ProviderConfig::AuthKit {
                    issuer,
                    jwks_uri,
                    audience,
                })
            }
            "oidc" => {
                let name = variables::get("auth_provider_name")
                    .context("auth_provider_name is required for OIDC provider")?;

                let jwks_uri = variables::get("auth_provider_jwks_uri")
                    .context("auth_provider_jwks_uri is required for OIDC provider")?;
                let jwks_uri = Self::ensure_https_url(jwks_uri)?;

                let authorization_endpoint = variables::get("auth_provider_authorize_endpoint")
                    .context("auth_provider_authorize_endpoint is required for OIDC provider")?;
                let authorization_endpoint = Self::ensure_https_url(authorization_endpoint)?;

                let token_endpoint = variables::get("auth_provider_token_endpoint")
                    .context("auth_provider_token_endpoint is required for OIDC provider")?;
                let token_endpoint = Self::ensure_https_url(token_endpoint)?;

                let userinfo_endpoint = variables::get("auth_provider_userinfo_endpoint")
                    .ok()
                    .filter(|s| !s.is_empty())
                    .map(Self::ensure_https_url)
                    .transpose()?;

                let allowed_domains = variables::get("auth_provider_allowed_domains")
                    .unwrap_or_default()
                    .split(',')
                    .filter(|s| !s.trim().is_empty())
                    .map(|s| s.trim().to_string())
                    .collect();

                Ok(ProviderConfig::Oidc {
                    name,
                    issuer,
                    jwks_uri,
                    audience,
                    authorization_endpoint,
                    token_endpoint,
                    userinfo_endpoint,
                    allowed_domains,
                })
            }
            _ => anyhow::bail!(
                "Unknown auth provider type: {}. Expected 'authkit' or 'oidc'",
                provider_type
            ),
        }
    }

    /// Build provider registry from configuration
    pub fn build_registry(&self) -> ProviderRegistry {
        let mut registry = ProviderRegistry::new();

        if let Some(provider_config) = &self.provider {
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
    fn test_authkit_provider_config() {
        let provider = ProviderConfig::AuthKit {
            issuer: "https://example.authkit.app".to_string(),
            jwks_uri: None,
            audience: Some("my-api".to_string()),
        };

        // Test serialization
        let json = serde_json::to_string(&provider).unwrap();
        assert!(json.contains("authkit"));
        assert!(json.contains("https://example.authkit.app"));
    }

    #[test]
    fn test_oidc_provider_config() {
        let provider = ProviderConfig::Oidc {
            name: "auth0".to_string(),
            issuer: "https://example.auth0.com".to_string(),
            jwks_uri: "https://example.auth0.com/.well-known/jwks.json".to_string(),
            audience: Some("my-api".to_string()),
            authorization_endpoint: "https://example.auth0.com/authorize".to_string(),
            token_endpoint: "https://example.auth0.com/oauth/token".to_string(),
            userinfo_endpoint: None,
            allowed_domains: vec!["*.auth0.com".to_string()],
        };

        // Test serialization
        let json = serde_json::to_string(&provider).unwrap();
        assert!(json.contains("oidc"));
        assert!(json.contains("auth0"));
    }

    #[test]
    fn test_gateway_config_with_provider() {
        let config = GatewayConfig {
            mcp_gateway_url: "http://gateway.internal".to_string(),
            trace_id_header: "X-Request-ID".to_string(),
            enabled: true,
            provider: Some(ProviderConfig::AuthKit {
                issuer: "https://example.authkit.app".to_string(),
                jwks_uri: None,
                audience: None,
            }),
        };

        assert!(config.enabled);
        assert!(config.provider.is_some());
    }

    #[test]
    fn test_gateway_config_without_provider() {
        let config = GatewayConfig {
            mcp_gateway_url: "http://gateway.internal".to_string(),
            trace_id_header: "X-Request-ID".to_string(),
            enabled: false,
            provider: None,
        };

        assert!(!config.enabled);
        assert!(config.provider.is_none());
    }

    #[test]
    fn test_ensure_https_url() {
        // Test with https:// already present
        assert_eq!(
            GatewayConfig::ensure_https_url("https://example.com".to_string()).unwrap(),
            "https://example.com"
        );

        // Test with http:// should fail
        let result = GatewayConfig::ensure_https_url("http://example.com".to_string());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Auth provider URLs must use HTTPS"));

        // Test without protocol - should add https://
        assert_eq!(
            GatewayConfig::ensure_https_url("example.com".to_string()).unwrap(),
            "https://example.com"
        );

        // Test with domain and path
        assert_eq!(
            GatewayConfig::ensure_https_url("example.com/path".to_string()).unwrap(),
            "https://example.com/path"
        );

        // Test with AuthKit style domain
        assert_eq!(
            GatewayConfig::ensure_https_url("divine-lion-50-staging.authkit.app".to_string())
                .unwrap(),
            "https://divine-lion-50-staging.authkit.app"
        );

        // Test with http://localhost should also fail
        let result = GatewayConfig::ensure_https_url("http://localhost:8080".to_string());
        assert!(result.is_err());
    }
}
