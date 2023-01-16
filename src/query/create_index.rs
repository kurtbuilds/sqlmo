use crate::{Dialect, ToSql};

/// Create index action for a table
#[derive(Debug)]
pub struct CreateIndex {
    pub name: String,
    pub table_name: String,
}

impl ToSql for CreateIndex {
    fn to_sql(&self, dialect: Dialect) -> String {
        unimplemented!()
    }
}
