use crate::util::SqlExtension;
use crate::{Dialect, Table, ToSql};

/// Create table action
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DropTable {
    pub schema: Option<String>,
    pub name: String,
}

impl DropTable {
    pub fn from_table(table: &Table) -> DropTable {
        DropTable {
            schema: table.schema.clone(),
            name: table.name.clone(),
        }
    }
}

impl ToSql for DropTable {
    fn write_sql(&self, buf: &mut String, _dialect: Dialect) {
        buf.push_str("DROP TABLE ");
        buf.push_table_name(&self.schema, &self.name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drop_table() {
        let t = Table::new("test_table");
        let dt = DropTable::from_table(&t);
        let sql = dt.to_sql(Dialect::Postgres).to_string();
        assert_eq!(sql, r#"DROP TABLE "test_table""#);
    }
}
