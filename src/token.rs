//! Token definitions for DQL (DapCode Query Language)

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords: Query & Manipulation
    Ambil,        // ambil
    Dari,         // dari
    Pilih,        // pilih
    Sebagai,      // sebagai, as
    DiMana,       // di_mana, dimana, filter, where
    Gabung,       // gabung, join
    GabungKiri,   // gabung_kiri, left_join
    GabungKanan,  // gabung_kanan, right_join
    GabungDalam,  // gabung_dalam, inner_join
    Pada,         // pada, on
    Urutkan,      // urutkan, order_by
    Naik,         // naik, asc
    Turun,        // turun, desc
    Kelompokkan,  // kelompokkan, group_by
    Memiliki,     // memiliki, having
    Batasi,       // batasi, limit
    Lewati,       // lewati, offset
    TambahKe,     // tambah_ke, sisip, insert_into
    Perbarui,     // perbarui, update
    Atur,         // atur, set
    HapusDari,    // hapus_dari, delete_from

    // DDL & Skema Basis Data
    Buat,         // buat, create
    Hapus,        // hapus, drop
    Tabel,        // tabel, table
    BasisData,    // basis_data, database
    JikaAda,      // jika_ada, if_exists
    JikaBelumAda, // jika_belum_ada, if_not_exists
    Tampilkan,    // tampilkan, lihat, show
    Struktur,     // struktur, deskripsi, describe, desc
    KunciUtama,   // kunci_utama, primary_key
    TidakKosong,  // tidak_kosong, not_null
    Unik,         // unik, unique
    Bawaan,       // bawaan, default
    Otomatis,     // otomatis, auto_increment
    Sekarang,     // sekarang, now, current_timestamp

    // Logical & Predicates
    Dan,          // dan, and, &&
    Atau,         // atau, or, ||
    Tidak,        // tidak, bukan, not
    Seperti,      // seperti, like
    Berisi,       // berisi (LIKE '%x%')
    Diawali,      // diawali (LIKE 'x%')
    Diakhiri,     // diakhiri (LIKE '%x')
    DiDalam,      // di_dalam, dalam, in
    Antara,       // antara, between

    // Aggregate functions
    Hitung,       // hitung, count
    Jumlah,       // jumlah, sum
    RataRata,     // rata_rata, avg
    Min,          // min
    Maks,         // maks, max

    // Literals
    Identifier(String),
    StringLiteral(String),
    IntLiteral(i64),
    FloatLiteral(f64),
    BoolLiteral(bool),
    Nihil,        // nihil, kosong, null

    // Operators & Punctuation
    Equal,        // ==, =
    NotEqual,     // !=, <>
    LessThan,     // <
    LessEqual,    // <=
    GreaterThan,  // >
    GreaterEqual, // >=
    Plus,         // +
    Minus,        // -
    Star,         // *
    Slash,        // /
    Pipe,         // |
    Comma,        // ,
    Colon,        // :
    Dot,          // .
    LParen,       // (
    RParen,       // )
    LBrace,       // {
    RBrace,       // }
    LBracket,     // [
    RBracket,     // ]
    Semicolon,    // ;

    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub col: usize,
    pub lexeme: String,
}

impl Token {
    pub fn new(kind: TokenKind, line: usize, col: usize, lexeme: impl Into<String>) -> Self {
        Self {
            kind,
            line,
            col,
            lexeme: lexeme.into(),
        }
    }
}
