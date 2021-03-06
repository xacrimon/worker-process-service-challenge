use anyhow::{anyhow, Result};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use tonic::{metadata::MetadataMap, Request, Status};

/// Example secret. In a real setting you may want to use an asymmetric keypair for signing instead of HMAC.
const JWT_SECRET: &[u8] = b"secret_charlie";

const AUTHORIZATION_HEADER_NAME: &str = "authorization";
const AUTHORIZATION_TYPE: &str = "Bearer";

/// The claims the JWT token must have.
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub exp: usize,
    pub username: String,
    pub spawn: bool,
    pub stop: bool,
    pub stream_log: bool,
    pub status: bool,
}

/// Extract the auth token from the gRPC request metadata.
fn get_auth_token<'a>(metadata: &'a MetadataMap) -> Result<&'a str> {
    let header = metadata
        .get(AUTHORIZATION_HEADER_NAME)
        .ok_or_else(|| anyhow!("no authorization token provided"))?;

    let string_header = header.to_str()?;
    let mut splits = string_header.split(' ');
    let given_auth_type = splits
        .next()
        .ok_or_else(|| anyhow!("malformed authorization header"))?;

    if given_auth_type != AUTHORIZATION_TYPE {
        return Err(anyhow!("authorization type not bearer"));
    }

    let token = splits
        .next()
        .ok_or_else(|| anyhow!("no authorization token provided"))?;

    Ok(token)
}

/// Extract and validate claims from a gRPC request.
pub fn validate_claims<T>(request: &Request<T>) -> Result<Claims, Status> {
    let meta = request.metadata();
    let auth_token =
        get_auth_token(meta).map_err(|error| Status::invalid_argument(error.to_string()))?;

    let key = DecodingKey::from_secret(JWT_SECRET);
    let validation = Validation::default();
    let token = jsonwebtoken::decode::<Claims>(&auth_token, &key, &validation)
        .map_err(|error| Status::invalid_argument(error.to_string()))?;

    Ok(token.claims)
}

/// Create a new JWT with a set of claims.
pub fn issue_jwt(claims: Claims) -> Result<String> {
    let header = Header::default();
    let key = EncodingKey::from_secret(JWT_SECRET);
    let token = jsonwebtoken::encode(&header, &claims, &key)?;
    Ok(token)
}
