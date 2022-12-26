#![allow(unused)]

//
// Given a OpenAPI spec, a connection to a database, and the schema within that database,
// - build a diff
// - Execute SQL

mod schema;
mod migrate;

pub use migrate::*;
pub use schema::*;