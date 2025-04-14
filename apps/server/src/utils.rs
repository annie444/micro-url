#[cfg(feature = "headers")]
use std::collections::BTreeMap;
use std::{sync::LazyLock, time::Duration};

#[cfg(feature = "headers")]
use axum::http::header::HeaderMap;
use chrono::TimeDelta;
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
use regex::{Regex, RegexBuilder};
#[cfg(feature = "headers")]
use sea_orm::query::JsonValue;
use serde::{Deserialize, Serialize};
#[cfg(feature = "headers")]
use serde_json::value::Value;
use ts_rs::TS;
use utoipa::ToSchema;

#[cfg(feature = "headers")]
use crate::error::ServerError;

static DURATION_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    RegexBuilder::new(
        r"
        ^
        (?:
            (?<year>\d+)
            (?:y(?:ea)?(?:r)?(?:s)?)
        )?
        (?:
            (?<month>\d+)
            (?:mon(?:th)?(?:s)?)
        )?
        (?:
            (?<week>\d+)
            (?:w(?:ee)?(?:k)?(?:s)?)
        )?
        (?:
            (?<day>\d+)
            (?:d(?:a)?(?:y)?(?:s)?)
        )?
        (?:
            (?<hour>\d+)
            (?:h(?:ou)?(?:r)?(?:s)?)
        )?
        (?:
            (?<minute>\d+)
            (?:m(?:in)?(?:ute)?(?:s)?)
        )?
        (?:
            (?<second>\d+)
            (?:s(?:ec)?(?:ond)?(?:s)?)
        )?
        (?:
            (?<millisecond>\d+)
            (?:
                (?:ms)|
                (?:milli(?:sec)?(?:ond)?(?:s)?)
            )
        )?
        (?:
            (?<microsecond>\d+)
            (?:
                (?:µs)|
                (?:us)|
                (?:micro(?:sec)?(?:ond)?(?:s)?)
            )
        )?
        (?:
            (?<nanosecond>\d+)
            (?:
                (?:ns)|
                (?:nano(?:sec)?(?:ond)?(?:s)?)
            )
        )?
        $",
    )
    .case_insensitive(true)
    .unicode(true)
    .multi_line(false)
    .ignore_whitespace(true)
    .build()
    .expect("Unable to compile Regex pattern")
});

pub(crate) fn parse_duration(s: &str) -> Result<Duration, ServerError> {
    let mut time = Duration::new(0, 0);
    if let Some(cap) = (&*DURATION_PATTERN).captures(s) {
        if let Some(years) = cap.name("year") {
            let year: u64 = years.as_str().parse()?;
            let yts = year * 365 * 24 * 60 * 60;
            let year_duration = Duration::from_secs(yts);
            time += year_duration;
        }
        if let Some(months) = cap.name("month") {
            let month: u64 = months.as_str().parse()?;
            let mts = month * 30 * 24 * 60 * 60;
            let month_duration = Duration::from_secs(mts);
            time += month_duration;
        }
        if let Some(weeks) = cap.name("week") {
            let week: u64 = weeks.as_str().parse()?;
            let wts = week * 7 * 24 * 60 * 60;
            let week_duration = Duration::from_secs(wts);
            time += week_duration;
        }
        if let Some(days) = cap.name("day") {
            let day: u64 = days.as_str().parse()?;
            let dts = day * 24 * 60 * 60;
            let day_duration = Duration::from_secs(dts);
            time += day_duration;
        }
        if let Some(hours) = cap.name("hour") {
            let hour: u64 = hours.as_str().parse()?;
            let hts = hour * 60 * 60;
            let hour_duration = Duration::from_secs(hts);
            time += hour_duration;
        }
        if let Some(mins) = cap.name("minute") {
            let min: u64 = mins.as_str().parse()?;
            let mts = min * 60;
            let min_duration = Duration::from_secs(mts);
            time += min_duration;
        }
        if let Some(secs) = cap.name("second") {
            let sec: u64 = secs.as_str().parse()?;
            let sec_duration = Duration::from_secs(sec);
            time += sec_duration;
        }
        if let Some(millis) = cap.name("millisecond") {
            let ms: u64 = millis.as_str().parse()?;
            let ms_duration = Duration::from_millis(ms);
            time += ms_duration;
        }
        if let Some(micros) = cap.name("microseconds") {
            let us: u64 = micros.as_str().parse()?;
            let us_duration = Duration::from_micros(us);
            time += us_duration;
        }
        if let Some(nanos) = cap.name("nanoseconds") {
            let ns: u64 = nanos.as_str().parse()?;
            let ns_duration = Duration::from_nanos(ns);
            time += ns_duration;
        }
    }
    Ok(time)
}

pub(crate) fn parse_time_delta(s: &str) -> Result<TimeDelta, ServerError> {
    let mut time = TimeDelta::zero();
    if let Some(cap) = (&*DURATION_PATTERN).captures(s) {
        if let Some(years) = cap.name("year") {
            let year: i64 = years.as_str().parse()?;
            let ytd = year * 365;
            let year_duration = TimeDelta::days(ytd);
            time += year_duration;
        }
        if let Some(months) = cap.name("month") {
            let month: i64 = months.as_str().parse()?;
            let mtd = month * 30;
            let month_duration = TimeDelta::days(mtd);
            time += month_duration;
        }
        if let Some(weeks) = cap.name("week") {
            let week: i64 = weeks.as_str().parse()?;
            let wtd = week * 7;
            let week_duration = TimeDelta::days(wtd);
            time += week_duration;
        }
        if let Some(days) = cap.name("day") {
            let day: i64 = days.as_str().parse()?;
            let day_duration = TimeDelta::days(day);
            time += day_duration;
        }
        if let Some(hours) = cap.name("hour") {
            let hour: i64 = hours.as_str().parse()?;
            let hour_duration = TimeDelta::hours(hour);
            time += hour_duration;
        }
        if let Some(mins) = cap.name("minute") {
            let min: i64 = mins.as_str().parse()?;
            let min_duration = TimeDelta::minutes(min);
            time += min_duration;
        }
        if let Some(secs) = cap.name("second") {
            let sec: i64 = secs.as_str().parse()?;
            let sec_duration = TimeDelta::seconds(sec);
            time += sec_duration;
        }
        if let Some(millis) = cap.name("millisecond") {
            let ms: i64 = millis.as_str().parse()?;
            let ms_duration = TimeDelta::milliseconds(ms);
            time += ms_duration;
        }
        if let Some(micros) = cap.name("microseconds") {
            let us: i64 = micros.as_str().parse()?;
            let us_duration = TimeDelta::microseconds(us);
            time += us_duration;
        }
        if let Some(nanos) = cap.name("nanoseconds") {
            let ns: i64 = nanos.as_str().parse()?;
            let ns_duration = TimeDelta::nanoseconds(ns);
            time += ns_duration;
        }
    }
    Ok(time)
}

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
