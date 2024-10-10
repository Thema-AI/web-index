//! # Web Index
//!
//! The web index store data retrieved from the web, metadata about the
//! retrieval, and data computed from this data. It is optimised for massive
//! scale and random access. In effect it is a kind of append-only database
//! supporting various versions of `SELECT` and `INSERT`.
//!
//! The index is specified formally in the standard, present in the
//! repository. This crate provides a low-level abstraction over the index,
//! suitable for use in binary crates, and as the basis for the high-level
//! abstraction provided in python.

/// Data stored in the Web Index
pub mod data;
/// Domain extraction; used to compute paths
pub mod domain;
/// Insertion of records
pub mod insert;
/// Path resolution
pub mod path;
/// Queries are used to retrieve and insert data
pub mod query;
mod io;
/// Functions to retrieve data
pub mod retrieve;
