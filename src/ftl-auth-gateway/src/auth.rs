use jsonwebtoken::{decode, decode_header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use spin_sdk::http::{Request, Response};

use crate::jwks;

/// Configuration for `AuthKit`
#[derive(Debug, Clone)]
pub struct AuthKitConfig {
    pub issuer: String,
    pub jwks_uri: String,
    pub audience: Option<String>,
    pub mcp_gateway_url: String,
}

/// `JWT` Claims structure
#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub sub: String,
    pub iss: String,
    pub aud: Option<Value>,
    pub exp: i64,
    pub iat: i64,
    pub email: Option<String>,
    #[serde(flatten)]
    pub extra: Value,
}

/// Extract bearer token from authorization header
fn extract_bearer_token(auth_header: &str) -> Option<&str> {
    auth_header.strip_prefix("Bearer ").map(str::trim)
}

/// Build authentication error response
pub fn auth_error_response(error: &str, host: Option<&str>) -> Response {
    let www_auth = host.map_or_else(
        || format!(r#"Bearer error="unauthorized", error_description="{error}""#),
        |h| format!(
            r#"Bearer error="unauthorized", error_description="{error}", resource_metadata="https://{h}/.well-known/oauth-protected-resource""#
        ),
    );

    let body = serde_json::json!({
        "error": "unauthorized",
        "error_description": error
    });

    Response::builder()
        .status(401)
        .header("WWW-Authenticate", www_auth)
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .build()
}

/// Verify `JWT` token with proper signature verification
async fn verify_token(token: &str, config: &AuthKitConfig) -> Result<Claims, String> {
    // Decode the header to get the key ID and algorithm
    let header = decode_header(token).map_err(|_| "Invalid token format".to_string())?;

    // Get the key ID from header
    let kid = header
        .kid
        .ok_or_else(|| "Invalid token format".to_string())?;

    // Fetch the appropriate decoding key from JWKS
    let decoding_key = jwks::get_decoding_key(&config.jwks_uri, &kid)
        .await
        .map_err(|e| {
            eprintln!(
                "Failed to get decoding key for kid '{kid}' from {}: {e}",
                &config.jwks_uri
            );
            "Token validation failed".to_string()
        })?;

    // Set up validation parameters
    let mut validation = Validation::new(header.alg);
    validation.set_issuer(&[&config.issuer]);

    if let Some(aud) = &config.audience {
        if aud.is_empty() {
            // Empty audience means don't validate
            eprintln!("Skipping audience validation (empty audience configured)");
            validation.validate_aud = false;
        } else {
            eprintln!("Validating audience: {aud}");
            validation.set_audience(&[aud]);
        }
    } else {
        // No audience configured means don't validate
        eprintln!("Skipping audience validation (no audience configured)");
        validation.validate_aud = false;
    }

    // Validate required claims
    validation.validate_exp = true;
    validation.validate_nbf = true;

    // Decode and verify the token with signature
    let token_data = decode::<Claims>(token, &decoding_key, &validation).map_err(|e| {
        eprintln!("JWT verification failed: {e:?}");
        "Token validation failed".to_string()
    })?;

    let sub = &token_data.claims.sub;
    eprintln!("Token verified successfully for subject: {sub}");
    Ok(token_data.claims)
}

/// Verify the request has valid authentication
pub async fn verify_request(
    req: &Request,
    config: &AuthKitConfig,
    host: Option<&str>,
) -> Result<Claims, Response> {
    // Extract authorization header
    let auth_header = req
        .headers()
        .find(|(name, _)| name.eq_ignore_ascii_case("authorization"))
        .and_then(|(_, value)| value.as_str());

    let Some(auth) = auth_header else {
        return Err(auth_error_response("Missing authorization header", host));
    };

    let Some(token) = extract_bearer_token(auth) else {
        return Err(auth_error_response(
            "Invalid authorization header format",
            host,
        ));
    };

    // Debug logging - remove or reduce for production
    // eprintln!("Verifying token with issuer: {}", &config.issuer);
    // eprintln!("JWKS URI: {}", &config.jwks_uri);

    match verify_token(token, config).await {
        Ok(claims) => Ok(claims),
        Err(e) => Err(auth_error_response(&e, host)),
    }
}

