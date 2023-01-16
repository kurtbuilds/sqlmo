mod from_postgres;
#[cfg(feature = "openapi")]
mod from_openapi;
mod table;
mod r#type;
mod column;
mod index;


pub use column::TableColumn;
pub use r#type::Type;
pub use table::Table;

use anyhow::Result;
use crate::migrate::{Migration, migrate, MigrationOptions};

#[derive(Debug)]
pub struct Schema {
    pub tables: Vec<Table>,
}

impl Schema {
    pub fn new() -> Schema {
        Schema {
            tables: vec![],
        }
    }

    pub fn migrate_to(self, desired: Schema, options: &MigrationOptions) -> Result<Migration> {
        migrate(self, desired, options)
    }

    pub fn name_schema(&mut self, schema: &str) {
        for table in &mut self.tables {
            table.schema = Some(schema.to_string());
        }
    }
}