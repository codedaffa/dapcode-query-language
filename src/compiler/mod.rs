//! DQL Target Compiler and SQL Code Generator

pub mod postgres;
pub mod sqlite;
pub mod mysql;

use crate::ast::*;
use crate::lexer::Lexer;
use crate::parser::Parser;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dialect {
    Postgres,
    Sqlite,
    Mysql,
}

impl Dialect {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "postgres" | "postgresql" | "pg" => Some(Dialect::Postgres),
            "sqlite" | "sqlite3" => Some(Dialect::Sqlite),
            "mysql" | "mariadb" => Some(Dialect::Mysql),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompileResult {
    pub sql: String,
    pub params: Vec<LiteralValue>,
}

pub fn compile(source: &str, dialect: Dialect, parameterized: bool) -> Result<CompileResult, String> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().map_err(|e| e.to_string())?;

    let mut parser = Parser::new(tokens);
    let stmt = parser.parse_statement().map_err(|e| e.to_string())?;

    match dialect {
        Dialect::Postgres => postgres::PostgresCompiler::new(parameterized).compile(&stmt),
        Dialect::Sqlite => sqlite::SqliteCompiler::new(parameterized).compile(&stmt),
        Dialect::Mysql => mysql::MysqlCompiler::new(parameterized).compile(&stmt),
    }
}
