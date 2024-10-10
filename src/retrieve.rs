use crate::query::{DeterministicQuery, Query};

// TODO think this should just be generic
pub(crate) fn exists(queries: &[Query]) -> anyhow::Result<Vec<Option<bool>>> {
    unimplemented!()
}

pub struct Persisted<T> {
    data: T,
    query: DeterministicQuery,
}

pub(crate) fn download<T>(queries: &[Query]) -> anyhow::Result<Vec<Option<Persisted<T>>>> {
    unimplemented!()
}
