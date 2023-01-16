use crate::{Dialect, ToSql};
use crate::util::{push_sql_sequence, quote, table_name};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum OnConflict {
    Ignore,
    Abort,
    // Replace,
}

impl Default for OnConflict {
    fn default() -> Self {
        OnConflict::Abort
    }
}

pub struct Values(pub Vec<String>);

impl ToSql for Values {
    fn to_sql(&self, _dialect: Dialect) -> String {
        let mut sql = String::new();
        sql.push('(');
        push_sql_sequence(&mut sql, &self.0, ", ", Dialect::Postgres);
        sql.push(')');
        sql
    }
}

pub struct Insert {
    pub schema: Option<String>,
    pub table: String,
    pub columns: Vec<String>,
    pub values: Vec<Values>,
    pub on_conflict: OnConflict,
    pub returning: Vec<String>,
}

fn query_start(dialect: Dialect, on_conflict: OnConflict) -> String {
    use Dialect::*;
    use OnConflict::*;
    if dialect == Sqlite {
        match on_conflict {
            Ignore => "INSERT OR IGNORE INTO ".to_string(),
            Abort => "INSERT OR ABORT INTO ".to_string(),
        }
    } else {
        "INSERT INTO ".to_string()
    }
}

impl ToSql for Insert {
    fn to_sql(&self, dialect: Dialect) -> String {
        let mut q = query_start(dialect, self.on_conflict);
        q.push_str(&table_name(self.schema.as_ref(), &self.table, None));
        q.push_str(" (");
        push_sql_sequence(&mut q, &self.columns, ", ", dialect);
        q.push_str(") VALUES ");
        push_sql_sequence(&mut q, &self.values, ", ", dialect);

        if !self.returning.is_empty() {
            q.push_str(" RETURNING ");
            let mut first = true;
            for column in &self.returning {
                if !first {
                    q.push_str(", ");
                }
                q.push_str(quote(column).as_str());
                first = false;
            }
        }
        q
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let insert = Insert {
            schema: None,
            table: "foo".to_string(),
            columns: vec!["bar".to_string(), "baz".to_string()],
            values: vec![
                Values(vec!["1".to_string(), "2".to_string()]),
                Values(vec!["3".to_string(), "4".to_string()]),
            ],
            on_conflict: OnConflict::Abort,
            returning: vec!["id".to_string()],
        };
        assert_eq!(1, 0);
    }
}