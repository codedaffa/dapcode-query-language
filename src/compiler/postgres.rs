//! PostgreSQL Code Generator for DQL

use crate::ast::*;
use crate::compiler::CompileResult;

pub struct PostgresCompiler {
    parameterized: bool,
    params: Vec<LiteralValue>,
    param_counter: usize,
}

impl PostgresCompiler {
    pub fn new(parameterized: bool) -> Self {
        Self {
            parameterized,
            params: Vec::new(),
            param_counter: 0,
        }
    }

    pub fn compile(mut self, stmt: &DqlStatement) -> Result<CompileResult, String> {
        let sql = match stmt {
            DqlStatement::Ambil(q) => self.compile_ambil(q)?,
            DqlStatement::Tambah(q) => self.compile_tambah(q)?,
            DqlStatement::Perbarui(q) => self.compile_perbarui(q)?,
            DqlStatement::Hapus(q) => self.compile_hapus(q)?,
            DqlStatement::BuatTabel(q) => self.compile_buat_tabel(q)?,
            DqlStatement::HapusTabel(q) => self.compile_hapus_tabel(q)?,
            DqlStatement::BuatDatabase(q) => self.compile_buat_database(q)?,
            DqlStatement::HapusDatabase(q) => self.compile_hapus_database(q)?,
            DqlStatement::TampilkanTabel => "SELECT table_name AS tabel FROM information_schema.tables WHERE table_schema = 'public' ORDER BY table_name;".to_string(),
            DqlStatement::DeskripsiTabel(t) => format!("SELECT column_name, data_type, is_nullable, column_default FROM information_schema.columns WHERE table_name = '{}' ORDER BY ordinal_position;", t),
        };

        Ok(CompileResult {
            sql,
            params: self.params,
        })
    }

    fn quote_ident(&self, ident: &str) -> String {
        if ident == "*" {
            return "*".to_string();
        }
        format!("\"{}\"", ident)
    }

    fn format_col_ref(&self, col: &ColumnRef) -> String {
        match &col.table {
            Some(t) => format!("{}.{}", self.quote_ident(t), self.quote_ident(&col.column)),
            None => self.quote_ident(&col.column),
        }
    }

    fn format_table_ref(&self, table: &TableRef) -> String {
        match &table.alias {
            Some(a) => format!("{} AS {}", self.quote_ident(&table.name), self.quote_ident(a)),
            None => self.quote_ident(&table.name),
        }
    }

    fn compile_ambil(&mut self, q: &AmbilQuery) -> Result<String, String> {
        let mut out = String::from("SELECT ");

        // Fields
        if q.fields.is_empty() || (q.fields.len() == 1 && q.fields[0] == SelectItem::All) {
            out.push('*');
        } else {
            let field_strs: Result<Vec<String>, String> = q.fields.iter().map(|f| match f {
                SelectItem::All => Ok("*".to_string()),
                SelectItem::Expr { expr, alias } => {
                    let s = self.compile_expr(expr)?;
                    match alias {
                        Some(a) => Ok(format!("{} AS {}", s, self.quote_ident(a))),
                        None => Ok(s),
                    }
                }
            }).collect();
            out.push_str(&field_strs?.join(", "));
        }

        // FROM
        out.push_str(&format!(" FROM {}", self.format_table_ref(&q.table)));

        // JOINS
        for j in &q.joins {
            let j_type = match j.join_type {
                JoinType::Inner => "INNER JOIN",
                JoinType::Left => "LEFT JOIN",
                JoinType::Right => "RIGHT JOIN",
            };
            let cond = self.compile_expr(&j.condition)?;
            out.push_str(&format!(" {} {} ON {}", j_type, self.format_table_ref(&j.table), cond));
        }

        // WHERE
        if let Some(ref filter) = q.filter {
            let cond = self.compile_expr(filter)?;
            out.push_str(&format!(" WHERE {}", cond));
        }

        // GROUP BY
        if !q.group_by.is_empty() {
            let grps: Result<Vec<String>, String> = q.group_by.iter().map(|e| self.compile_expr(e)).collect();
            out.push_str(&format!(" GROUP BY {}", grps?.join(", ")));
        }

        // HAVING
        if let Some(ref having) = q.having {
            let h = self.compile_expr(having)?;
            out.push_str(&format!(" HAVING {}", h));
        }

        // ORDER BY
        if !q.order_by.is_empty() {
            let ords: Result<Vec<String>, String> = q.order_by.iter().map(|o| {
                let e = self.compile_expr(&o.expr)?;
                let dir = match o.direction {
                    OrderDirection::Asc => "ASC",
                    OrderDirection::Desc => "DESC",
                };
                Ok(format!("{} {}", e, dir))
            }).collect();
            out.push_str(&format!(" ORDER BY {}", ords?.join(", ")));
        }

        // LIMIT
        if let Some(lim) = q.limit {
            out.push_str(&format!(" LIMIT {}", lim));
        }

        // OFFSET
        if let Some(off) = q.offset {
            out.push_str(&format!(" OFFSET {}", off));
        }

        out.push(';');
        Ok(out)
    }

    fn compile_tambah(&mut self, q: &TambahQuery) -> Result<String, String> {
        if q.records.is_empty() {
            return Err("Tidak ada data record yang diberikan pada 'tambah_ke'".to_string());
        }

        // Ambil urutan kolom dari record pertama
        let columns: Vec<String> = q.records[0].iter().map(|r| r.field.clone()).collect();
        let col_names = columns.iter().map(|c| self.quote_ident(c)).collect::<Vec<_>>().join(", ");

        let mut values_list = Vec::new();

        for rec in &q.records {
            let mut val_items = Vec::new();
            for col in &columns {
                if let Some(assignment) = rec.iter().find(|r| r.field == *col) {
                    val_items.push(self.compile_expr(&assignment.value)?);
                } else {
                    val_items.push("NULL".to_string());
                }
            }
            values_list.push(format!("({})", val_items.join(", ")));
        }

        Ok(format!(
            "INSERT INTO {} ({}) VALUES {};",
            self.format_table_ref(&q.table),
            col_names,
            values_list.join(", ")
        ))
    }

    fn compile_perbarui(&mut self, q: &PerbaruiQuery) -> Result<String, String> {
        if q.assignments.is_empty() {
            return Err("Tidak ada kolom yang diatur pada 'perbarui'".to_string());
        }

        let sets: Result<Vec<String>, String> = q.assignments.iter().map(|a| {
            let val = self.compile_expr(&a.value)?;
            Ok(format!("{} = {}", self.quote_ident(&a.field), val))
        }).collect();

        let mut out = format!(
            "UPDATE {} SET {}",
            self.format_table_ref(&q.table),
            sets?.join(", ")
        );

        if let Some(ref filter) = q.filter {
            let cond = self.compile_expr(filter)?;
            out.push_str(&format!(" WHERE {}", cond));
        }

        out.push(';');
        Ok(out)
    }

    fn compile_hapus(&mut self, q: &HapusQuery) -> Result<String, String> {
        let mut out = format!("DELETE FROM {}", self.format_table_ref(&q.table));

        if let Some(ref filter) = q.filter {
            let cond = self.compile_expr(filter)?;
            out.push_str(&format!(" WHERE {}", cond));
        }

        out.push(';');
        Ok(out)
    }

    pub fn compile_expr(&mut self, expr: &Expr) -> Result<String, String> {
        match expr {
            Expr::Literal(lit) => {
                if self.parameterized {
                    self.param_counter += 1;
                    self.params.push(lit.clone());
                    Ok(format!("${}", self.param_counter))
                } else {
                    Ok(match lit {
                        LiteralValue::Int(n) => n.to_string(),
                        LiteralValue::Float(f) => f.to_string(),
                        LiteralValue::String(s) => format!("'{}'", s.replace('\'', "''")),
                        LiteralValue::Bool(b) => if *b { "TRUE".to_string() } else { "FALSE".to_string() },
                        LiteralValue::Null => "NULL".to_string(),
                    })
                }
            }
            Expr::Column(col) => Ok(self.format_col_ref(col)),
            Expr::Unary { op, expr } => {
                let inner = self.compile_expr(expr)?;
                match op {
                    UnaryOp::Not => Ok(format!("NOT ({})", inner)),
                    UnaryOp::Neg => Ok(format!("-({})", inner)),
                }
            }
            Expr::Binary { left, op, right } => {
                let l = self.compile_expr(left)?;
                let r = self.compile_expr(right)?;
                let op_str = match op {
                    BinaryOp::Add => "+",
                    BinaryOp::Sub => "-",
                    BinaryOp::Mul => "*",
                    BinaryOp::Div => "/",
                    BinaryOp::Eq => "=",
                    BinaryOp::Neq => "<>",
                    BinaryOp::Lt => "<",
                    BinaryOp::Lte => "<=",
                    BinaryOp::Gt => ">",
                    BinaryOp::Gte => ">=",
                    BinaryOp::And => "AND",
                    BinaryOp::Or => "OR",
                    BinaryOp::Like => "LIKE",
                    BinaryOp::Contains => return Ok(format!("{} LIKE ('%' || {} || '%')", l, r)),
                    BinaryOp::StartsWith => return Ok(format!("{} LIKE ({} || '%')", l, r)),
                    BinaryOp::EndsWith => return Ok(format!("{} LIKE ('%' || {})", l, r)),
                };
                Ok(format!("({} {} {})", l, op_str, r))
            }
            Expr::FunctionCall { name, args } => {
                let lower = name.to_lowercase();
                let sql_fn = match lower.as_str() {
                    "hitung" | "count" => "COUNT",
                    "jumlah" | "sum" => "SUM",
                    "rata_rata" | "avg" => "AVG",
                    "min" => "MIN",
                    "maks" | "max" => "MAX",
                    _ => name.as_str(),
                };
                let arg_strs: Result<Vec<String>, String> = args.iter().map(|a| self.compile_expr(a)).collect();
                Ok(format!("{}({})", sql_fn, arg_strs?.join(", ")))
            }
            Expr::InList { expr, list, negated } => {
                let e = self.compile_expr(expr)?;
                let items: Result<Vec<String>, String> = list.iter().map(|item| self.compile_expr(item)).collect();
                let not_str = if *negated { "NOT " } else { "" };
                Ok(format!("{} {}IN ({})", e, not_str, items?.join(", ")))
            }
            Expr::Between { expr, low, high, negated } => {
                let e = self.compile_expr(expr)?;
                let l = self.compile_expr(low)?;
                let h = self.compile_expr(high)?;
                let not_str = if *negated { "NOT " } else { "" };
                Ok(format!("{} {}BETWEEN {} AND {}", e, not_str, l, h))
            }
        }
    }

    fn compile_buat_tabel(&self, q: &BuatTabelQuery) -> Result<String, String> {
        let if_not_exists = if q.if_not_exists { "IF NOT EXISTS " } else { "" };
        let mut cols_sql = Vec::new();

        for col in &q.columns {
            let mut col_def = format!("{} {}", self.quote_ident(&col.name), self.sql_type_for_column(&col.col_type, &col.constraints));

            if col.constraints.is_primary_key && col.col_type != ColumnType::Serial {
                col_def.push_str(" PRIMARY KEY");
            }
            if col.constraints.is_not_null {
                col_def.push_str(" NOT NULL");
            }
            if col.constraints.is_unique {
                col_def.push_str(" UNIQUE");
            }
            if let Some(def) = &col.constraints.default_value {
                col_def.push_str(&format!(" DEFAULT {}", def));
            }

            cols_sql.push(col_def);
        }

        Ok(format!(
            "CREATE TABLE {}{} ({});",
            if_not_exists,
            self.quote_ident(&q.table_name),
            cols_sql.join(", ")
        ))
    }

    fn sql_type_for_column(&self, col_type: &ColumnType, constraints: &ColumnConstraints) -> String {
        match col_type {
            ColumnType::Serial => {
                if constraints.is_primary_key {
                    "SERIAL PRIMARY KEY".to_string()
                } else {
                    "SERIAL".to_string()
                }
            }
            ColumnType::Text => "TEXT".to_string(),
            ColumnType::Integer => "INTEGER".to_string(),
            ColumnType::Float => "DOUBLE PRECISION".to_string(),
            ColumnType::Boolean => "BOOLEAN".to_string(),
            ColumnType::Timestamp => "TIMESTAMP".to_string(),
            ColumnType::Json => "JSONB".to_string(),
            ColumnType::Blob => "BYTEA".to_string(),
            ColumnType::Custom(s) => s.clone(),
        }
    }

    fn compile_hapus_tabel(&self, q: &HapusTabelQuery) -> Result<String, String> {
        let if_exists = if q.if_exists { "IF EXISTS " } else { "" };
        Ok(format!("DROP TABLE {}{};", if_exists, self.quote_ident(&q.table_name)))
    }

    fn compile_buat_database(&self, q: &BuatDatabaseQuery) -> Result<String, String> {
        Ok(format!("CREATE DATABASE {};", self.quote_ident(&q.name)))
    }

    fn compile_hapus_database(&self, q: &HapusDatabaseQuery) -> Result<String, String> {
        let if_exists = if q.if_exists { "IF EXISTS " } else { "" };
        Ok(format!("DROP DATABASE {}{};", if_exists, self.quote_ident(&q.name)))
    }
}

