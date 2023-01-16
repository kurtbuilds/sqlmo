use crate::{Dialect, ToSql};
use crate::query::Direction::Asc;
use crate::util::{column_name, push_sql_sequence, quote};

#[derive(Debug, Clone)]
pub struct Cte {
    pub name: String,
    pub query: Select,
}

impl ToSql for Cte {
    fn to_sql(&self, dialect: Dialect) -> String {
        let mut sql = String::new();
        sql.push_str(&self.name);
        sql.push_str(" AS (");
        sql.push_str(&self.query.to_sql(dialect));
        sql.push_str(")");
        sql
    }
}

#[derive(Debug, Clone)]
pub enum Expression {
    Column { schema: Option<String>, table: Option<String>, column: String },
    Literal(String),
}


#[derive(Debug, Clone)]
pub struct QueryColumn {
    pub expression: Expression,
    pub alias: Option<String>,
}

impl ToSql for QueryColumn {
    fn to_sql(&self, dialect: Dialect) -> String {
        use Expression::*;
        let mut sql = String::new();
        match &self.expression {
            Column { schema, table, column } => {
                sql.push_str(&column_name(schema.as_ref(), table.as_ref(), column, None));
            }
            Literal(literal) => {
                sql.push_str(literal);
            }
        }
        if let Some(alias) = &self.alias {
            sql.push_str(" AS ");
            sql.push_str(&quote(alias));
        }
        sql
    }
}


#[derive(Debug, Clone)]
pub struct From {
    pub schema: Option<String>,
    pub table: String,
    pub alias: Option<String>,
}

impl ToSql for From {
    fn to_sql(&self, dialect: Dialect) -> String {
        crate::util::table_name(self.schema.as_ref(), &self.table, self.alias.as_ref())
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
        match self {
            Where::And(v) => v.is_empty(),
            Where::Or(v) => v.is_empty(),
            Where::Literal(s) => s.is_empty(),
        }
    }
}


impl ToSql for Where {
    fn to_sql(&self, dialect: Dialect) -> String {
        match self {
            Where::And(v) => {
                let mut sql = String::new();
                push_sql_sequence(&mut sql, v, " AND ", dialect);
                sql
            }
            Where::Or(v) => {
                let mut sql = String::new();
                sql.push_str("(");
                push_sql_sequence(&mut sql, v, " OR ", dialect);
                sql.push_str(")");
                sql
            }
            Where::Literal(s) => s.clone(),
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
    fn to_sql(&self, dialect: Dialect) -> String {
        let mut sql = String::new();
        match self.typ {
            JoinType::Inner => sql.push_str("INNER JOIN "),
            JoinType::Left => sql.push_str("LEFT JOIN "),
            JoinType::Right => sql.push_str("RIGHT JOIN "),
            JoinType::Full => sql.push_str("FULL JOIN "),
        }
        match &self.table {
            JoinTable::Select(s) => {
                sql.push_str("(");
                sql.push_str(&s.to_sql(dialect));
                sql.push_str(")");
            }
            JoinTable::Table(f) => {
                sql.push_str(&f.to_sql(dialect));
            }
        }
        if let Some(alias) = &self.alias {
            sql.push_str(" AS ");
            sql.push_str(&alias);
        }
        sql.push_str(" ON ");
        sql.push_str(&self.on.to_sql(dialect));
        sql
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Asc,
    Desc,
}

#[derive(Debug, Clone)]
pub struct Order {
    pub column: String,
    pub direction: Direction,
}

impl ToSql for Order {
    fn to_sql(&self, dialect: Dialect) -> String {
        let mut sql = String::new();
        sql.push_str(&self.column);
        match self.direction {
            Asc => sql.push_str(" ASC"),
            Desc => sql.push_str(" DESC"),
        }
        sql
    }
}


impl Order {
    pub fn asc() -> Direction {
        Direction::Asc
    }
    pub fn desc() -> Direction {
        Direction::Desc
    }
}


#[derive(Debug, Clone)]
pub struct Group(String);

impl ToSql for Group {
    fn to_sql(&self, dialect: Dialect) -> String {
        self.0.clone()
    }
}

#[derive(Debug, Clone)]
pub struct Select {
    pub ctes: Vec<Cte>,
    pub distinct: bool,
    pub columns: Vec<QueryColumn>,
    pub from: Option<From>,
    pub join: Vec<Join>,
    pub where_: Where,
    pub group: Vec<Group>,
    pub having: Where,
    pub order: Vec<Order>,
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

    pub fn select(mut self, expression: Expression, alias: Option<&str>) -> Self {
        self.columns.push(QueryColumn {
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

    pub fn where_(mut self, where_: Where) -> Self {
        match self.where_ {
            Where::And(ref mut v) => v.push(where_),
            _ => self.where_ = Where::And(vec![self.where_, where_]),
        }
        self
    }

    pub fn group(mut self, group: &str) -> Self {
        self.group.push(Group(group.to_string()));
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
        self.order.push(Order {
            column: order.to_string(),
            direction: direction,
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
    fn to_sql(&self, dialect: Dialect) -> String {
        let mut sql = String::new();
        if !self.ctes.is_empty() {
            sql.push_str("WITH ");
            push_sql_sequence(&mut sql, &self.ctes, ", ", dialect);
            sql.push_str(" ");
        }
        sql.push_str("SELECT ");
        if self.distinct {
            sql.push_str("DISTINCT ");
        }
        push_sql_sequence(&mut sql, &self.columns, ", ", dialect);
        if let Some(from) = &self.from {
            sql.push_str(" FROM ");
            sql.push_str(&from.to_sql(dialect));
            sql.push_str(" ");
        }
        if !self.join.is_empty() {
            push_sql_sequence(&mut sql, &self.join, " ", dialect);
        }
        if !self.where_.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&self.where_.to_sql(dialect));
        }
        if !self.group.is_empty() {
            sql.push_str(" GROUP BY ");
            push_sql_sequence(&mut sql, &self.group, ", ", dialect);
        }
        if !self.having.is_empty() {
            sql.push_str(" HAVING ");
            sql.push_str(&self.having.to_sql(dialect));
        }
        if !self.order.is_empty() {
            sql.push_str(" ORDER BY ");
            push_sql_sequence(&mut sql, &self.order, ", ", dialect);
        }
        if let Some(limit) = self.limit {
            sql.push_str(" LIMIT ");
            sql.push_str(&limit.to_string());
        }
        if let Some(offset) = self.offset {
            sql.push_str(" OFFSET ");
            sql.push_str(&offset.to_string());
        }
        sql
    }
}