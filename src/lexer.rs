//! Lexer (tokenizer) for DQL

use crate::token::{Token, TokenKind};

#[derive(Debug, Clone)]
pub struct LexerError {
    pub message: String,
    pub line: usize,
    pub col: usize,
}

impl std::fmt::Display for LexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Baris {}, Kolom {}] Kesalahan Leksikal: {}", self.line, self.col, self.message)
    }
}

impl std::error::Error for LexerError {}

pub struct Lexer {
    chars: Vec<(usize, char)>,
    cursor: usize,
    line: usize,
    col: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        let chars: Vec<(usize, char)> = source.char_indices().collect();
        Self {
            chars,
            cursor: 0,
            line: 1,
            col: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            self.skip_whitespace_and_comments();
            if self.is_at_end() {
                break;
            }

            let start_line = self.line;
            let start_col = self.col;
            let ch = self.advance();

            match ch {
                '(' => tokens.push(Token::new(TokenKind::LParen, start_line, start_col, "(")),
                ')' => tokens.push(Token::new(TokenKind::RParen, start_line, start_col, ")")),
                '{' => tokens.push(Token::new(TokenKind::LBrace, start_line, start_col, "{")),
                '}' => tokens.push(Token::new(TokenKind::RBrace, start_line, start_col, "}")),
                '[' => tokens.push(Token::new(TokenKind::LBracket, start_line, start_col, "[")),
                ']' => tokens.push(Token::new(TokenKind::RBracket, start_line, start_col, "]")),
                ',' => tokens.push(Token::new(TokenKind::Comma, start_line, start_col, ",")),
                ':' => tokens.push(Token::new(TokenKind::Colon, start_line, start_col, ":")),
                ';' => tokens.push(Token::new(TokenKind::Semicolon, start_line, start_col, ";")),
                '.' => tokens.push(Token::new(TokenKind::Dot, start_line, start_col, ".")),
                '+' => tokens.push(Token::new(TokenKind::Plus, start_line, start_col, "+")),
                '-' => tokens.push(Token::new(TokenKind::Minus, start_line, start_col, "-")),
                '*' => tokens.push(Token::new(TokenKind::Star, start_line, start_col, "*")),
                '/' => tokens.push(Token::new(TokenKind::Slash, start_line, start_col, "/")),
                '|' => {
                    if self.peek() == Some('|') {
                        self.advance();
                        tokens.push(Token::new(TokenKind::Atau, start_line, start_col, "||"));
                    } else {
                        tokens.push(Token::new(TokenKind::Pipe, start_line, start_col, "|"));
                    }
                }
                '&' => {
                    if self.peek() == Some('&') {
                        self.advance();
                        tokens.push(Token::new(TokenKind::Dan, start_line, start_col, "&&"));
                    } else {
                        return Err(LexerError {
                            message: "Karakter tidak dikenal: '&', gunakan '&&' atau 'dan'".to_string(),
                            line: start_line,
                            col: start_col,
                        });
                    }
                }
                '=' => {
                    if self.peek() == Some('=') {
                        self.advance();
                        tokens.push(Token::new(TokenKind::Equal, start_line, start_col, "=="));
                    } else {
                        tokens.push(Token::new(TokenKind::Equal, start_line, start_col, "="));
                    }
                }
                '!' => {
                    if self.peek() == Some('=') {
                        self.advance();
                        tokens.push(Token::new(TokenKind::NotEqual, start_line, start_col, "!="));
                    } else {
                        tokens.push(Token::new(TokenKind::Tidak, start_line, start_col, "!"));
                    }
                }
                '<' => {
                    if self.peek() == Some('=') {
                        self.advance();
                        tokens.push(Token::new(TokenKind::LessEqual, start_line, start_col, "<="));
                    } else if self.peek() == Some('>') {
                        self.advance();
                        tokens.push(Token::new(TokenKind::NotEqual, start_line, start_col, "<>"));
                    } else {
                        tokens.push(Token::new(TokenKind::LessThan, start_line, start_col, "<"));
                    }
                }
                '>' => {
                    if self.peek() == Some('=') {
                        self.advance();
                        tokens.push(Token::new(TokenKind::GreaterEqual, start_line, start_col, ">="));
                    } else {
                        tokens.push(Token::new(TokenKind::GreaterThan, start_line, start_col, ">"));
                    }
                }
                '"' | '\'' => {
                    let s = self.read_string(ch, start_line, start_col)?;
                    tokens.push(Token::new(TokenKind::StringLiteral(s.clone()), start_line, start_col, s));
                }
                c if c.is_ascii_digit() => {
                    let (kind, lexeme) = self.read_number(c, start_line, start_col)?;
                    tokens.push(Token::new(kind, start_line, start_col, lexeme));
                }
                c if is_identifier_start(c) => {
                    let lexeme = self.read_identifier(c);
                    let kind = match_keyword_or_ident(&lexeme);
                    tokens.push(Token::new(kind, start_line, start_col, lexeme));
                }
                other => {
                    return Err(LexerError {
                        message: format!("Karakter tidak valid atau tidak didukung: '{}'", other),
                        line: start_line,
                        col: start_col,
                    });
                }
            }
        }

        tokens.push(Token::new(TokenKind::Eof, self.line, self.col, ""));
        Ok(tokens)
    }

    fn peek(&self) -> Option<char> {
        if self.cursor < self.chars.len() {
            Some(self.chars[self.cursor].1)
        } else {
            None
        }
    }

    fn advance(&mut self) -> char {
        let ch = self.chars[self.cursor].1;
        self.cursor += 1;
        if ch == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        ch
    }

    fn is_at_end(&self) -> bool {
        self.cursor >= self.chars.len()
    }

    fn skip_whitespace_and_comments(&mut self) {
        while !self.is_at_end() {
            if let Some(ch) = self.peek() {
                if ch.is_whitespace() || ch == '\u{feff}' {
                    self.advance();
                } else if ch == '#' {
                    while !self.is_at_end() && self.peek() != Some('\n') {
                        self.advance();
                    }
                } else if ch == '/' && self.cursor + 1 < self.chars.len() && self.chars[self.cursor + 1].1 == '/' {
                    while !self.is_at_end() && self.peek() != Some('\n') {
                        self.advance();
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    fn read_string(&mut self, quote: char, start_line: usize, start_col: usize) -> Result<String, LexerError> {
        let mut result = String::new();

        while !self.is_at_end() {
            let ch = self.advance();
            if ch == quote {
                return Ok(result);
            }
            if ch == '\\' {
                if self.is_at_end() {
                    break;
                }
                let esc = self.advance();
                match esc {
                    'n' => result.push('\n'),
                    't' => result.push('\t'),
                    'r' => result.push('\r'),
                    '\\' => result.push('\\'),
                    '\'' => result.push('\''),
                    '"' => result.push('"'),
                    other => {
                        result.push('\\');
                        result.push(other);
                    }
                }
            } else {
                result.push(ch);
            }
        }

        Err(LexerError {
            message: format!("Literal teks (string) tidak ditutup dengan {}", quote),
            line: start_line,
            col: start_col,
        })
    }

    fn read_number(&mut self, first_digit: char, _start_line: usize, _start_col: usize) -> Result<(TokenKind, String), LexerError> {
        let mut s = String::new();
        s.push(first_digit);
        let mut is_float = false;

        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                s.push(self.advance());
            } else if ch == '.' && !is_float {
                if self.cursor + 1 < self.chars.len() && self.chars[self.cursor + 1].1.is_ascii_digit() {
                    is_float = true;
                    s.push(self.advance());
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        if is_float {
            let val: f64 = s.parse().unwrap_or(0.0);
            Ok((TokenKind::FloatLiteral(val), s))
        } else {
            let val: i64 = s.parse().unwrap_or(0);
            Ok((TokenKind::IntLiteral(val), s))
        }
    }

    fn read_identifier(&mut self, first_char: char) -> String {
        let mut s = String::new();
        s.push(first_char);

        while let Some(ch) = self.peek() {
            if is_identifier_part(ch) {
                s.push(self.advance());
            } else {
                break;
            }
        }

        s
    }
}

fn is_identifier_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

fn is_identifier_part(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn match_keyword_or_ident(lexeme: &str) -> TokenKind {
    let lower = lexeme.to_lowercase();
    match lower.as_str() {
        "ambil" => TokenKind::Ambil,
        "dari" => TokenKind::Dari,
        "pilih" => TokenKind::Pilih,
        "sebagai" | "as" => TokenKind::Sebagai,
        "di_mana" | "dimana" | "filter" | "where" => TokenKind::DiMana,
        "gabung" | "join" => TokenKind::Gabung,
        "gabung_kiri" | "left_join" => TokenKind::GabungKiri,
        "gabung_kanan" | "right_join" => TokenKind::GabungKanan,
        "gabung_dalam" | "inner_join" => TokenKind::GabungDalam,
        "pada" | "on" => TokenKind::Pada,
        "urutkan" | "order_by" => TokenKind::Urutkan,
        "naik" | "asc" => TokenKind::Naik,
        "turun" | "desc" => TokenKind::Turun,
        "kelompokkan" | "group_by" => TokenKind::Kelompokkan,
        "memiliki" | "having" => TokenKind::Memiliki,
        "batasi" | "limit" => TokenKind::Batasi,
        "lewati" | "offset" => TokenKind::Lewati,
        "tambah" | "tambah_ke" | "sisip" | "insert_into" => TokenKind::TambahKe,
        "perbarui" | "update" => TokenKind::Perbarui,
        "atur" | "set" => TokenKind::Atur,
        "hapus_dari" | "delete_from" => TokenKind::HapusDari,
        "buat" | "create" => TokenKind::Buat,
        "hapus" | "drop" => TokenKind::Hapus,
        "tabel" | "table" => TokenKind::Tabel,
        "basis_data" | "database" => TokenKind::BasisData,
        "jika_ada" | "if_exists" => TokenKind::JikaAda,
        "jika_belum_ada" | "if_not_exists" => TokenKind::JikaBelumAda,
        "tampilkan" | "lihat" | "show" => TokenKind::Tampilkan,
        "struktur" | "deskripsi" | "describe" => TokenKind::Struktur,
        "kunci_utama" | "primary_key" | "pk" => TokenKind::KunciUtama,
        "tidak_kosong" | "not_null" | "bukan_null" => TokenKind::TidakKosong,
        "unik" | "unique" => TokenKind::Unik,
        "bawaan" | "default" => TokenKind::Bawaan,
        "otomatis" | "auto_increment" => TokenKind::Otomatis,
        "sekarang" | "now" | "current_timestamp" => TokenKind::Sekarang,

        "dan" | "and" => TokenKind::Dan,
        "atau" | "or" => TokenKind::Atau,
        "tidak" | "bukan" | "not" => TokenKind::Tidak,
        "seperti" | "like" => TokenKind::Seperti,
        "berisi" => TokenKind::Berisi,
        "diawali" => TokenKind::Diawali,
        "diakhiri" => TokenKind::Diakhiri,
        "di_dalam" | "dalam" | "in" => TokenKind::DiDalam,
        "antara" | "between" => TokenKind::Antara,

        "hitung" | "count" => TokenKind::Hitung,
        "jumlah" | "sum" => TokenKind::Jumlah,
        "rata_rata" | "avg" => TokenKind::RataRata,
        "min" => TokenKind::Min,
        "maks" | "max" => TokenKind::Maks,

        "benar" | "true" => TokenKind::BoolLiteral(true),
        "salah" | "false" => TokenKind::BoolLiteral(false),
        "nihil" | "kosong" | "null" => TokenKind::Nihil,

        _ => TokenKind::Identifier(lexeme.to_string()),
    }
}
