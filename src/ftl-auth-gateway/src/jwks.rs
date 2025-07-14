use anyhow::{anyhow, Result};
use jsonwebtoken::{Algorithm, DecodingKey};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// `JWKS` response structure
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct JwksResponse {
    pub keys: Vec<Jwk>,
}

/// JSON Web Key structure
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Jwk {
    pub kty: String,
    pub kid: Option<String>,
    pub alg: Option<String>,
    pub r#use: Option<String>,
    pub n: Option<String>,
    pub e: Option<String>,
    pub x5c: Option<Vec<String>>,
    pub x5t: Option<String>,
}

/// Type alias for the JWKS cache entry
type JwksCacheEntry = (JwksResponse, std::time::Instant);

/// Type alias for the JWKS cache
type JwksCache = Arc<RwLock<HashMap<String, JwksCacheEntry>>>;

/// Cache for `JWKS` data
static JWKS_CACHE: Lazy<JwksCache> = Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

/// Cache duration (5 minutes)
const CACHE_DURATION: std::time::Duration = std::time::Duration::from_secs(300);

/// Maximum number of `JWKS` URIs to cache (prevent `DoS`)
const MAX_CACHE_SIZE: usize = 100;

/// Fetch `JWKS` from the given URI with caching
pub async fn fetch_jwks(jwks_uri: &str) -> Result<JwksResponse> {
    // Validate URI to prevent cache pollution
    if jwks_uri.is_empty() || jwks_uri.len() > 2048 {
        return Err(anyhow!("Invalid JWKS URI"));
    }

    // Check cache first
    {
        let cache = JWKS_CACHE.read().await;
        if let Some((jwks, timestamp)) = cache.get(jwks_uri) {
            if timestamp.elapsed() < CACHE_DURATION {
                return Ok(jwks.clone());
            }
        }
    }

    // Fetch from network
    let request = spin_sdk::http::Request::builder()
        .method(spin_sdk::http::Method::Get)
        .uri(jwks_uri)
        .header("Accept", "application/json")
        .build();

    let response: spin_sdk::http::Response = spin_sdk::http::send(request)
        .await
        .map_err(|e| anyhow!("Failed to fetch JWKS from {jwks_uri}: {e}"))?;

    if *response.status() != 200 {
        let status = response.status();
        return Err(anyhow!("Failed to fetch JWKS: HTTP {status}"));
    }

    let jwks: JwksResponse = serde_json::from_slice(response.body())?;

    // Update cache
    {
        let mut cache = JWKS_CACHE.write().await;

        // If cache is at max size, remove oldest entry
        if cache.len() >= MAX_CACHE_SIZE {
            // Find and remove the oldest entry
            if let Some(oldest_key) = cache
                .iter()
                .min_by_key(|(_, (_, timestamp))| timestamp)
                .map(|(key, _)| key.clone())
            {
                cache.remove(&oldest_key);
            }
        }

        cache.insert(
            jwks_uri.to_string(),
            (jwks.clone(), std::time::Instant::now()),
        );
    }

    Ok(jwks)
}

/// Get decoding key for a specific key ID
pub async fn get_decoding_key(jwks_uri: &str, kid: &str) -> Result<DecodingKey> {
    let jwks = fetch_jwks(jwks_uri).await?;

    let jwk = jwks
        .keys
        .iter()
        .find(|k| k.kid.as_deref() == Some(kid))
        .ok_or_else(|| anyhow!("Key with id '{kid}' not found in JWKS"))?;

    match jwk.kty.as_str() {
        "RSA" => {
            let n = jwk
                .n
                .as_ref()
                .ok_or_else(|| anyhow!("Missing 'n' in RSA key"))?;
            let e = jwk
                .e
                .as_ref()
                .ok_or_else(|| anyhow!("Missing 'e' in RSA key"))?;

            DecodingKey::from_rsa_components(n, e)
                .map_err(|e| anyhow!("Failed to create RSA key: {e}"))
        }
        "EC" => {
            // For EC keys, we'd need to handle them differently
            // For now, we'll use the x5c certificate if available
            jwk.x5c
                .as_ref()
                .ok_or_else(|| anyhow!("EC key support requires x5c certificate"))
                .and_then(|x5c| {
                    x5c.first()
                        .ok_or_else(|| anyhow!("No certificate found in x5c"))
                        .and_then(|cert| {
                            DecodingKey::from_ec_pem(cert.as_bytes()).map_err(|e| {
                                anyhow!("Failed to create EC key from certificate: {e}")
                            })
                        })
                })
        }
        _ => {
            let kty = &jwk.kty;
            Err(anyhow!("Unsupported key type: {kty}"))
        }
    }
}

/// Get the algorithm from a `JWK`
#[allow(dead_code)]
pub fn get_algorithm(jwk: &Jwk) -> Result<Algorithm> {
    match jwk.alg.as_deref() {
        Some("RS256") => Ok(Algorithm::RS256),
        Some("RS384") => Ok(Algorithm::RS384),
        Some("RS512") => Ok(Algorithm::RS512),
        Some("ES256") => Ok(Algorithm::ES256),
        Some("ES384") => Ok(Algorithm::ES384),
        Some("HS256") => Ok(Algorithm::HS256),
        Some("HS384") => Ok(Algorithm::HS384),
        Some("HS512") => Ok(Algorithm::HS512),
        Some(alg) => Err(anyhow!("Unsupported algorithm: {alg}")),
        None => {
            // Default based on key type
            match jwk.kty.as_str() {
                "RSA" => Ok(Algorithm::RS256),
                "EC" => Ok(Algorithm::ES256),
                _ => {
                    let kty = &jwk.kty;
                    Err(anyhow!("Cannot determine algorithm for key type: {kty}"))
                }
            }
        }
    }
}

/// Clear the `JWKS` cache - useful for testing or forced refresh
#[allow(dead_code)]
pub async fn clear_cache() {
    let mut cache = JWKS_CACHE.write().await;
    cache.clear();
}
