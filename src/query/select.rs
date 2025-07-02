use crate::query::{Cte, CteQuery};
use crate::util::SqlExtension;
use crate::{Dialect, ToSql};

mod expr;
mod join;

pub use expr::*;
pub use join::*;

/// A SELECT query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Select {
    pub ctes: Vec<Cte>,
    pub distinct: bool,
    pub columns: Vec<SelectColumn>,
    pub from: Option<From>,
    pub join: Vec<Join>,
    pub where_: Where,
    pub group: Vec<GroupBy>,
    pub having: Where,
    pub order: Vec<OrderBy>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl Default for Select {
    fn default() -> Self {
        Self {
            ctes: vec![],
            distinct: false,
            columns: vec![],
            from: None,
            join: vec![],
            where_: Where::And(vec![]),
            group: vec![],
            having: Where::And(vec![]),
            order: vec![],
            limit: None,
            offset: None,
        }
    }
}

impl Select {
    pub fn with_raw(mut self, name: &str, query: &str) -> Self {
        self.ctes.push(Cte {
            name: name.to_string(),
            query: CteQuery::Raw(query.to_string()),
        });
        self
    }

    pub fn with(mut self, name: &str, query: Select) -> Self {
        self.ctes.push(Cte {
            name: name.to_string(),
            query: CteQuery::Select(query),
        });
        self
    }

    pub fn distinct(mut self) -> Self {
        self.distinct = true;
        self
    }

    pub fn table_column(mut self, table: &str, column: &str) -> Self {
        self.columns.push(SelectColumn::table_column(table, column));
        self
    }

    pub fn select_raw(mut self, expression: impl Into<String>) -> Self {
        self.columns.push(SelectColumn {
            expression: SelectExpression::Raw(expression.into()),
            alias: None,
        });
        self
    }

    pub fn from(mut self, table: &str) -> Self {
        self.from = Some(From {
            schema: None,
            table: table.to_string(),
            alias: None,
        });
        self
    }

    pub fn join(mut self, join: Join) -> Self {
        self.join.push(join);
        self
    }

    /// Assumes `AND`. Access the `.where_` field directly for more advanced operations.
    pub fn where_(mut self, where_: Where) -> Self {
        match self.where_ {
            Where::And(ref mut v) => v.push(where_),
            _ => self.where_ = Where::And(vec![self.where_, where_]),
        }
        self
    }

    pub fn where_raw(self, where_: impl Into<String>) -> Self {
        self.where_(Where::raw(where_))
    }

    pub fn group_by(mut self, group: &str) -> Self {
        self.group.push(GroupBy(group.to_string()));
        self
    }

    pub fn having(mut self, having: Where) -> Self {
        match self.having {
            Where::And(ref mut v) => v.push(having),
            _ => self.having = Where::And(vec![self.having, having]),
        }
        self
    }

    pub fn order_by(mut self, order: OrderBy) -> Self {
        self.order.push(order);
        self
    }

    pub fn order_asc(self, order: &str) -> Self {
        self.order_by(OrderBy::new(order).asc())
    }

    pub fn order_desc(self, order: &str) -> Self {
        self.order_by(OrderBy::new(order).desc())
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }
}

/// Represents a select column value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectExpression {
    Column {
        schema: Option<String>,
        table: Option<String>,
        column: String,
    },
    Raw(String),
}

/// Represents a column of a SELECT statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectColumn {
    pub expression: SelectExpression,
    pub alias: Option<String>,
}

impl SelectColumn {
    pub fn column(&self) -> Option<&str> {
        match &self.expression {
            SelectExpression::Column { column, .. } => Some(column),
            _ => None,
        }
    }

    pub fn new(column: &str) -> Self {
        Self {
            expression: SelectExpression::Column {
                schema: None,
                table: None,
                column: column.to_string(),
            },
            alias: None,
        }
    }

    pub fn table_column(table: &str, column: &str) -> Self {
        Self {
            expression: SelectExpression::Column {
                schema: None,
                table: Some(table.to_string()),
                column: column.to_string(),
            },
            alias: None,
        }
    }

    pub fn raw(expression: &str) -> Self {
        Self {
            expression: SelectExpression::Raw(expression.to_string()),
            alias: None,
        }
    }

    pub fn alias(mut self, alias: impl Into<String>) -> Self {
        self.alias = Some(alias.into());
        self
    }
}

impl<T: Into<String>> std::convert::From<T> for SelectColumn {
    fn from(column: T) -> Self {
        Self {
            expression: SelectExpression::Column {
                schema: None,
                table: None,
                column: column.into(),
            },
            alias: None,
        }
    }
}

impl ToSql for SelectColumn {
    fn write_sql(&self, buf: &mut String, _: Dialect) {
        use SelectExpression::*;
        match &self.expression {
            Column {
                schema,
                table,
                column,
            } => {
                if let Some(schema) = schema {
                    buf.push_quoted(schema);
                    buf.push('.');
                }
                if let Some(table) = table {
                    buf.push_quoted(table);
                    buf.push('.');
                }
                buf.push_quoted(column);
            }
            Raw(raw) => {
                buf.push_str(raw);
            }
        }
        if let Some(alias) = &self.alias {
            buf.push_str(" AS ");
            buf.push_quoted(alias);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct From {
    pub schema: Option<String>,
    pub table: String,
    pub alias: Option<String>,
}

impl<T: Into<String>> std::convert::From<T> for From {
    fn from(table: T) -> Self {
        Self {
            schema: None,
            table: table.into(),
            alias: None,
        }
    }
}

impl ToSql for From {
    fn write_sql(&self, buf: &mut String, _: Dialect) {
        buf.push_table_name(&self.schema, &self.table);
        if let Some(alias) = &self.alias {
            buf.push_str(" AS ");
            buf.push_quoted(alias);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Where {
    And(Vec<Where>),
    Or(Vec<Where>),
    #[deprecated]
    Raw(String),
    Expr(Expr),
}

impl std::convert::From<String> for Where {
    fn from(s: String) -> Self {
        Where::Expr(Expr::Raw(s))
    }
}

impl Where {
    pub fn is_empty(&self) -> bool {
        use Where::*;
        match self {
            And(v) => v.is_empty(),
            Or(v) => v.is_empty(),
            #[allow(deprecated)]
            Raw(s) => s.is_empty(),
            Expr(_) => false,
        }
    }

    pub fn raw(s: impl Into<String>) -> Self {
        Where::Expr(Expr::Raw(s.into()))
    }
}

impl ToSql for Where {
    fn write_sql(&self, buf: &mut String, dialect: Dialect) {
        match self {
            Where::And(v) => {
                buf.push_sql_sequence(&v, " AND ", dialect);
            }
            Where::Or(v) => {
                buf.push('(');
                buf.push_sql_sequence(&v, " OR ", dialect);
                buf.push(')');
            }
            #[allow(deprecated)]
            Where::Raw(s) => {
                buf.push_str(s);
            }
            Where::Expr(expr) => {
                buf.push_sql(expr, dialect);
            }
        }
    }
}

/// The direction of a column in an ORDER BY clause.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NullsOrder {
    First,
    Last,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderBy {
    pub column: String,
    pub direction: Option<Direction>,
    pub nulls: Option<NullsOrder>,
}

impl OrderBy {
    pub fn new(column: &str) -> Self {
        OrderBy {
            column: column.to_string(),
            direction: None,
            nulls: None,
        }
    }

    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = Some(direction);
        self
    }

    pub fn asc(mut self) -> Self {
        self.direction = Some(Direction::Asc);
        self
    }

    pub fn desc(mut self) -> Self {
        self.direction = Some(Direction::Desc);
        self
    }

    pub fn nulls(mut self, nulls: NullsOrder) -> Self {
        self.nulls = Some(nulls);
        self
    }

    pub fn nulls_first(mut self) -> Self {
        self.nulls = Some(NullsOrder::First);
        self
    }

    pub fn nulls_last(mut self) -> Self {
        self.nulls = Some(NullsOrder::Last);
        self
    }
}

impl ToSql for OrderBy {
    fn write_sql(&self, buf: &mut String, _: Dialect) {
        use Direction::*;
        buf.push_str(&self.column);
        if let Some(direction) = self.direction {
            match direction {
                Asc => buf.push_str(" ASC"),
                Desc => buf.push_str(" DESC"),
            }
        }
        if let Some(nulls) = self.nulls {
            match nulls {
                NullsOrder::First => buf.push_str(" NULLS FIRST"),
                NullsOrder::Last => buf.push_str(" NULLS LAST"),
            }
        }
    }
}

impl Default for Direction {
    fn default() -> Self {
        Direction::Asc
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupBy(String);

impl ToSql for GroupBy {
    fn write_sql(&self, buf: &mut String, _: Dialect) {
        buf.push_str(&self.0)
    }
}

impl ToSql for Select {
    fn write_sql(&self, buf: &mut String, dialect: Dialect) {
        if !self.ctes.is_empty() {
            buf.push_str("WITH ");
            buf.push_sql_sequence(&self.ctes, ", ", dialect);
            buf.push(' ');
        }
        buf.push_str("SELECT ");
        if self.distinct {
            buf.push_str("DISTINCT ");
        }
        buf.push_sql_sequence(&self.columns, ", ", dialect);
        if let Some(from) = &self.from {
            buf.push_str(" FROM ");
            buf.push_str(&from.to_sql(dialect));
            buf.push(' ');
        }
        if !self.join.is_empty() {
            buf.push_sql_sequence(&self.join, " ", dialect);
        }
        if !self.where_.is_empty() {
            buf.push_str(" WHERE ");
            buf.push_str(&self.where_.to_sql(dialect));
        }
        if !self.group.is_empty() {
            buf.push_str(" GROUP BY ");
            buf.push_sql_sequence(&self.group, ", ", dialect);
        }
        if !self.having.is_empty() {
            buf.push_str(" HAVING ");
            buf.push_str(&self.having.to_sql(dialect));
        }
        if !self.order.is_empty() {
            buf.push_str(" ORDER BY ");
            buf.push_sql_sequence(&self.order, ", ", dialect);
        }
        if let Some(limit) = self.limit {
            buf.push_str(" LIMIT ");
            buf.push_str(&limit.to_string());
        }
        if let Some(offset) = self.offset {
            buf.push_str(" OFFSET ");
            buf.push_str(&offset.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let select = Select::default()
            .with_raw("foo", "SELECT 1")
            .with("bar", Select::default().select_raw("1"))
            .select_raw("id")
            .select_raw("name")
            .from("users")
            .join(Join::new("posts").on_raw("users.id = posts.user_id"))
            .where_raw("1=1")
            .order_asc("id")
            .order_desc("name")
            .limit(10)
            .offset(5);
        assert_eq!(
            select.to_sql(Dialect::Postgres),
            r#"WITH foo AS (SELECT 1), bar AS (SELECT 1) SELECT id, name FROM "users" JOIN "posts" ON users.id = posts.user_id WHERE 1=1 ORDER BY id ASC, name DESC LIMIT 10 OFFSET 5"#
        );
    }
}
