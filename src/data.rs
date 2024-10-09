use anyhow::Context;
use itertools::izip;
use polars::prelude::*;
use std::fmt::Display;

use bytes::Bytes;
use chrono::{DateTime, Utc};
use serde_json::{Map, Value};
use url::Url;
use uuid::Uuid;

pub type HeadersJson = Map<String, Value>;

#[derive(Clone, PartialEq, Debug)]
pub struct RequestID {
    inner: String,
}

impl RequestID {
    pub fn new() -> Self {
        let uuid = Uuid::new_v4();
        Self {
            inner: format!("request:{uuid}"),
        }
    }
}
impl Default for RequestID {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for RequestID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        String::fmt(&self.inner, f)
    }
}

trait ToFromDf {
    fn to_df(data: &[Self]) -> Result<DataFrame, PolarsError>
    where
        Self: Sized;
    fn from_df(df: DataFrame) -> anyhow::Result<Vec<Self>>
    where
        Self: Sized;
}

#[derive(PartialEq, Debug, Clone)]
pub struct GetResponse {
    pub url: Url,
    pub request_url: Url,
    pub status_code: u16,
    pub data: Option<Bytes>,
    pub headers: Option<HeadersJson>,
    pub timestamp: DateTime<Utc>,
    pub retry_attempt: u8,
    pub is_final: bool,
    pub fetcher_name: String,
    pub fetcher_version: String,
    pub fetcher_calibre: u8,
}

impl ToFromDf for GetResponse {
    /// Convert a vec of GetResponses into a dataframe
    // TODO abstract the serialising/deserialising
    fn to_df(data: &[Self]) -> Result<DataFrame, PolarsError> {
        let response_data: ChunkedArray<BinaryType> = {
            // Data is Bytes, which is basically Rc, so clone is cheap (just a reference)
            let data: Vec<Option<Bytes>> = data.iter().map(|row| row.data.clone()).collect();
            let data = LargeBinaryArray::from_iter(data.iter().map(|d| d.as_ref()));
            data.into()
        };
        let headers = data
            .iter()
            .map(|d| {
                d.headers
                    .as_ref()
                    .map(|h| serde_json::to_string(h).unwrap())
            })
            .collect::<Vec<_>>();
        let timestamp = data
            .iter()
            .map(|d| {
                d.timestamp
                    .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
            })
            .collect::<Vec<_>>();

        df![
            "url" => data.iter().map(|d| d.url.to_string()).collect::<Vec<_>>(),
            "request_url" => data.iter().map(|d| d.request_url.to_string()).collect::<Vec<_>>(),
            "status_code" => data.iter().map(|d| d.status_code).collect::<Vec<_>>(),
            "data" => response_data,
            "headers" => headers,
            "timestamp" => timestamp,
            "retry_attempt" => data.iter().map(|d| d.retry_attempt).collect::<Vec<_>>(),
            "is_final" => data.iter().map(|d| d.is_final).collect::<Vec<_>>(),
            "fetcher_name" => data.iter().map(|d| d.fetcher_name.clone()).collect::<Vec<_>>(),
            "fetcher_version" => data.iter().map(|d| d.fetcher_version.clone()).collect::<Vec<_>>(),
            "fetcher_calibre" => data.iter().map(|d| d.fetcher_calibre).collect::<Vec<_>>(),
        ]
    }

    /// Convert a dataframe into a vec of GetResponses
    fn from_df(df: DataFrame) -> anyhow::Result<Vec<Self>> {
        let urls = df.column("url")?.str()?.into_iter();
        let request_urls = df.column("request_url")?.str()?.into_iter();
        let status_codes = df.column("status_code")?.u16()?.into_iter();
        // TODO consider just using [u8 (or vec)] to save all this.
        let datas = df
            .column("data")?
            .binary()?
            .into_iter()
            .map(|val| {
                val.map(|d| {
                    let data = d.iter().map(|v| v.clone()).collect::<Vec<_>>();
                    Bytes::from(data)
                })
            })
            .into_iter();
        let headers = df.column("headers")?.str()?.into_iter();
        let timestamps = df.column("timestamp")?.str()?.into_iter();
        let retry_attempts = df.column("retry_attempt")?.u8()?.into_iter();
        let is_finals = df.column("is_final")?.bool()?.into_iter();
        let fetcher_names = df.column("fetcher_name")?.str()?.into_iter();
        let fetcher_versions = df.column("fetcher_version")?.str()?.into_iter();
        let fetcher_calibres = df.column("fetcher_calibre")?.u8()?.into_iter();
        izip!(
            urls,
            request_urls,
            status_codes,
            datas,
            headers,
            timestamps,
            retry_attempts,
            is_finals,
            fetcher_names,
            fetcher_versions,
            fetcher_calibres
        )
        .map(
            |(
                url,
                request_url,
                status_code,
                data,
                headers,
                timestamp,
                retry_attempt,
                is_final,
                fetcher_name,
                fetcher_version,
                fetcher_calibre,
            )| {
                let headers_val = if let Some(headers_json) = headers {
                    let val: serde_json::Value = serde_json::from_str(headers_json)?;
                    let map = val.as_object().context("json is object")?;
                    Some(map.clone())
                } else {
                    None
                };

                Ok(Self {
                    url: url.context("url")?.parse()?,
                    request_url: request_url.context("request_url")?.parse()?,
                    status_code: status_code.context("status_code")?,
                    data: data.into(),
                    headers: headers_val,
                    timestamp: timestamp.context("timestamp")?.parse()?,
                    retry_attempt: retry_attempt.context("retry_attempt")?,
                    is_final: is_final.context("is_final")?,
                    fetcher_name: fetcher_name.context("fetcher_name")?.to_string(),
                    fetcher_version: fetcher_version.context("fetcher_version")?.to_string(),
                    fetcher_calibre: fetcher_calibre.context("fetcher_calibre")?,
                })
            },
        )
        .collect()
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct HeadResponse {
    pub url: Url,
    pub request_url: Url,
    pub status_code: u16,
    pub headers: Option<HeadersJson>,
    pub timestamp: DateTime<Utc>,
    pub retry_attempt: u8,
    pub is_final: bool,
    pub fetcher_name: String,
    pub fetcher_version: String,
    pub fetcher_calibre: u8,
}

impl ToFromDf for HeadResponse {
    /// Convert a vec of structs to a dataframe.
    fn to_df(data: &[Self]) -> Result<DataFrame, PolarsError> {
        let headers = data
            .iter()
            .map(|d| {
                d.headers
                    .as_ref()
                    .map(|h| serde_json::to_string(h).unwrap())
            })
            .collect::<Vec<_>>();
        let timestamp = data
            .iter()
            .map(|d| {
                d.timestamp
                    .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
            })
            .collect::<Vec<_>>();

        df![
            "url" => data.iter().map(|d| d.url.to_string()).collect::<Vec<_>>(),
            "request_url" => data.iter().map(|d| d.request_url.to_string()).collect::<Vec<_>>(),
            "status_code" => data.iter().map(|d| d.status_code).collect::<Vec<_>>(),
            "headers" => headers,
            "timestamp" => timestamp,
            "retry_attempt" => data.iter().map(|d| d.retry_attempt).collect::<Vec<_>>(),
            "is_final" => data.iter().map(|d| d.is_final).collect::<Vec<_>>(),
            "fetcher_name" => data.iter().map(|d| d.fetcher_name.clone()).collect::<Vec<_>>(),
            "fetcher_version" => data.iter().map(|d| d.fetcher_version.clone()).collect::<Vec<_>>(),
            "fetcher_calibre" => data.iter().map(|d| d.fetcher_calibre).collect::<Vec<_>>(),
        ]
    }

    /// Convert a dataframe into a vec of HeadResponses
    fn from_df(df: DataFrame) -> anyhow::Result<Vec<Self>> {
        let urls = df.column("url")?.str()?.into_iter();
        let request_urls = df.column("request_url")?.str()?.into_iter();
        let status_codes = df.column("status_code")?.u16()?.into_iter();
        let headers = df.column("headers")?.str()?.into_iter();
        let timestamps = df.column("timestamp")?.str()?.into_iter();
        let retry_attempts = df.column("retry_attempt")?.u8()?.into_iter();
        let is_finals = df.column("is_final")?.bool()?.into_iter();
        let fetcher_names = df.column("fetcher_name")?.str()?.into_iter();
        let fetcher_versions = df.column("fetcher_version")?.str()?.into_iter();
        let fetcher_calibres = df.column("fetcher_calibre")?.u8()?.into_iter();
        izip!(
            urls,
            request_urls,
            status_codes,
            headers,
            timestamps,
            retry_attempts,
            is_finals,
            fetcher_names,
            fetcher_versions,
            fetcher_calibres
        )
        .map(
            |(
                url,
                request_url,
                status_code,
                headers,
                timestamp,
                retry_attempt,
                is_final,
                fetcher_name,
                fetcher_version,
                fetcher_calibre,
            )| {
                let headers_val = if let Some(headers_json) = headers {
                    let val: serde_json::Value = serde_json::from_str(headers_json)?;
                    let map = val.as_object().context("json is object")?;
                    Some(map.clone())
                } else {
                    None
                };

                Ok(Self {
                    url: url.context("url")?.parse()?,
                    request_url: request_url.context("request_url")?.parse()?,
                    status_code: status_code.context("status_code")?,
                    headers: headers_val,
                    timestamp: timestamp.context("timestamp")?.parse()?,
                    retry_attempt: retry_attempt.context("retry_attempt")?,
                    is_final: is_final.context("is_final")?,
                    fetcher_name: fetcher_name.context("fetcher_name")?.to_string(),
                    fetcher_version: fetcher_version.context("fetcher_version")?.to_string(),
                    fetcher_calibre: fetcher_calibre.context("fetcher_calibre")?,
                })
            },
        )
        .collect()
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct Metadata {
    pub state: String,
    pub url: Url,
    pub logs: Option<String>,
    pub traceback: Option<String>,
    pub run_time: Option<f64>,
}

impl ToFromDf for Metadata {
    fn from_df(df: DataFrame) -> anyhow::Result<Vec<Self>> {
        let state = df.column("state")?.str()?.into_iter();
        let urls = df.column("url")?.str()?.into_iter();
        let logs = df.column("logs")?.str()?.into_iter();
        let traceback = df.column("traceback")?.str()?.into_iter();
        let run_time = df.column("run_time")?.f64()?.into_iter();
        izip!(state, urls, logs, traceback, run_time)
            .map(
                |(state, url, logs, traceback, run_time)|
                {
                    Ok(Self {
                        state: state.context("state")?.parse()?,
                        url: url.context("url")?.parse()?,
                        logs: logs.map(|l| l.to_string()),
                        traceback: traceback.map(|l| l.to_string()),
                        run_time,
                    })
                },
            )
            .collect()
    }

    /// Convert a vec of structs to a dataframe.
    fn to_df(data: &[Self]) -> Result<DataFrame, PolarsError> {
        df![
            "state" => data.iter().map(|d| d.state.to_string()).collect::<Vec<_>>(),
            "url" => data.iter().map(|d| d.url.to_string()).collect::<Vec<_>>(),
            "logs" => data.iter().map(|d| d.logs.clone()).collect::<Vec<_>>(),
            "traceback" => data.iter().map(|d| d.traceback.clone()).collect::<Vec<_>>(),
            "run_time" => data.iter().map(|d| d.run_time.clone()).collect::<Vec<_>>(),
        ]
    }
}

pub struct PersistedData<T> {
    data: T,
    pub(crate) request_id: RequestID,
}

impl<T> PersistedData<T> {
    pub fn wrap_data(data: Vec<T>) -> Vec<Self> {
        // consider Rc
        let request_id = RequestID::new();
        data.into_iter()
            .map(|record| Self::new_with_id(record, request_id.clone()))
            .collect()
    }

    pub fn wrap_with_id(data: Vec<T>, request_id: RequestID) -> Vec<Self> {
        data.into_iter()
            .map(|record| Self::new_with_id(record, request_id.clone()))
            .collect()
    }

    pub fn new_with_id(data: T, request_id: RequestID) -> Self {
        Self { data, request_id }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use polars::testing::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    fn fake_get_data() -> Vec<GetResponse> {
        let headers = json![{"foo": "bar"}].as_object().unwrap().clone();
        vec![
            GetResponse {
                url: "http://thema.ai".parse().unwrap(),
                request_url: "http://thema.ai".parse().unwrap(),
                status_code: 301,
                data: None,
                headers: None,
                timestamp: "2024-01-01T12:13:14Z".parse().unwrap(),
                retry_attempt: 0,
                is_final: false,
                fetcher_name: "Test".to_string(),
                fetcher_version: "v0.0.1".to_string(),
                fetcher_calibre: 0,
            },
            GetResponse {
                url: "http://thema.ai".parse().unwrap(),
                request_url: "http://thema.ai".parse().unwrap(),
                status_code: 200,
                data: Some(Bytes::from("data")),
                headers: Some(headers),
                timestamp: "2024-01-01T12:13:14Z".parse().unwrap(),
                retry_attempt: 0,
                is_final: true,
                fetcher_name: "Test".to_string(),
                fetcher_version: "v0.0.1".to_string(),
                fetcher_calibre: 0,
            },
        ]
    }

    fn fake_head_data() -> Vec<HeadResponse> {
        vec![
            HeadResponse {
                url: "http://thema.ai".parse().unwrap(),
                request_url: "http://thema.ai".parse().unwrap(),
                status_code: 301,
                headers: None,
                timestamp: "2024-01-01T12:13:14Z".parse().unwrap(),
                retry_attempt: 0,
                is_final: false,
                fetcher_name: "Test".to_string(),
                fetcher_version: "v0.0.1".to_string(),
                fetcher_calibre: 0,
            },
            HeadResponse {
                url: "http://thema.ai".parse().unwrap(),
                request_url: "http://thema.ai".parse().unwrap(),
                status_code: 200,
                headers: None,
                timestamp: "2024-01-01T12:13:14Z".parse().unwrap(),
                retry_attempt: 0,
                is_final: true,
                fetcher_name: "Test".to_string(),
                fetcher_version: "v0.0.1".to_string(),
                fetcher_calibre: 0,
            },
        ]
    }

    fn fake_metadata() -> Metadata {
        Metadata {
            state: "success".into(),
            url: "https://thema.ai/".parse().unwrap(),
            logs: Some("foo bar, bar baz".into()),
            traceback: None,
            run_time: Some(0.112),
        }
    }

    #[test]
    fn wrapped_data_has_same_generated_id_for_all_records() {
        let data = fake_get_data();

        let persisted = PersistedData::wrap_data(data);

        let ids: Vec<_> = persisted.iter().map(|p| p.request_id.clone()).collect();

        assert_eq!(ids, vec![ids[0].clone(), ids[0].clone()]);
    }

    #[test]
    fn wrapped_data_has_same_manual_id_for_all_records() {
        let data = fake_get_data();
        let id = RequestID::new();

        let persisted = PersistedData::wrap_with_id(data, id);

        let ids: Vec<_> = persisted.iter().map(|p| p.request_id.clone()).collect();

        assert_eq!(ids, vec![ids[0].clone(), ids[0].clone()]);
    }

    #[test]
    fn head_data_turned_into_df() {
        let data = fake_head_data();
        let expected = df![
            "url" => ["http://thema.ai/", "http://thema.ai/"],
            "request_url" => ["http://thema.ai/", "http://thema.ai/"],
            "status_code" => [301_u16, 200],
            "headers" => [None::<String>, None],
            "timestamp" => ["2024-01-01T12:13:14Z", "2024-01-01T12:13:14Z"],
            "retry_attempt" => [0_u8, 0],
            "is_final" => [false, true],
            "fetcher_name" => ["Test", "Test"],
            "fetcher_version" => ["v0.0.1", "v0.0.1"],
            "fetcher_calibre" => [0_u8, 0],
        ]
        .unwrap();

        let df = HeadResponse::to_df(&data).unwrap();

        assert_eq!(df, expected)
    }

    #[test]
    fn get_data_turned_into_df() {
        let data = fake_get_data();
        let response_data: ChunkedArray<BinaryType> = {
            let data = LargeBinaryArray::from_iter([None, Some(Bytes::from("data"))]);
            data.into()
        };

        let expected = df![
            "url" => ["http://thema.ai/", "http://thema.ai/"],
            "request_url" => ["http://thema.ai/", "http://thema.ai/"],
            "status_code" => [301_u16, 200],
            "data" => response_data,
            "headers" => [None::<String>, Some(r#"{"foo":"bar"}"#.to_string())],
            "timestamp" => ["2024-01-01T12:13:14Z", "2024-01-01T12:13:14Z"],
            "retry_attempt" => [0_u8, 0],
            "is_final" => [false, true],
            "fetcher_name" => ["Test", "Test"],
            "fetcher_version" => ["v0.0.1", "v0.0.1"],
            "fetcher_calibre" => [0_u8, 0],
        ]
        .unwrap();

        let df = GetResponse::to_df(&data).unwrap();

        assert_eq!(df, expected)
    }

    #[test]
    fn get_data_round_tripped() -> Result<()> {
        let data = fake_get_data();
        let df = GetResponse::to_df(&data)?;

        let deserialised = GetResponse::from_df(df)?;

        assert_eq!(data, deserialised);
        Ok(())
    }

    #[test]
    fn head_data_round_tripped() -> Result<()> {
        let data = fake_head_data();
        let df = HeadResponse::to_df(&data)?;

        let deserialised = HeadResponse::from_df(df)?;

        assert_eq!(data, deserialised);
        Ok(())
    }

    #[test]
    fn metadata_round_tripped() -> Result<()> {
        let data = fake_metadata();
        let df = Metadata::to_df(&[data.clone()])?;

        let deserialised = Metadata::from_df(df)?;

        assert_eq!(deserialised.len(), 1);
        assert_eq!(data, deserialised[0]);
        Ok(())
    }
}
