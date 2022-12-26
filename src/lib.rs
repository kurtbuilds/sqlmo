#![allow(unused)]

//
// Given a OpenAPI spec, a connection to a database, and the schema within that database,
// - build a diff
// - Execute SQL

mod schema;
mod migrate;

use std::collections::{HashMap, HashSet};
use anyhow::{Error, Result};
use schema::{Table, Column, SchemaColumn};
use itertools::Itertools;
use sqlx::PgConnection;
use migrate::Migration;
pub use schema::Schema;