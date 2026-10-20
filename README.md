# DQL (DapCode Query Language)

**DQL (DapCode Query Language)** adalah bahasa kueri mandiri (*standalone query language*) yang modern, aman, dan ergonomis untuk berinteraksi dengan database tanpa perlu menulis SQL manual secara mentah.

DQL dirancang untuk menjadi bahasa tingkat tinggi yang dapat digunakan secara universal:
- **Di dalam ekosistem DapCode** (`.dap`)
- **Di berbagai bahasa pemrograman lain** (Python, JavaScript/TypeScript, Go, Rust, C#, PHP) melalui modul library native Rust dan C-ABI FFI (`dql.dll` / `.so`).

---

## 🌟 Fitur Utama DQL

1. **Sintaks Alami & Bersih (Tanpa Titik Koma):**
   - Menggunakan kata kunci deklaratif berbahasa Indonesia yang ekspresif:
     - `ambil` / `dari` (SELECT / FROM)
     - `pilih` (SELECT fields) & `sebagai` (AS alias)
     - `di_mana` (WHERE filter)
     - `gabung` (JOIN) ... `pada` (ON)
     - `kelompokkan` (GROUP BY)
     - `memiliki` (HAVING)
     - `urutkan` (ORDER BY) `naik`/`turun` (ASC/DESC)
     - `batasi` (LIMIT) & `lewati` (OFFSET)
     - `tambah_ke` (INSERT)
     - `perbarui` (UPDATE) ... `atur` (SET)
     - `hapus_dari` (DELETE)
     - `buat tabel` / `hapus tabel` (DDL CREATE TABLE / DROP TABLE)
     - `tampilkan tabel` & `struktur` (SHOW TABLES & DESCRIBE)

2. **Dukungan Multi-Dialek Database:**
   - **PostgreSQL**: Standard identifier quoting (`"name"`), `$1`, `$2` parameter binding, boolean `TRUE`/`FALSE`.
   - **SQLite**: Standard table quoting, `?` / `?1` parameter binding, numeric boolean (`1`/`0`).
   - **MySQL / MariaDB**: Backtick identifier quoting (`` `name` ``), `?` parameter binding.

3. **Live Database Execution Engine:**
   - Eksekusi kueri DQL langsung ke database fisik aktif (`SQLite`, `MySQL`, `PostgreSQL`) via terminal tanpa perlu menulis SQL manual!
   - Mode REPL Live interaktif (`dql live`) dengan format tabel ASCII otomatis.

4. **Perlindungan Terhadap SQL Injection (Parameterized Query):**
   - Mendukung mode parameter binding otomatis (`-p`), memisahkan logika kueri dari nilai data untuk keamanan tingkat tinggi.

5. **Multi-Bahasa Melalui C-ABI FFI:**
   - Diimplementasikan dalam Rust, menghasilkan pustaka dinamis C-ABI (`dql.dll`, `libdql.so`, `libdql.dylib`) yang bisa langsung dipanggil oleh Python (`ctypes`), Node.js, Go (`cgo`), dan bahasa lainnya.

---

## 🚀 Penggunaan CLI Tool (`dql`)

Kompilasi kueri atau kelola database langsung dari terminal:

```bash
# 1. Membuat tabel baru (DDL)
dql compile "buat tabel Pengguna { id: seri kunci_utama, nama: teks tidak_kosong, umur: bulat bawaan 18 }"

# 2. Eksekusi DQL langsung ke database aktif (Live Execution)
dql exec "buat tabel jika_belum_ada Pengguna { id: seri kunci_utama, nama: teks, umur: bulat }" sqlite ./database.db
dql exec "tambah_ke Pengguna { nama: 'Daffa', umur: 22 }" sqlite ./database.db
dql exec "ambil Pengguna" sqlite ./database.db

# 3. Sesi Interaktif REPL Terhubung Langsung ke Database (Live REPL)
dql live sqlite ./database.db
dql live mysql -u root -d toko_db
dql live postgres -U postgres -d toko_db

# 4. Sesi Interaktif REPL Compiler Mode
dql repl
```

---

## 🐍 Penggunaan dari Python

DQL dapat dipanggil langsung dari Python tanpa perlu compiler di sisi Python:

```python
import ctypes
import sqlite3

# Muat library DQL yang sudah terkompilasi
dql = ctypes.CDLL("./target/release/dql.dll")
dql.dql_compile.argtypes = [ctypes.c_char_p, ctypes.c_char_p, ctypes.c_bool, ctypes.POINTER(ctypes.c_char_p)]
dql.dql_compile.restype = ctypes.c_void_p
dql.dql_free_string.argtypes = [ctypes.c_void_p]

def compile_dql(query: str, dialect: str = "sqlite") -> str:
    err_ptr = ctypes.c_char_p()
    addr = dql.dql_compile(query.encode("utf-8"), dialect.encode("utf-8"), False, ctypes.byref(err_ptr))
    if not addr:
        raise RuntimeError(err_ptr.value.decode("utf-8"))
    sql = ctypes.string_at(addr).decode("utf-8")
    dql.dql_free_string(addr)
    return sql

# Gunakan DQL untuk query SQLite!
kueri_dql = "ambil Pengguna di_mana umur >= 20 urutkan umur turun batasi 5"
sql = compile_dql(kueri_dql, dialect="sqlite")

conn = sqlite3.connect("database.db")
cursor = conn.cursor()
cursor.execute(sql)
print(cursor.fetchall())
```

---

## ⚡ Penggunaan dari Node.js / JavaScript

```javascript
const { execFileSync } = require("child_process");

function compileDQL(query, dialect = "postgres") {
    return execFileSync("./target/release/dql.exe", ["compile", "-d", dialect, query], {
        encoding: "utf-8"
    });
}

const sql = compileDQL("ambil Produk di_mana stok > 0 urutkan harga naik batasi 10");
console.log(sql);
```

---

## 📁 Struktur Repositori

```
dapcode query language/
├── Cargo.toml            # Manifes Rust (lib, cdylib, bin)
├── README.md             # Dokumentasi lengkap DQL
├── src/
│   ├── lib.rs            # Public API library DQL
│   ├── main.rs           # CLI Tool & Interactive REPL
│   ├── token.rs          # Token definitions
│   ├── lexer.rs          # Lexer & scanner dengan dukungan UTF-8 BOM
│   ├── ast.rs            # Abstract Syntax Tree nodes
│   ├── parser.rs         # Recursive descent parser
│   ├── ffi.rs            # C-ABI export (dql_compile, dql_free_string)
│   └── compiler/
│       ├── mod.rs        # Dialect enum & dispatcher
│       ├── postgres.rs   # PostgreSQL code generator
│       ├── sqlite.rs     # SQLite code generator
│       └── mysql.rs      # MySQL / MariaDB code generator
├── examples/
│   ├── contoh.dql        # Contoh kueri DQL lengkap
│   ├── python_demo.py    # Demo nyata Python + SQLite + DQL
│   └── node_demo.js      # Demo Node.js + DQL
└── tests/
    └── dql_tests.rs      # Unit & integration tests
```

---

## 🧪 Menjalankan Pengujian

```bash
cargo test
```