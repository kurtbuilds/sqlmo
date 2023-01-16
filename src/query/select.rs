use crate::{Dialect, ToSql};
use crate::util::SqlExtension;

/// Common table expression
#[derive(Debug, Clone)]
pub struct Cte {
    pub name: String,
    pub query: Select,
}

impl ToSql for Cte {
    fn write_sql(&self, buf: &mut String, dialect: Dialect) {
        buf.push_str(&self.name);
        buf.push_str(" AS (");
        buf.push_str(&self.query.to_sql(dialect));
        buf.push(')');
    }
}

/// Represents a select column value.
#[derive(Debug, Clone)]
pub enum SelectExpression {
    Column { schema: Option<String>, table: Option<String>, column: String },
    Literal(String),
}


/// Represents a column of a SELECT statement.
#[derive(Debug, Clone)]
pub struct SelectColumn {
    pub expression: SelectExpression,
    pub alias: Option<String>,
}

impl ToSql for SelectColumn {
    fn write_sql(&self, buf: &mut String, _: Dialect) {
        use SelectExpression::*;
        match &self.expression {
            Column { schema, table, column } => {
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
            Literal(literal) => {
                buf.push_str(literal);
            }
        }
        if let Some(alias) = &self.alias {
            buf.push_str(" AS ");
            buf.push_quoted(alias);
        }
    }
}


#[derive(Debug, Clone)]
pub struct From {
    pub schema: Option<String>,
    pub table: String,
    pub alias: Option<String>,
}

impl ToSql for From {
    fn write_sql(&self, buf: &mut String, _: Dialect) {
        buf.push_table_name(&self.schema, &self.table, self.alias.as_ref());
    }
}

#[derive(Debug, Clone)]
pub enum Where {
    And(Vec<Where>),
    Or(Vec<Where>),
    Literal(String),
}

impl Where {
    pub fn is_empty(&self) -> bool {
        use Where::*;
        match self {
            And(v) => v.is_empty(),
            Or(v) => v.is_empty(),
            Literal(s) => s.is_empty(),
        }
    }
}

impl ToSql for Where {
    fn write_sql(&self, buf: &mut String, dialect: Dialect) {
        match self {
            Where::And(v) => {
                buf.push_sql_sequence(v, " AND ", dialect);
            }
            Where::Or(v) => {
                buf.push('(');
                buf.push_sql_sequence(v, " OR ", dialect);
                buf.push(')');
            }
            Where::Literal(s) => {
                buf.push_str(s);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum JoinTable {
    Select(Select),
    Table(From),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
}

#[derive(Debug, Clone)]
pub struct Join {
    pub typ: JoinType,
    pub table: JoinTable,
    pub alias: Option<String>,
    pub on: Where,
}


impl ToSql for Join {
    fn write_sql(&self, buf: &mut String, dialect: Dialect) {
        use JoinType::*;
        use JoinTable::*;
        match self.typ {
            Inner => buf.push_str("INNER JOIN "),
            Left => buf.push_str("LEFT JOIN "),
            Right => buf.push_str("RIGHT JOIN "),
            Full => buf.push_str("FULL JOIN "),
        }
        match &self.table {
            Select(s) => {
                buf.push('(');
                buf.push_str(&s.to_sql(dialect));
                buf.push(')');
            }
            Table(f) => {
                buf.push_str(&f.to_sql(dialect));
            }
        }
        if let Some(alias) = &self.alias {
            buf.push_str(" AS ");
            buf.push_str(alias);
        }
        buf.push_str(" ON ");
        buf.push_str(&self.on.to_sql(dialect));
    }
}

/// The direction of a column in an ORDER BY clause.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Asc,
    Desc,
}

#[derive(Debug, Clone)]
pub struct OrderBy {
    pub column: String,
    pub direction: Direction,
}

impl ToSql for OrderBy {
    fn write_sql(&self, buf: &mut String, _: Dialect) {
        use Direction::*;
        buf.push_str(&self.column);
        match self.direction {
            Asc => buf.push_str(" ASC"),
            Desc => buf.push_str(" DESC"),
        }
    }
}


impl OrderBy {
    pub fn asc() -> Direction {
        Direction::Asc
    }
    pub fn desc() -> Direction {
        Direction::Desc
    }
}


#[derive(Debug, Clone)]
pub struct GroupBy(String);

impl ToSql for GroupBy {
    fn write_sql(&self, buf: &mut String, _: Dialect) {
        buf.push_str(&self.0)
    }
}

/// A SELECT query.
#[derive(Debug, Clone)]
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

impl Select {
    pub fn new() -> Self {
        Select {
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

    pub fn cte(mut self, name: &str, query: Select) -> Self {
        self.ctes.push(Cte {
            name: name.to_string(),
            query,
        });
        self
    }

    pub fn distinct(mut self) -> Self {
        self.distinct = true;
        self
    }

    pub fn select(mut self, expression: SelectExpression, alias: Option<&str>) -> Self {
        self.columns.push(SelectColumn {
            expression,
            alias: alias.map(|s| s.to_string()),
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

    /// Assumes `AND`. Access the `.where_` field directly for more advanced operations.
    pub fn where_(mut self, where_: Where) -> Self {
        match self.where_ {
            Where::And(ref mut v) => v.push(where_),
            _ => self.where_ = Where::And(vec![self.where_, where_]),
        }
        self
    }

    pub fn group(mut self, group: &str) -> Self {
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

    pub fn order(mut self, order: &str, direction: Direction) -> Self {
        self.order.push(OrderBy {
            column: order.to_string(),
            direction,
        });
        self
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