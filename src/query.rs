use std::{collections::HashMap, str::FromStr};

use anyhow::{Context, Result};
use chrono::{DateTime, Datelike, Utc};
use url::Url;

use crate::{domain::Extractor, path::LogicalPath};

pub enum DataType {
    Head,
    Get,
}

pub enum MetadataType {
    HeadMetadata,
    GetMetadata,
}

#[derive(PartialEq, Debug)]
pub enum RecordType {
    Head,
    Get,
    HeadMetadata,
    GetMetadata,
}

impl RecordType {
    fn dir(&self) -> &str {
        match self {
            RecordType::Head => "head",
            RecordType::Get => "get",
            RecordType::HeadMetadata => "head-metadata",
            RecordType::GetMetadata => "get-metadata",
        }
    }
}

impl FromStr for RecordType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "head" => Ok(RecordType::Head),
            "get" => Ok(RecordType::Get),
            "head-metadata" => Ok(RecordType::HeadMetadata),
            "get-metadata" => Ok(RecordType::GetMetadata),
            _ => Err(Self::Err::msg("Failed parsing request id")),
        }
    }
}

impl From<DataType> for RecordType {
    fn from(value: DataType) -> Self {
        match value {
            DataType::Head => Self::Head,
            DataType::Get => Self::Get,
        }
    }
}

impl From<MetadataType> for RecordType {
    fn from(value: MetadataType) -> Self {
        match value {
            MetadataType::HeadMetadata => Self::HeadMetadata,
            MetadataType::GetMetadata => Self::GetMetadata,
        }
    }
}

/// Attempted conversion with an invalid record type.
pub struct InvalidRecordType;

impl TryFrom<RecordType> for DataType {
    type Error = InvalidRecordType;

    fn try_from(value: RecordType) -> Result<Self, Self::Error> {
        match value {
            RecordType::Head => Ok(Self::Head),
            RecordType::Get => Ok(Self::Get),
            _ => Err(InvalidRecordType),
        }
    }
}

impl TryFrom<RecordType> for MetadataType {
    type Error = InvalidRecordType;

    fn try_from(value: RecordType) -> Result<Self, Self::Error> {
        match value {
            RecordType::HeadMetadata => Ok(Self::HeadMetadata),
            RecordType::GetMetadata => Ok(Self::GetMetadata),
            _ => Err(InvalidRecordType),
        }
    }
}

pub struct InsertionQuery {
    pub record_type: RecordType,
    pub url: Url,
    pub timestamp: DateTime<Utc>,
}

impl InsertionQuery {
    /// Compute the directory for this insertion request.
    pub fn dir(&self) -> String {
        format!(
            "{}/{}/{:02}",
            self.record_type.dir(),
            self.timestamp.year(),
            self.timestamp.month()
        )
    }

    /// Compute the logical path for this insertion request, using a previously
    /// instantised extractor. This method should be used for batch operations,
    /// to avoid repeatedly building extractors.
    pub fn compute_path(self, extractor: &mut Extractor) -> Result<LogicalPath> {
        let dir = self.dir();
        let domain = extractor.domain(&self.url)?;
        Ok(LogicalPath::new(dir, domain.into(), "parquet".into()))
    }

    /// Compute the logical path for this insertion request. This method builds a
    /// new extractor every time; prefer `InsertionQuery::compute_path` with a
    /// seperate extractor for batch operations.
    pub fn path(self) -> Result<LogicalPath> {
        let mut extractor = Extractor::new();
        self.compute_path(&mut extractor)
    }

    pub fn new(record_type: RecordType, url: Url, timestamp: DateTime<Utc>) -> Self {
        Self {
            record_type,
            url,
            timestamp,
        }
    }
    pub fn get(url: Url, timestamp: DateTime<Utc>) -> Self {
        Self::new(RecordType::Get, url, timestamp)
    }
    pub fn head(url: Url, timestamp: DateTime<Utc>) -> Self {
        Self::new(RecordType::Head, url, timestamp)
    }
    pub fn get_metadata(url: Url, timestamp: DateTime<Utc>) -> Self {
        Self::new(RecordType::GetMetadata, url, timestamp)
    }
    pub fn head_metadata(url: Url, timestamp: DateTime<Utc>) -> Self {
        Self::new(RecordType::HeadMetadata, url, timestamp)
    }
}

#[derive(PartialEq, Debug)]
pub struct DeterministicQuery {
    pub record_type: RecordType,
    pub url: Url,
    pub timestamp: DateTime<Utc>,
    pub request_id: String,
}

impl FromStr for DeterministicQuery {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let url = Url::parse(s)?;
        let params: HashMap<_, _> = url.query_pairs().collect();
        if url.scheme() == "thema" {
            Ok(DeterministicQuery {
                record_type: url.path()[1..].parse()?,
                url: params.get("url").context("url")?.parse()?,
                timestamp: params.get("timestamp").context("timestamp")?.parse()?,
                request_id: params.get("request_id").context("request_id")?.parse()?,
            })
        } else {
            None.context("wrong scheme to parse as deterministic query")
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct SimpleQuery {
    pub record_type: RecordType,
    pub url: Url,
    pub calibre: u8,
    pub calibre_strict: bool,
}

impl FromStr for SimpleQuery {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let url = Url::parse(s)?;
        let params: HashMap<_, _> = url.query_pairs().collect();
        if url.scheme() == "thema" {
            Ok(SimpleQuery {
                record_type: url.path()[1..].parse()?,
                url: params.get("url").context("url")?.parse()?,
                calibre: params.get("calibre").context("calibre")?.parse()?,
                calibre_strict: params
                    .get("calibre_strict")
                    .context("calibre_strict")?
                    .parse()?,
            })
        } else {
            None.context("wrong scheme to parse as deterministic query")
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct TimeBoundedQuery {
    pub record_type: RecordType,
    pub url: Url,
    pub not_before: DateTime<Utc>,
    pub not_after: DateTime<Utc>,
    pub calibre: u8,
    pub calibre_strict: bool,
}

impl FromStr for TimeBoundedQuery {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let url = Url::parse(s)?;
        let params: HashMap<_, _> = url.query_pairs().collect();
        if url.scheme() == "thema" {
            Ok(Self {
                record_type: url.path()[1..].parse()?,
                url: params.get("url").context("url")?.parse()?,
                not_before: params.get("not_before").context("not_before")?.parse()?,
                not_after: params.get("not_after").context("not_after")?.parse()?,
                calibre: params.get("calibre").context("calibre")?.parse()?,
                calibre_strict: params
                    .get("calibre_strict")
                    .context("calibre_strict")?
                    .parse()?,
            })
        } else {
            None.context("wrong scheme to parse as deterministic query")
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum Query {
    Deterministic(DeterministicQuery),
    Simple(SimpleQuery),
    TimeBounded(TimeBoundedQuery),
}

impl FromStr for Query {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if let Ok(query) = s.parse::<DeterministicQuery>() {
            Ok(Query::Deterministic(query))
        } else if let Ok(query) = s.parse::<TimeBoundedQuery>() {
            Ok(Query::TimeBounded(query))
        } else if let Ok(query) = s.parse::<SimpleQuery>() {
            Ok(Query::Simple(query))
        } else {
            None.context("Unable to parse query")
        }
    }
}

#[cfg(test)]
mod test_insertion_query {

    use super::*;
    use anyhow::Result;
    use pretty_assertions::assert_eq;
    use rstest::*;

    #[rstest]
    #[case::head(InsertionQuery::head("https://thema.ai/foobar".parse().unwrap(), "2024-01-01T12:13:14Z".parse().unwrap()), "head/2024/01")]
    #[case::head_metadata(InsertionQuery::head_metadata("https://thema.ai/foobar".parse().unwrap(), "2024-01-01T12:13:14Z".parse().unwrap()), "head-metadata/2024/01")]
    #[case::get(InsertionQuery::get("https://thema.ai/foobar".parse().unwrap(), "2024-01-01T12:13:14Z".parse().unwrap()), "get/2024/01")]
    #[case::get_metadata(InsertionQuery::get_metadata("https://thema.ai/foobar".parse().unwrap(), "2024-01-01T12:13:14Z".parse().unwrap()), "get-metadata/2024/01")]
    fn dir_constructed_correctly(#[case] request: InsertionQuery, #[case] dir: String) {
        assert_eq!(request.dir(), dir)
    }

    #[test]
    fn path_composed_of_dir_and_suffix() -> Result<()> {
        let query =
            InsertionQuery::get("https://thema.ai".parse()?, "2024-01-01T12:13:14Z".parse()?);
        let path = query.path()?;

        assert_eq!(path.to_string(), "get/2024/01/thema.ai.parquet".to_string());
        Ok(())
    }

    #[test]
    fn deterministic_query_parsed_from_uri() -> Result<()> {
        let query: DeterministicQuery = "thema://web-index/get?url=https%3A%2F%2Fthema.ai%2F&timestamp=2024-01-02T12%3A13%3A14Z&request_id=ID".parse()?;

        assert_eq!(
            query,
            DeterministicQuery {
                record_type: RecordType::Get,
                url: "https://thema.ai/".parse()?,
                timestamp: "2024-01-02T12:13:14Z".parse()?,
                request_id: "ID".to_string(),
            }
        );
        Ok(())
    }

    // TODO data-driven tests for the rest, prob with yaml: this is a pain
}
