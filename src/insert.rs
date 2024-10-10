use std::collections::HashMap;

use polars::frame::DataFrame;

use crate::data::{self, GetResponse, HeadResponse, Metadata, PersistedData, ToFromDf};
use crate::io::LogicalFile;
use crate::path::{LogicalPath, PhysicalPath};
use crate::query::{DeterministicQuery, InsertionQuery};

pub struct InsertionRequest<T> {
    pub query: InsertionQuery,
    pub data: Vec<T>,
}

struct PreparedRecord<T: ToFromDf> {
    data: Vec<T>,
    path: LogicalPath,
    retrieval: DeterministicQuery,
}

pub struct InsertionRequests<T> {
    requests: Vec<InsertionRequest<T>>,
}

impl<T: data::ToFromDf> InsertionRequests<T> {
    pub fn new(requests: Vec<InsertionRequest<T>>) -> Self {
        Self { requests }
    }

    fn prepare(self) -> anyhow::Result<Vec<PreparedRecord<T>>> {
        unimplemented!()
    //     let mut files = HashMap::new();
    //     for request in self.requests.into_iter() {
    //         let entry = files.entry(request.path).or_insert(Vec::new());
    //         entry.extend(request.data);
    //     }
    //     files
    //         .into_iter()
    //         .map(|(path, data)| {
    //             let df = T::to_df(&data)?;
    //             Ok(LogicalFile { df, path })
    //         })
    //         .collect()
    }
}

/// Group prpared records by logical path and convert to dataframes ready for persistence
fn prepared_to_files<T: ToFromDf>(prepared: &[PreparedRecord<T>]) -> Vec<LogicalFile> {
    unimplemented!()
}



pub fn persist(requests: Vec<InsertionQuery>) -> Vec<DeterministicQuery> {
    // let logical_files = F
    unimplemented!()
}
