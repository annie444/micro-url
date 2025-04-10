#[cfg(feature = "headers")]
use std::collections::BTreeMap;

#[cfg(feature = "headers")]
use axum::http::header::HeaderMap;
use openidconnect::{
    Client, EmptyAdditionalClaims, EmptyExtraTokenFields, EndpointMaybeSet, EndpointNotSet,
    EndpointSet, IdTokenFields, RevocationErrorResponseType, StandardErrorResponse,
    StandardTokenIntrospectionResponse, StandardTokenResponse,
    core::{
        CoreAuthDisplay, CoreAuthPrompt, CoreErrorResponseType, CoreGenderClaim, CoreJsonWebKey,
        CoreJweContentEncryptionAlgorithm, CoreJwsSigningAlgorithm, CoreRevocableToken,
        CoreTokenType,
    },
};
#[cfg(feature = "headers")]
use sea_orm::query::JsonValue;
use serde::{Deserialize, Serialize};
#[cfg(feature = "headers")]
use serde_json::value::Value;
use ts_rs::TS;
use utoipa::ToSchema;

#[cfg(feature = "headers")]
use crate::error::ServerError;

#[cfg(feature = "headers")]
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema, TS)]
#[ts(export, export_to = "../../../js/frontend/src/lib/types/")]
pub struct HeaderMapDef(pub BTreeMap<String, Vec<String>>);

#[cfg(feature = "headers")]
impl TryFrom<HeaderMap> for HeaderMapDef {
    type Error = ServerError;

    fn try_from(hm: HeaderMap) -> Result<Self, Self::Error> {
        let mut map: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for (name, value) in hm.iter() {
            if map.contains_key(name.as_str()) {
                if let Some(val) = map.get_mut(name.as_str()) {
                    val.push(value.to_str()?.to_string());
                } else {
                    map.insert(name.to_string(), vec![value.to_str()?.to_string()]);
                }
            } else {
                map.insert(name.to_string(), vec![value.to_str()?.to_string()]);
            }
        }
        Ok(Self(map))
    }
}

#[cfg(feature = "headers")]
impl From<JsonValue> for HeaderMapDef {
    fn from(js: JsonValue) -> Self {
        let mut map = BTreeMap::new();
        fn map_value(map: &mut BTreeMap<String, Vec<String>>, val: JsonValue) {
            fn map_named_value(map: &mut BTreeMap<String, Vec<String>>, name: String, val: Value) {
                match val {
                    JsonValue::Null => {
                        map.entry(name).or_default();
                    }
                    JsonValue::Number(n) => {
                        if let Some(val) = map.get_mut(&name) {
                            val.push(format!("{}", n));
                        } else {
                            map.insert(name, vec![format!("{}", n)]);
                        }
                    }
                    JsonValue::Bool(b) => {
                        if let Some(val) = map.get_mut(&name) {
                            val.push(format!("{}", b));
                        } else {
                            map.insert(name, vec![format!("{}", b)]);
                        }
                    }
                    JsonValue::String(s) => {
                        if let Some(val) = map.get_mut(&name) {
                            val.push(s);
                        } else {
                            map.insert(name, vec![s]);
                        }
                    }
                    JsonValue::Array(mut a) => {
                        if let Some(val) = map.get_mut(&name) {
                            val.append(
                                &mut a.iter_mut().map(|v| v.to_string()).collect::<Vec<String>>(),
                            );
                        } else {
                            map.insert(
                                name,
                                a.iter_mut().map(|v| v.to_string()).collect::<Vec<String>>(),
                            );
                        }
                    }
                    JsonValue::Object(o) => {
                        for (name, val) in o {
                            map_named_value(map, name.to_owned(), val.to_owned());
                        }
                    }
                };
            }
            match val {
                JsonValue::Bool(b) => {
                    map.insert("bool".to_string(), vec![format!("{}", b)]);
                }
                JsonValue::Number(n) => {
                    map.insert("number".to_string(), vec![format!("{}", n)]);
                }
                JsonValue::String(s) => {
                    map.insert("string".to_string(), vec![s]);
                }
                JsonValue::Array(mut a) => {
                    map.insert(
                        "array".to_string(),
                        a.iter_mut().map(|v| v.to_string()).collect::<Vec<String>>(),
                    );
                }
                JsonValue::Object(o) => {
                    for (name, val) in o.iter() {
                        map_named_value(map, name.to_owned(), val.to_owned());
                    }
                }
                _ => (),
            };
        }
        map_value(&mut map, js);
        Self(map)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export, export_to = "../../../js/frontend/src/lib/types/")]
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

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export, export_to = "../../../js/frontend/src/lib/types/")]
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

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../js/frontend/src/lib/types/")]
pub struct AuthUrl {
    pub url: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../js/frontend/src/lib/types/")]
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
