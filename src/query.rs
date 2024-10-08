use anyhow::Result;
use chrono::{DateTime, Datelike, Utc};
use url::Url;
use uuid::Uuid;

use crate::domain::Extractor;

pub enum DataType {
    Head,
    Get,
}

pub enum MetadataType {
    HeadMetadata,
    GetMetadata,
}

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

    /// Compute the full path for this insertion request, using a previously
    /// instantised extractor. This method should be used for batch operations,
    /// to avoid repeatedly building extractors.
    pub fn compute_path(&self, extractor: &mut Extractor) -> Result<String> {
        let uuid = Uuid::new_v4();
        let dir = self.dir();
        let domain = extractor.domain(&self.url)?;

        Ok(format!("{dir}/{domain}.{uuid}.parquet"))
    }

    /// Compute the full path for this insertion request. This method builds a
    /// new extractor every time; prefer `InsertionQuery::compute_path` with a
    /// seperate extractor for batch operations.
    pub fn path(&self) -> Result<String> {
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

pub struct DeterministicQuery {
    pub r#type: RecordType,
    pub url: Url,
    pub timestamp: DateTime<Utc>,
    pub request_id: String,
}

pub struct SimpleQuery {
    pub r#type: RecordType,
    pub url: Url,
    pub calibre: u8,
    pub calibre_strict: bool,
}

pub struct TimeBoundedQuery {
    pub r#type: RecordType,
    pub url: Url,
    pub not_before: DateTime<Utc>,
    pub not_after: DateTime<Utc>,
    pub calibre: u8,
    pub calibre_strict: bool,
}

pub enum Query {
    Deterministic(DeterministicQuery),
    Simple(SimpleQuery),
    TimeBounded(TimeBoundedQuery),
}

#[cfg(test)]
mod test_insertion_query {

    use super::*;
    use anyhow::Result;
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
    fn path_contains_uuid() -> Result<()> {
        let query =
            InsertionQuery::get("https://thema.ai".parse()?, "2024-01-01T12:13:14Z".parse()?);
        let path = query.path()?;
        let parts: Vec<_> = path.split(".").collect();
        let uuid = parts[parts.len() - 2];
        println!("{uuid}");
        Uuid::parse_str(&uuid)?;

        Ok(())
    }

    #[test]
    fn path_composed_of_dir_uuid_and_suffix() -> Result<()> {
        let query =
            InsertionQuery::get("https://thema.ai".parse()?, "2024-01-01T12:13:14Z".parse()?);
        let path = query.path()?;
        let parts: Vec<_> = path.split(".").collect();
        let uuid = parts[parts.len() - 2];
        let known_path = path.replace(uuid, "UUID");

        assert_eq!(known_path, "get/2024/01/thema.ai.UUID.parquet");
        Ok(())
    }
}
