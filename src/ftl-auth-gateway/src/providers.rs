use serde::{Deserialize, Serialize};

/// Trait for authentication providers
pub trait AuthProvider: Send + Sync {
    /// Get the JWKS URI for this provider
    fn jwks_uri(&self) -> &str;

    /// Get the issuer for this provider
    fn issuer(&self) -> &str;

    /// Get the audience for this provider (optional)
    fn audience(&self) -> Option<&str>;

    /// Get allowed domains for JWKS fetching
    #[allow(dead_code)]
    fn allowed_domains(&self) -> Vec<&str>;

    /// Get discovery metadata for OAuth 2.0
    fn discovery_metadata(&self, resource_url: &str) -> DiscoveryMetadata;

    /// Extract the provider-specific user context from claims
    fn extract_user_context(&self, claims: &crate::auth::Claims) -> UserContext {
        UserContext {
            id: claims.sub.clone(),
            email: claims.email.clone(),
            provider: self.name().to_string(),
        }
    }

    /// Get the provider name
    fn name(&self) -> &str;
}

/// User context extracted from JWT claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext {
    pub id: String,
    pub email: Option<String>,
    pub provider: String,
}

/// OAuth 2.0 discovery metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryMetadata {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub jwks_uri: String,
    pub userinfo_endpoint: Option<String>,
    pub revocation_endpoint: Option<String>,
    pub introspection_endpoint: Option<String>,
}

/// Generic OIDC provider configuration
#[derive(Debug, Clone, Deserialize)]
pub struct OidcProviderConfig {
    pub name: String,
    pub issuer: String,
    pub jwks_uri: String,
    pub audience: Option<String>,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: Option<String>,
    #[allow(dead_code)]
    pub allowed_domains: Vec<String>,
}

/// Generic OIDC provider implementation
pub struct OidcProvider {
    config: OidcProviderConfig,
}

impl OidcProvider {
    pub fn new(config: OidcProviderConfig) -> Self {
        Self { config }
    }
}

impl AuthProvider for OidcProvider {
    fn jwks_uri(&self) -> &str {
        &self.config.jwks_uri
    }

    fn issuer(&self) -> &str {
        &self.config.issuer
    }

    fn audience(&self) -> Option<&str> {
        self.config.audience.as_deref()
    }

    fn allowed_domains(&self) -> Vec<&str> {
        self.config
            .allowed_domains
            .iter()
            .map(String::as_str)
            .collect()
    }

    fn discovery_metadata(&self, _resource_url: &str) -> DiscoveryMetadata {
        DiscoveryMetadata {
            issuer: self.config.issuer.clone(),
            authorization_endpoint: self.config.authorization_endpoint.clone(),
            token_endpoint: self.config.token_endpoint.clone(),
            jwks_uri: self.config.jwks_uri.clone(),
            userinfo_endpoint: self.config.userinfo_endpoint.clone(),
            revocation_endpoint: None,
            introspection_endpoint: None,
        }
    }

    fn name(&self) -> &str {
        &self.config.name
    }
}

/// `WorkOS` `AuthKit` provider
pub struct AuthKitProvider {
    issuer: String,
    jwks_uri: String,
    audience: Option<String>,
}

impl AuthKitProvider {
    pub fn new(issuer: String, jwks_uri: Option<String>, audience: Option<String>) -> Self {
        let jwks_uri = jwks_uri.unwrap_or_else(|| format!("{issuer}/oauth2/jwks"));
        Self {
            issuer,
            jwks_uri,
            audience,
        }
    }
}

impl AuthProvider for AuthKitProvider {
    fn jwks_uri(&self) -> &str {
        &self.jwks_uri
    }

    fn issuer(&self) -> &str {
        &self.issuer
    }

    fn audience(&self) -> Option<&str> {
        self.audience.as_deref()
    }

    fn allowed_domains(&self) -> Vec<&str> {
        vec!["*.authkit.app"]
    }

    fn discovery_metadata(&self, _resource_url: &str) -> DiscoveryMetadata {
        DiscoveryMetadata {
            issuer: self.issuer.clone(),
            authorization_endpoint: format!("{}/oauth2/authorize", self.issuer),
            token_endpoint: format!("{}/oauth2/token", self.issuer),
            jwks_uri: self.jwks_uri.clone(),
            userinfo_endpoint: Some(format!("{}/oauth2/userinfo", self.issuer)),
            revocation_endpoint: Some(format!("{}/oauth2/revoke", self.issuer)),
            introspection_endpoint: Some(format!("{}/oauth2/introspect", self.issuer)),
        }
    }

    fn name(&self) -> &'static str {
        "authkit"
    }
}

/// Provider registry to support multiple providers
pub struct ProviderRegistry {
    providers: Vec<Box<dyn AuthProvider>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    pub fn add_provider(&mut self, provider: Box<dyn AuthProvider>) {
        self.providers.push(provider);
    }

    /// Find a provider by issuer
    #[allow(dead_code)]
    pub fn find_by_issuer(&self, issuer: &str) -> Option<&dyn AuthProvider> {
        self.providers
            .iter()
            .find(|p| p.issuer() == issuer)
            .map(std::convert::AsRef::as_ref)
    }

    /// Get all providers
    pub fn providers(&self) -> &[Box<dyn AuthProvider>] {
        &self.providers
    }

    /// Get all allowed domains across all providers
    #[allow(dead_code)]
    pub fn all_allowed_domains(&self) -> Vec<&str> {
        self.providers
            .iter()
            .flat_map(|p| p.allowed_domains())
            .collect()
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}
