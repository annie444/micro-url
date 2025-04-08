use openidconnect::{
    core::{
        CoreAuthDisplay, CoreAuthPrompt, CoreErrorResponseType, CoreGenderClaim, CoreJsonWebKey,
        CoreJweContentEncryptionAlgorithm, CoreJwsSigningAlgorithm, CoreRevocableToken,
        CoreTokenType,
    },
    Client, EmptyAdditionalClaims, EmptyExtraTokenFields, EndpointMaybeSet, EndpointNotSet,
    EndpointSet, IdTokenFields, RevocationErrorResponseType, StandardErrorResponse,
    StandardTokenIntrospectionResponse, StandardTokenResponse,
};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[typeshare]
pub struct BasicError {
    pub error: String,
}

impl From<String> for BasicError {
    fn from(e: String) -> Self {
        Self { error: e }
    }
}

impl From<&str> for BasicError {
    fn from(e: &str) -> Self {
        Self {
            error: e.to_owned(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[typeshare]
pub struct BasicResponse {
    pub message: String,
}

impl From<String> for BasicResponse {
    fn from(e: String) -> Self {
        Self { message: e }
    }
}

impl From<&str> for BasicResponse {
    fn from(e: &str) -> Self {
        Self {
            message: e.to_owned(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare]
pub struct AuthUrl {
    pub url: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare]
pub struct AuthUrls(pub Vec<AuthUrl>); // New struct to hold a vector of AuthUrl

pub type OidcClient = Client<
    EmptyAdditionalClaims,
    CoreAuthDisplay,
    CoreGenderClaim,
    CoreJweContentEncryptionAlgorithm,
    CoreJsonWebKey,
    CoreAuthPrompt,
    StandardErrorResponse<CoreErrorResponseType>,
    StandardTokenResponse<
        IdTokenFields<
            EmptyAdditionalClaims,
            EmptyExtraTokenFields,
            CoreGenderClaim,
            CoreJweContentEncryptionAlgorithm,
            CoreJwsSigningAlgorithm,
        >,
        CoreTokenType,
    >,
    StandardTokenIntrospectionResponse<EmptyExtraTokenFields, CoreTokenType>,
    CoreRevocableToken,
    StandardErrorResponse<RevocationErrorResponseType>,
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointMaybeSet,
    EndpointMaybeSet,
>;
