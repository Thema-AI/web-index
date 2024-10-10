//! Do the actual IO: reading, writing and scanning files to S3.
use polars::prelude::*;
use crate::{path::*, query::{DeterministicQuery, Query}};

pub(crate) struct LogicalFile {
    df: DataFrame,
    path: LogicalPath,
}

impl LogicalFile {
    pub(crate) fn new(df: DataFrame, path: LogicalPath) -> LogicalFile {
        Self { df, path }
    }
}

struct PhysicalFile {
    df: DataFrame,
    path: PhysicalPath,
}

impl From<LogicalFile> for PhysicalFile {
    fn from(value: LogicalFile) -> Self {
        Self {
            df: value.df,
            path: PhysicalPath::new_default(value.path),
        }
    }
}

/// This fn takes care of all the IO:
/// - serialise files to parquet
/// - calculate checksum
/// - upload parquet
pub(crate) fn upload(files: Vec<LogicalFile>) -> anyhow::Result<()> {
    unimplemented!()
}

