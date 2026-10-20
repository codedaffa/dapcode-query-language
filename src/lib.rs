//! # DQL (DapCode Query Language)
//!
//! Bahasa kueri modern, ergonomis, dan mandiri yang dirancang untuk DapCode
//! serta interoperabel dengan berbagai bahasa pemrograman lain (Python, Node.js, Go, Rust, C).

pub mod token;
pub mod lexer;
pub mod ast;
pub mod parser;
pub mod compiler;
pub mod executor;
pub mod ffi;

pub use token::{Token, TokenKind};
pub use lexer::Lexer;
pub use ast::*;
pub use parser::Parser;
pub use compiler::{compile, CompileResult, Dialect};
pub use executor::{LiveExecutor, LiveTarget};
