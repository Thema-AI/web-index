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

impl Display for RequestID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        String::fmt(&self.inner, f)
    }
}

pub struct GetData {
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

pub struct HeadData {
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

pub struct Metadata {
    pub state: String,
    pub url: Url,
    pub logs: Option<String>,
    pub traceback: Option<String>,
    pub run_time: Option<f64>,
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

    fn fake_get_data() -> Vec<GetData> {
        vec![
            GetData {
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
            GetData {
                url: "http://thema.ai".parse().unwrap(),
                request_url: "http://thema.ai".parse().unwrap(),
                status_code: 200,
                data: Some(Bytes::new()),
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
}
