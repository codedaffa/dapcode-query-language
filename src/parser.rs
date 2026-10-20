//! Recursive descent parser for DQL

use crate::ast::*;
use crate::token::{Token, TokenKind};

#[derive(Debug, Clone)]
pub struct ParserError {
    pub message: String,
    pub line: usize,
    pub col: usize,
}

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Baris {}, Kolom {}] Kesalahan Sintaks: {}", self.line, self.col, self.message)
    }
}

impl std::error::Error for ParserError {}

pub struct Parser {
    tokens: Vec<Token>,
    cursor: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, cursor: 0 }
    }

    pub fn parse_statement(&mut self) -> Result<DqlStatement, ParserError> {
        self.skip_pipes_and_semicolons();
        let tok = self.peek();

        let stmt = match tok.kind {
            TokenKind::Ambil | TokenKind::Dari => self.parse_ambil()?,
            TokenKind::TambahKe => self.parse_tambah()?,
            TokenKind::Perbarui => self.parse_perbarui()?,
            TokenKind::HapusDari => self.parse_hapus()?,
            TokenKind::Buat => self.parse_buat()?,
            TokenKind::Hapus => self.parse_hapus_statement()?,
            TokenKind::Tampilkan => self.parse_tampilkan()?,
            TokenKind::Struktur => self.parse_struktur()?,
            _ => return Err(self.error(format!("Kueri DQL tidak dikenal atau tidak didukung: diawali dengan '{}'", tok.lexeme))),
        };
        self.skip_pipes_and_semicolons();
        Ok(stmt)
    }

    fn skip_pipes(&mut self) {
        while self.match_token(&[TokenKind::Pipe]) {}
    }

    fn skip_pipes_and_semicolons(&mut self) {
        while self.match_token(&[TokenKind::Pipe, TokenKind::Semicolon]) {}
    }

    // ==========================================
    // PARSER: AMBIL / DARI (SELECT / FROM)
    // ==========================================
    fn parse_ambil(&mut self) -> Result<DqlStatement, ParserError> {
        self.advance(); // consume 'ambil' or 'dari'

        let mut initial_fields = Vec::new();
        if self.match_token(&[TokenKind::Star]) {
            initial_fields.push(SelectItem::All);
            self.consume_keyword(&[TokenKind::Dari], "Diharapkan 'dari' setelah 'ambil *'")?;
        } else if self.match_token(&[TokenKind::Pilih]) {
            initial_fields = self.parse_select_fields()?;
            self.consume_keyword(&[TokenKind::Dari], "Diharapkan 'dari' setelah daftar kolom 'pilih'")?;
        }

        let table_name = self.consume_ident("Diharapkan nama tabel setelah 'ambil'/'dari'")?;
        let table_alias = if self.match_token(&[TokenKind::Sebagai]) {
            Some(self.consume_ident("Diharapkan nama alias setelah 'sebagai'")?)
        } else {
            None
        };

        let table = TableRef {
            name: table_name,
            alias: table_alias,
        };

        let mut fields = initial_fields;
        let mut joins = Vec::new();
        let mut filter = None;
        let mut group_by = Vec::new();
        let mut having = None;
        let mut order_by = Vec::new();
        let mut limit = None;
        let mut offset = None;

        while !self.is_at_end() {
            self.skip_pipes();
            if self.is_at_end() {
                break;
            }

            match self.peek().kind {
                TokenKind::Pilih => {
                    self.advance();
                    fields = self.parse_select_fields()?;
                }
                TokenKind::DiMana => {
                    self.advance();
                    filter = Some(self.parse_expr()?);
                }
                TokenKind::Gabung | TokenKind::GabungKiri | TokenKind::GabungKanan | TokenKind::GabungDalam => {
                    let join_type = match self.peek().kind {
                        TokenKind::GabungKiri => JoinType::Left,
                        TokenKind::GabungKanan => JoinType::Right,
                        _ => JoinType::Inner,
                    };
                    self.advance();

                    let join_table_name = self.consume_ident("Diharapkan nama tabel setelah 'gabung'")?;
                    let join_table_alias = if self.match_token(&[TokenKind::Sebagai]) {
                        Some(self.consume_ident("Diharapkan nama alias setelah 'sebagai'")?)
                    } else {
                        None
                    };

                    self.consume_keyword(&[TokenKind::Pada], "Diharapkan 'pada' (ON) untuk kondisi penggabungan tabel")?;
                    let condition = self.parse_expr()?;

                    joins.push(JoinClause {
                        join_type,
                        table: TableRef {
                            name: join_table_name,
                            alias: join_table_alias,
                        },
                        condition,
                    });
                }
                TokenKind::Kelompokkan => {
                    self.advance();
                    group_by.push(self.parse_expr()?);
                    while self.match_token(&[TokenKind::Comma]) {
                        group_by.push(self.parse_expr()?);
                    }
                }
                TokenKind::Memiliki => {
                    self.advance();
                    having = Some(self.parse_expr()?);
                }
                TokenKind::Urutkan => {
                    self.advance();
                    order_by = self.parse_order_by_list()?;
                }
                TokenKind::Batasi => {
                    self.advance();
                    let num = self.consume_int("Diharapkan angka bulat setelah 'batasi'")?;
                    limit = Some(num as usize);
                }
                TokenKind::Lewati => {
                    self.advance();
                    let num = self.consume_int("Diharapkan angka bulat setelah 'lewati'")?;
                    offset = Some(num as usize);
                }
                _ => break,
            }
        }

        if fields.is_empty() {
            fields.push(SelectItem::All);
        }

        Ok(DqlStatement::Ambil(AmbilQuery {
            table,
            fields,
            joins,
            filter,
            group_by,
            having,
            order_by,
            limit,
            offset,
        }))
    }

    fn parse_select_fields(&mut self) -> Result<Vec<SelectItem>, ParserError> {
        let mut items = Vec::new();

        if self.match_token(&[TokenKind::Star]) {
            items.push(SelectItem::All);
            return Ok(items);
        }

        loop {
            let expr = self.parse_expr()?;
            let alias = if self.match_token(&[TokenKind::Sebagai]) {
                Some(self.consume_ident("Diharapkan nama alias kolom setelah 'sebagai'")?)
            } else {
                None
            };
            items.push(SelectItem::Expr { expr, alias });

            if !self.match_token(&[TokenKind::Comma]) {
                break;
            }
        }

        Ok(items)
    }

    fn parse_order_by_list(&mut self) -> Result<Vec<OrderByClause>, ParserError> {
        let mut list = Vec::new();

        loop {
            let expr = self.parse_expr()?;
            let direction = if self.match_token(&[TokenKind::Turun]) {
                OrderDirection::Desc
            } else if self.match_token(&[TokenKind::Naik]) {
                OrderDirection::Asc
            } else {
                OrderDirection::Asc
            };

            list.push(OrderByClause { expr, direction });

            if !self.match_token(&[TokenKind::Comma]) {
                break;
            }
        }

        Ok(list)
    }

    // ==========================================
    // PARSER: TAMBAH_KE (INSERT)
    // ==========================================
    fn parse_tambah(&mut self) -> Result<DqlStatement, ParserError> {
        self.advance(); // consume 'tambah' / 'tambah_ke'
        if let TokenKind::Identifier(ref s) = self.peek().kind {
            if s.to_lowercase() == "ke" {
                self.advance();
            }
        }
        let table_name = self.consume_ident("Diharapkan nama tabel setelah 'tambah_ke'")?;
        let table = TableRef {
            name: table_name,
            alias: None,
        };

        let mut records = Vec::new();

        if self.match_token(&[TokenKind::LBracket]) {
            // Daftar record [ { ... }, { ... } ]
            while !self.check(&TokenKind::RBracket) && !self.is_at_end() {
                records.push(self.parse_single_record()?);
                if !self.match_token(&[TokenKind::Comma]) {
                    break;
                }
            }
            self.consume_keyword(&[TokenKind::RBracket], "Diharapkan ']' penutup daftar record")?;
        } else if self.check(&TokenKind::LBrace) {
            // Record tunggal { ... }
            records.push(self.parse_single_record()?);
        } else {
            return Err(self.error("Diharapkan objek '{ ... }' atau daftar '[ { ... } ]' setelah nama tabel"));
        }

        Ok(DqlStatement::Tambah(TambahQuery { table, records }))
    }

    fn parse_single_record(&mut self) -> Result<Vec<RecordAssignment>, ParserError> {
        self.consume_keyword(&[TokenKind::LBrace], "Diharapkan '{' pembuka data record")?;
        let mut assignments = Vec::new();

        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            let field = self.consume_ident("Diharapkan nama kolom pada data record")?;
            self.consume_keyword(&[TokenKind::Colon, TokenKind::Equal], "Diharapkan ':' atau '=' setelah nama kolom")?;
            let value = self.parse_expr()?;

            assignments.push(RecordAssignment { field, value });

            if !self.match_token(&[TokenKind::Comma]) {
                break;
            }
        }

        self.consume_keyword(&[TokenKind::RBrace], "Diharapkan '}' penutup data record")?;
        Ok(assignments)
    }

    // ==========================================
    // PARSER: PERBARUI (UPDATE)
    // ==========================================
    fn parse_perbarui(&mut self) -> Result<DqlStatement, ParserError> {
        self.advance(); // consume 'perbarui'
        let table_name = self.consume_ident("Diharapkan nama tabel setelah 'perbarui'")?;
        let table = TableRef {
            name: table_name,
            alias: None,
        };

        self.skip_pipes();
        self.consume_keyword(&[TokenKind::Atur], "Diharapkan 'atur' (SET) setelah nama tabel pada perintah 'perbarui'")?;

        let mut assignments = Vec::new();
        loop {
            let field = self.consume_ident("Diharapkan nama kolom yang ingin diperbarui")?;
            self.consume_keyword(&[TokenKind::Equal], "Diharapkan '=' setelah nama kolom")?;
            let value = self.parse_expr()?;

            assignments.push(RecordAssignment { field, value });

            if !self.match_token(&[TokenKind::Comma]) {
                break;
            }
        }

        self.skip_pipes();
        let filter = if self.match_token(&[TokenKind::DiMana]) {
            Some(self.parse_expr()?)
        } else {
            None
        };

        Ok(DqlStatement::Perbarui(PerbaruiQuery {
            table,
            assignments,
            filter,
        }))
    }

    // ==========================================
    // PARSER: HAPUS_DARI (DELETE)
    // ==========================================
    fn parse_hapus(&mut self) -> Result<DqlStatement, ParserError> {
        self.advance(); // consume 'hapus_dari'
        let table_name = self.consume_ident("Diharapkan nama tabel setelah 'hapus_dari'")?;
        let table = TableRef {
            name: table_name,
            alias: None,
        };

        self.skip_pipes();
        let filter = if self.match_token(&[TokenKind::DiMana]) {
            Some(self.parse_expr()?)
        } else {
            None
        };

        Ok(DqlStatement::Hapus(HapusQuery { table, filter }))
    }

    // ==========================================
    // PARSER: DDL & DATABASE MANAGEMENT
    // ==========================================
    fn parse_buat(&mut self) -> Result<DqlStatement, ParserError> {
        self.advance(); // consume 'buat'
        match self.peek().kind {
            TokenKind::Tabel => {
                self.advance(); // consume 'tabel'
                let if_not_exists = self.match_token(&[TokenKind::JikaBelumAda]);
                let table_name = self.consume_ident("Diharapkan nama tabel setelah 'buat tabel'")?;

                let is_brace = self.match_token(&[TokenKind::LBrace]);
                if !is_brace && !self.match_token(&[TokenKind::LParen]) {
                    return Err(self.error("Diharapkan '{' atau '(' untuk membuka daftar kolom tabel"));
                }

                let mut columns = Vec::new();
                while !self.is_at_end() {
                    if is_brace && self.check(&TokenKind::RBrace) {
                        break;
                    }
                    if !is_brace && self.check(&TokenKind::RParen) {
                        break;
                    }

                    let col_name = self.consume_ident("Diharapkan nama kolom")?;
                    self.match_token(&[TokenKind::Colon]);

                    let col_type = self.parse_column_type()?;

                    let mut constraints = ColumnConstraints::default();
                    loop {
                        if self.match_token(&[TokenKind::KunciUtama]) {
                            constraints.is_primary_key = true;
                        } else if self.match_token(&[TokenKind::TidakKosong]) {
                            constraints.is_not_null = true;
                        } else if self.check(&TokenKind::Tidak) {
                            if self.cursor + 1 < self.tokens.len() && self.tokens[self.cursor + 1].kind == TokenKind::Nihil {
                                self.advance();
                                self.advance();
                                constraints.is_not_null = true;
                            } else {
                                break;
                            }
                        } else if self.match_token(&[TokenKind::Unik]) {
                            constraints.is_unique = true;
                        } else if self.match_token(&[TokenKind::Otomatis]) {
                            constraints.is_auto_increment = true;
                        } else if self.match_token(&[TokenKind::Bawaan]) {
                            if self.match_token(&[TokenKind::Sekarang]) {
                                constraints.default_value = Some("CURRENT_TIMESTAMP".to_string());
                            } else if let TokenKind::StringLiteral(s) = &self.peek().kind {
                                let val = s.clone();
                                self.advance();
                                constraints.default_value = Some(format!("'{}'", val));
                            } else if let TokenKind::IntLiteral(i) = self.peek().kind {
                                self.advance();
                                constraints.default_value = Some(i.to_string());
                            } else if let TokenKind::FloatLiteral(f) = self.peek().kind {
                                self.advance();
                                constraints.default_value = Some(f.to_string());
                            } else if let TokenKind::BoolLiteral(b) = self.peek().kind {
                                self.advance();
                                constraints.default_value = Some(b.to_string());
                            } else if let TokenKind::Identifier(id) = &self.peek().kind {
                                let val = id.clone();
                                self.advance();
                                constraints.default_value = Some(val);
                            } else {
                                return Err(self.error("Diharapkan nilai bawaan setelah 'bawaan'"));
                            }
                        } else {
                            break;
                        }
                    }

                    columns.push(ColumnDefinition {
                        name: col_name,
                        col_type,
                        constraints,
                    });

                    self.match_token(&[TokenKind::Comma]);
                }

                if is_brace {
                    self.consume_keyword(&[TokenKind::RBrace], "Diharapkan '}' untuk menutup definisi kolom")?;
                } else {
                    self.consume_keyword(&[TokenKind::RParen], "Diharapkan ')' untuk menutup definisi kolom")?;
                }

                Ok(DqlStatement::BuatTabel(BuatTabelQuery {
                    table_name,
                    if_not_exists,
                    columns,
                }))
            }
            TokenKind::BasisData => {
                self.advance(); // consume 'basis_data'
                let name = self.consume_ident("Diharapkan nama database setelah 'buat basis_data'")?;
                Ok(DqlStatement::BuatDatabase(BuatDatabaseQuery { name }))
            }
            _ => Err(self.error("Diharapkan 'tabel' atau 'basis_data' setelah kata kunci 'buat'")),
        }
    }

    fn parse_column_type(&mut self) -> Result<ColumnType, ParserError> {
        let type_name = self.consume_ident("Diharapkan tipe data kolom (contoh: seri, teks, bulat, desimal, boolean, waktu)")?;
        let lower = type_name.to_lowercase();
        match lower.as_str() {
            "seri" | "serial" | "id_otomatis" => Ok(ColumnType::Serial),
            "teks" | "text" | "string" => Ok(ColumnType::Text),
            "bulat" | "integer" | "int" => Ok(ColumnType::Integer),
            "desimal" | "pecahan" | "float" | "double" => Ok(ColumnType::Float),
            "boolean" | "bool" => Ok(ColumnType::Boolean),
            "waktu" | "tanggal" | "timestamp" | "datetime" => Ok(ColumnType::Timestamp),
            "json" => Ok(ColumnType::Json),
            "blob" | "biner" => Ok(ColumnType::Blob),
            custom => {
                if self.match_token(&[TokenKind::LParen]) {
                    if let TokenKind::IntLiteral(len) = self.peek().kind {
                        self.advance();
                        self.consume_keyword(&[TokenKind::RParen], "Diharapkan ')'")?;
                        return Ok(ColumnType::Custom(format!("{}({})", custom, len)));
                    }
                }
                Ok(ColumnType::Custom(custom.to_string()))
            }
        }
    }

    fn parse_hapus_statement(&mut self) -> Result<DqlStatement, ParserError> {
        self.advance(); // consume 'hapus'
        match self.peek().kind {
            TokenKind::Dari => {
                self.parse_hapus()
            }
            TokenKind::Tabel => {
                self.advance(); // consume 'tabel'
                let if_exists = self.match_token(&[TokenKind::JikaAda]);
                let table_name = self.consume_ident("Diharapkan nama tabel setelah 'hapus tabel'")?;
                Ok(DqlStatement::HapusTabel(HapusTabelQuery {
                    table_name,
                    if_exists,
                }))
            }
            TokenKind::BasisData => {
                self.advance(); // consume 'basis_data'
                let if_exists = self.match_token(&[TokenKind::JikaAda]);
                let name = self.consume_ident("Diharapkan nama database setelah 'hapus basis_data'")?;
                Ok(DqlStatement::HapusDatabase(HapusDatabaseQuery {
                    name,
                    if_exists,
                }))
            }
            _ => Err(self.error("Diharapkan 'tabel', 'basis_data', atau 'dari' setelah kata kunci 'hapus'")),
        }
    }

    fn parse_tampilkan(&mut self) -> Result<DqlStatement, ParserError> {
        self.advance(); // consume 'tampilkan' / 'lihat' / 'show'
        if self.match_token(&[TokenKind::Tabel]) {
            Ok(DqlStatement::TampilkanTabel)
        } else if self.match_token(&[TokenKind::BasisData]) {
            Ok(DqlStatement::TampilkanDatabase)
        } else {
            Err(self.error("Diharapkan 'tabel' atau 'basis_data' setelah 'tampilkan' (contoh: 'tampilkan tabel', 'tampilkan basis_data')"))
        }
    }

    fn parse_struktur(&mut self) -> Result<DqlStatement, ParserError> {
        self.advance(); // consume 'struktur' / 'deskripsi'
        let table_name = self.consume_ident("Diharapkan nama tabel setelah 'struktur'/'deskripsi'")?;
        Ok(DqlStatement::DeskripsiTabel(table_name))
    }

    // ==========================================
    // PARSER: EXPRESSIONS (Precedence Climbing)
    // ==========================================
    pub fn parse_expr(&mut self) -> Result<Expr, ParserError> {
        self.parse_logical_or()
    }

    fn parse_logical_or(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.parse_logical_and()?;

        while self.match_token(&[TokenKind::Atau]) {
            let right = self.parse_logical_and()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op: BinaryOp::Or,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_logical_and(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.parse_equality()?;

        while self.match_token(&[TokenKind::Dan]) {
            let right = self.parse_equality()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op: BinaryOp::And,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_equality(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.parse_comparison()?;

        loop {
            if self.match_token(&[TokenKind::Equal]) {
                let right = self.parse_comparison()?;
                expr = Expr::Binary {
                    left: Box::new(expr),
                    op: BinaryOp::Eq,
                    right: Box::new(right),
                };
            } else if self.match_token(&[TokenKind::NotEqual]) {
                let right = self.parse_comparison()?;
                expr = Expr::Binary {
                    left: Box::new(expr),
                    op: BinaryOp::Neq,
                    right: Box::new(right),
                };
            } else if self.match_token(&[TokenKind::Seperti]) {
                let right = self.parse_comparison()?;
                expr = Expr::Binary {
                    left: Box::new(expr),
                    op: BinaryOp::Like,
                    right: Box::new(right),
                };
            } else if self.match_token(&[TokenKind::Berisi]) {
                let right = self.parse_comparison()?;
                expr = Expr::Binary {
                    left: Box::new(expr),
                    op: BinaryOp::Contains,
                    right: Box::new(right),
                };
            } else if self.match_token(&[TokenKind::Diawali]) {
                let right = self.parse_comparison()?;
                expr = Expr::Binary {
                    left: Box::new(expr),
                    op: BinaryOp::StartsWith,
                    right: Box::new(right),
                };
            } else if self.match_token(&[TokenKind::Diakhiri]) {
                let right = self.parse_comparison()?;
                expr = Expr::Binary {
                    left: Box::new(expr),
                    op: BinaryOp::EndsWith,
                    right: Box::new(right),
                };
            } else if self.match_token(&[TokenKind::DiDalam]) {
                self.consume_keyword(&[TokenKind::LParen], "Diharapkan '(' setelah 'di_dalam'")?;
                let mut list = Vec::new();
                while !self.check(&TokenKind::RParen) && !self.is_at_end() {
                    list.push(self.parse_expr()?);
                    if !self.match_token(&[TokenKind::Comma]) {
                        break;
                    }
                }
                self.consume_keyword(&[TokenKind::RParen], "Diharapkan ')' penutup 'di_dalam'")?;
                expr = Expr::InList {
                    expr: Box::new(expr),
                    list,
                    negated: false,
                };
            } else if self.match_token(&[TokenKind::Antara]) {
                let low = self.parse_comparison()?;
                self.consume_keyword(&[TokenKind::Dan], "Diharapkan 'dan' pada klausa 'antara A dan B'")?;
                let high = self.parse_comparison()?;
                expr = Expr::Between {
                    expr: Box::new(expr),
                    low: Box::new(low),
                    high: Box::new(high),
                    negated: false,
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.parse_term()?;

        loop {
            let op = if self.match_token(&[TokenKind::LessThan]) {
                BinaryOp::Lt
            } else if self.match_token(&[TokenKind::LessEqual]) {
                BinaryOp::Lte
            } else if self.match_token(&[TokenKind::GreaterThan]) {
                BinaryOp::Gt
            } else if self.match_token(&[TokenKind::GreaterEqual]) {
                BinaryOp::Gte
            } else {
                break;
            };

            let right = self.parse_term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.parse_factor()?;

        loop {
            let op = if self.match_token(&[TokenKind::Plus]) {
                BinaryOp::Add
            } else if self.match_token(&[TokenKind::Minus]) {
                BinaryOp::Sub
            } else {
                break;
            };

            let right = self.parse_factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.parse_unary()?;

        loop {
            let op = if self.match_token(&[TokenKind::Star]) {
                BinaryOp::Mul
            } else if self.match_token(&[TokenKind::Slash]) {
                BinaryOp::Div
            } else {
                break;
            };

            let right = self.parse_unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParserError> {
        if self.match_token(&[TokenKind::Tidak]) {
            let expr = self.parse_unary()?;
            return Ok(Expr::Unary {
                op: UnaryOp::Not,
                expr: Box::new(expr),
            });
        }
        if self.match_token(&[TokenKind::Minus]) {
            let expr = self.parse_unary()?;
            return Ok(Expr::Unary {
                op: UnaryOp::Neg,
                expr: Box::new(expr),
            });
        }

        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr, ParserError> {
        let tok = self.peek();

        match &tok.kind {
            TokenKind::IntLiteral(n) => {
                let val = *n;
                self.advance();
                Ok(Expr::Literal(LiteralValue::Int(val)))
            }
            TokenKind::FloatLiteral(f) => {
                let val = *f;
                self.advance();
                Ok(Expr::Literal(LiteralValue::Float(val)))
            }
            TokenKind::StringLiteral(s) => {
                let val = s.clone();
                self.advance();
                Ok(Expr::Literal(LiteralValue::String(val)))
            }
            TokenKind::BoolLiteral(b) => {
                let val = *b;
                self.advance();
                Ok(Expr::Literal(LiteralValue::Bool(val)))
            }
            TokenKind::Nihil => {
                self.advance();
                Ok(Expr::Literal(LiteralValue::Null))
            }
            TokenKind::Hitung | TokenKind::Jumlah | TokenKind::RataRata | TokenKind::Min | TokenKind::Maks => {
                let fn_name = tok.lexeme.clone();
                self.advance();
                self.consume_keyword(&[TokenKind::LParen], "Diharapkan '(' setelah fungsi agregasi")?;

                let mut args = Vec::new();
                if self.match_token(&[TokenKind::Star]) {
                    args.push(Expr::Column(ColumnRef::simple("*")));
                } else if !self.check(&TokenKind::RParen) {
                    args.push(self.parse_expr()?);
                    while self.match_token(&[TokenKind::Comma]) {
                        args.push(self.parse_expr()?);
                    }
                }
                self.consume_keyword(&[TokenKind::RParen], "Diharapkan ')' penutup fungsi agregasi")?;

                Ok(Expr::FunctionCall {
                    name: fn_name,
                    args,
                })
            }
            TokenKind::Identifier(name) => {
                let ident = name.clone();
                self.advance();

                // Cek apakah function call `fungsi(...)`
                if self.match_token(&[TokenKind::LParen]) {
                    let mut args = Vec::new();
                    if self.match_token(&[TokenKind::Star]) {
                        args.push(Expr::Column(ColumnRef::simple("*")));
                    } else if !self.check(&TokenKind::RParen) {
                        args.push(self.parse_expr()?);
                        while self.match_token(&[TokenKind::Comma]) {
                            args.push(self.parse_expr()?);
                        }
                    }
                    self.consume_keyword(&[TokenKind::RParen], "Diharapkan ')' penutup fungsi")?;
                    return Ok(Expr::FunctionCall { name: ident, args });
                }

                // Cek apakah kolom berkualifikasi: `Tabel.kolom`
                if self.match_token(&[TokenKind::Dot]) {
                    if self.match_token(&[TokenKind::Star]) {
                        return Ok(Expr::Column(ColumnRef::qualified(ident, "*")));
                    }
                    let col_name = self.consume_ident("Diharapkan nama kolom setelah '.'")?;
                    return Ok(Expr::Column(ColumnRef::qualified(ident, col_name)));
                }

                Ok(Expr::Column(ColumnRef::simple(ident)))
            }
            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.consume_keyword(&[TokenKind::RParen], "Diharapkan ')' penutup ekspresi")?;
                Ok(expr)
            }
            _ => Err(self.error(format!("Diharapkan ekspresi atau nilai, ditemukan '{}'", tok.lexeme))),
        }
    }

    // ==========================================
    // UTILITIES
    // ==========================================
    fn peek(&self) -> &Token {
        if self.cursor < self.tokens.len() {
            &self.tokens[self.cursor]
        } else {
            self.tokens.last().unwrap()
        }
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.cursor += 1;
        }
        &self.tokens[self.cursor - 1]
    }

    fn check(&self, kind: &TokenKind) -> bool {
        if self.is_at_end() {
            false
        } else {
            &self.peek().kind == kind
        }
    }

    fn match_token(&mut self, kinds: &[TokenKind]) -> bool {
        for kind in kinds {
            if self.check(kind) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn consume_keyword(&mut self, kinds: &[TokenKind], err_msg: &str) -> Result<(), ParserError> {
        for kind in kinds {
            if self.check(kind) {
                self.advance();
                return Ok(());
            }
        }
        Err(self.error(err_msg))
    }

    fn consume_ident(&mut self, err_msg: &str) -> Result<String, ParserError> {
        if let TokenKind::Identifier(name) = &self.peek().kind {
            let res = name.clone();
            self.advance();
            Ok(res)
        } else {
            Err(self.error(err_msg))
        }
    }

    fn consume_int(&mut self, err_msg: &str) -> Result<i64, ParserError> {
        if let TokenKind::IntLiteral(val) = self.peek().kind {
            self.advance();
            Ok(val)
        } else {
            Err(self.error(err_msg))
        }
    }

    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenKind::Eof
    }

    fn error(&self, msg: impl Into<String>) -> ParserError {
        let tok = self.peek();
        ParserError {
            message: msg.into(),
            line: tok.line,
            col: tok.col,
        }
    }
}
