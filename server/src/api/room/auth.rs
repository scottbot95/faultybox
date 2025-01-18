use std::sync::LazyLock;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::{HeaderName, HeaderValue, StatusCode};
use axum::{Json, RequestPartsExt};
use axum::response::{IntoResponse, Response};
use axum_extra::extract::CookieJar;
use headers::{Error, Header};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Validation};
use serde_json::json;
use models::room::api::Claims;

static KEYS: LazyLock<Keys> = LazyLock::new(|| {
    // let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let secret = "secret";
    Keys::new(secret.as_bytes())
});

struct Keys {
    encoding: EncodingKey,
    decoding: DecodingKey
}

impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

pub fn create_token(claims: &Claims) -> Result<String, AuthError> {
    encode(&jsonwebtoken::Header::default(), claims, &KEYS.encoding)
        .map_err(|_| AuthError::TokenCreation)
}

pub struct RoomToken(pub Claims);

impl<S> FromRequestParts<S> for RoomToken
where
    S: Send + Sync
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // extract the token from cookies
        let jar = parts.extract::<CookieJar>()
            .await
            .map_err(|_| AuthError::MissingToken)?;
        
        let token = jar.get("room_token")
            .ok_or(AuthError::MissingToken)?
            .value();
        
        // extract the token from the header
        // let TypedHeader(RoomTokenHeader(token)) = parts
        //     .extract()
        //     .await
        //     .map_err(|err: TypedHeaderRejection| 
        //         if err.is_missing() {
        //             tracing::debug!("Request missing room token");
        //             AuthError::MissingToken
        //         } else {
        //             tracing::debug!("Request had invalid room token: {:?}", err);
        //             AuthError::InvalidToken
        //         })?;

        // decode the token
        let mut validator = Validation::default();
        validator.validate_exp = false;
        validator.required_spec_claims.remove("exp");
        let token_data = decode::<Claims>(token, &KEYS.decoding, &validator)
            .map_err(|err| {
                tracing::debug!("Failed to decode token: {}", err);
                AuthError::InvalidToken
            })?;
        
        Ok(RoomToken(token_data.claims))
    }
}

#[derive(Clone, Debug, PartialEq)]
struct RoomTokenHeader(String);

static ROOM_TOKEN_NAME: HeaderName = HeaderName::from_static("room-token");

impl Header for RoomTokenHeader {
    fn name() -> &'static HeaderName { &ROOM_TOKEN_NAME }

    fn decode<'i, I:Iterator<Item=&'i HeaderValue>>(values: &mut I) -> Result<Self, Error> {
        values
            .next()
            .and_then(|v| Some(v.to_str().ok()?.to_owned()))
            .map(RoomTokenHeader)
            .ok_or_else(Error::invalid)
    }

    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        let value = HeaderValue::from_str(&self.0).unwrap();
        values.extend(std::iter::once(value));
    }
}

#[derive(Debug)]
pub enum AuthError {
    TokenCreation,
    InvalidToken,
    MissingToken,
    NotFound(String),
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::TokenCreation => (StatusCode::INTERNAL_SERVER_ERROR, "Token creation error".to_owned()),
            AuthError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token".to_owned()),
            AuthError::MissingToken => (StatusCode::UNAUTHORIZED, "Missing token".to_owned()),
            AuthError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}