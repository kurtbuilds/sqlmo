use crate::{Dialect, ToSql};
use crate::util::SqlExtension;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexType {
    BTree,
    Hash,
    Gist,
    SpGist,
    Brin,
}

impl Default for IndexType {
    fn default() -> Self {
        IndexType::BTree
    }
}

/// Create index action for a table
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateIndex {
    pub name: String,
    pub unique: bool,
    pub schema: Option<String>,
    pub table: String,
    pub columns: Vec<String>,
    pub type_: IndexType,
}

impl ToSql for CreateIndex {
    fn write_sql(&self, buf: &mut String, _: Dialect) {
        buf.push_str("CREATE ");
        if self.unique {
            buf.push_str("UNIQUE ");
        }
        buf.push_quoted(&self.name);
        buf.push_str(" ON ");
        buf.push_table_name(&self.schema, &self.table);
        buf.push_str(" USING ");
        match self.type_ {
            IndexType::BTree => buf.push_str("BTREE"),
            IndexType::Hash => buf.push_str("HASH"),
            IndexType::Gist => buf.push_str("GIST"),
            IndexType::SpGist => buf.push_str("SPGIST"),
            IndexType::Brin => buf.push_str("BRIN"),
        }
        buf.push_str(" (");
        buf.push_quoted_sequence(&self.columns, ", ");
        buf.push(')');
    }
}
