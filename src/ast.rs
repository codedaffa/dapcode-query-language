//! AST (Abstract Syntax Tree) for DQL

#[derive(Debug, Clone, PartialEq)]
pub enum DqlStatement {
    Ambil(AmbilQuery),
    Tambah(TambahQuery),
    Perbarui(PerbaruiQuery),
    Hapus(HapusQuery),
    BuatTabel(BuatTabelQuery),
    HapusTabel(HapusTabelQuery),
    BuatDatabase(BuatDatabaseQuery),
    HapusDatabase(HapusDatabaseQuery),
    TampilkanTabel,
    TampilkanDatabase,
    DeskripsiTabel(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableRef {
    pub name: String,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColumnRef {
    pub table: Option<String>,
    pub column: String,
}

impl ColumnRef {
    pub fn simple(name: impl Into<String>) -> Self {
        Self {
            table: None,
            column: name.into(),
        }
    }

    pub fn qualified(table: impl Into<String>, col: impl Into<String>) -> Self {
        Self {
            table: Some(table.into()),
            column: col.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SelectItem {
    All,
    Expr {
        expr: Expr,
        alias: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum JoinType {
    Inner,
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq)]
pub struct JoinClause {
    pub join_type: JoinType,
    pub table: TableRef,
    pub condition: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OrderDirection {
    Asc,
    Desc,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrderByClause {
    pub expr: Expr,
    pub direction: OrderDirection,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AmbilQuery {
    pub table: TableRef,
    pub fields: Vec<SelectItem>,
    pub joins: Vec<JoinClause>,
    pub filter: Option<Expr>,
    pub group_by: Vec<Expr>,
    pub having: Option<Expr>,
    pub order_by: Vec<OrderByClause>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecordAssignment {
    pub field: String,
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TambahQuery {
    pub table: TableRef,
    pub records: Vec<Vec<RecordAssignment>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PerbaruiQuery {
    pub table: TableRef,
    pub assignments: Vec<RecordAssignment>,
    pub filter: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HapusQuery {
    pub table: TableRef,
    pub filter: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ColumnType {
    Serial,
    Text,
    Integer,
    Float,
    Boolean,
    Timestamp,
    Json,
    Blob,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ColumnConstraints {
    pub is_primary_key: bool,
    pub is_not_null: bool,
    pub is_unique: bool,
    pub is_auto_increment: bool,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColumnDefinition {
    pub name: String,
    pub col_type: ColumnType,
    pub constraints: ColumnConstraints,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BuatTabelQuery {
    pub table_name: String,
    pub if_not_exists: bool,
    pub columns: Vec<ColumnDefinition>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HapusTabelQuery {
    pub table_name: String,
    pub if_exists: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BuatDatabaseQuery {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HapusDatabaseQuery {
    pub name: String,
    pub if_exists: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    // Comparison
    Eq,
    Neq,
    Lt,
    Lte,
    Gt,
    Gte,
    // Logical
    And,
    Or,
    // Pattern Matching
    Like,
    Contains,  // LIKE '%v%'
    StartsWith,// LIKE 'v%'
    EndsWith,  // LIKE '%v'
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Not,
    Neg,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(LiteralValue),
    Column(ColumnRef),
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    FunctionCall {
        name: String,
        args: Vec<Expr>,
    },
    InList {
        expr: Box<Expr>,
        list: Vec<Expr>,
        negated: bool,
    },
    Between {
        expr: Box<Expr>,
        low: Box<Expr>,
        high: Box<Expr>,
        negated: bool,
    },
}
