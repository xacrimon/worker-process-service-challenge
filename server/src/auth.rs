use anyhow::{anyhow, Result};
use jsonwebtoken::{DecodingKey, Validation, Header, EncodingKey};
use serde::{Deserialize, Serialize};
use tonic::{metadata::MetadataMap, Request, Status};

const JWT_SECRET: &[u8] = b"secret_charlie";
const AUTHORIZATION_HEADER_NAME: &str = "Authorization";
const AUTHORIZATION_TYPE: &str = "Bearer";

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    exp: usize,
    pub username: String,
    pub spawn: bool,
    pub stop: bool,
    pub stream_log: bool,
    pub status: bool,
}

fn get_auth_token<'a>(metadata: &'a MetadataMap) -> Result<&'a str> {
    let header = metadata
        .get(AUTHORIZATION_HEADER_NAME)
        .ok_or_else(|| anyhow!("no authorization token provided"))?;

    let string_header = header.to_str()?;
    let mut splits = string_header.split(" ");
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

pub fn issue_jwt(claims: Claims) -> Result<String> {
    let header = Header::default();
    let key = EncodingKey::from_secret(JWT_SECRET);
    let token = jsonwebtoken::encode(&header, &claims, &key)?;
    Ok(token)
}
