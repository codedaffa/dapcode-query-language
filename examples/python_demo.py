"""
Demo Pemanggilan DQL dari Bahasa Python menggunakan C-ABI (ctypes).
Membuktikan bahwa DQL dapat digunakan secara mandiri oleh Python tanpa menulis SQL manual.
"""

import os
import sys
import ctypes
import sqlite3
if sys.platform == 'win32':
    try:
        sys.stdout.reconfigure(encoding='utf-8')
    except Exception:
        pass

# Jalur ke dql.dll
dll_path = os.path.join(os.path.dirname(__file__), "..", "target", "release", "dql.dll")
if not os.path.exists(dll_path):
    print(f"Error: berkas library tidak ditemukan di {dll_path}")
    sys.exit(1)

# Muat library dinamis Rust DQL
lib = ctypes.CDLL(dll_path)

# Konfigurasi tipe argumen dan return value C-ABI
lib.dql_compile.argtypes = [
    ctypes.c_char_p,
    ctypes.c_char_p,
    ctypes.c_bool,
    ctypes.POINTER(ctypes.c_char_p)
]
lib.dql_compile.restype = ctypes.c_void_p

lib.dql_free_string.argtypes = [ctypes.c_void_p]
lib.dql_free_string.restype = None

def compile_dql(query: str, dialect: str = "sqlite") -> str:
    err_ptr = ctypes.c_char_p()
    res_addr = lib.dql_compile(
        query.encode("utf-8"),
        dialect.encode("utf-8"),
        False,
        ctypes.byref(err_ptr)
    )
    if not res_addr:
        msg = err_ptr.value.decode("utf-8") if err_ptr.value else "Unknown DQL error"
        raise RuntimeError(f"Kesalahan DQL: {msg}")
    
    sql = ctypes.string_at(res_addr).decode("utf-8")
    lib.dql_free_string(res_addr)
    return sql

def main():
    print("==========================================================")
    print("   DEMO DQL (DapCode Query Language) di Python 3          ")
    print("==========================================================")

    # 1. Buat database SQLite in-memory untuk pengujian nyata
    conn = sqlite3.connect(":memory:")
    cursor = conn.cursor()

    # Siapkan skema database
    cursor.execute("""
        CREATE TABLE "Pengguna" (
            "id" INTEGER PRIMARY KEY AUTOINCREMENT,
            "nama" TEXT NOT NULL,
            "email" TEXT NOT NULL,
            "umur" INTEGER NOT NULL,
            "status" TEXT NOT NULL
        )
    """)

    # 2. Sisipkan data menggunakan DQL
    dql_insert = """
    tambah_ke Pengguna [
        { nama: 'Ahmad Dahlan', email: 'ahmad@mail.com', umur: 28, status: 'aktif' },
        { nama: 'Siti Rahma', email: 'siti@mail.com', umur: 22, status: 'aktif' },
        { nama: 'Doni Prasetyo', email: 'doni@mail.com', umur: 17, status: 'pending' },
        { nama: 'Maya Putri', email: 'maya@mail.com', umur: 31, status: 'aktif' }
    ]
    """
    print("\n[1] Menjalankan Kueri Tambah (Insert) DQL:")
    print(dql_insert.strip())

    sql_insert = compile_dql(dql_insert, dialect="sqlite")
    print(f"\n--> Dikompilasi oleh DQL Engine ke SQLite SQL:\n{sql_insert}")
    cursor.execute(sql_insert)
    conn.commit()

    # 3. Kueri data pengguna aktif berumur >= 20 tahun
    dql_select = """
    ambil Pengguna
    pilih id, nama, email, umur
    di_mana status == 'aktif' dan umur >= 20
    urutkan umur turun
    batasi 10
    """
    print("\n[2] Menjalankan Kueri Ambil (Select) DQL:")
    print(dql_select.strip())

    sql_select = compile_dql(dql_select, dialect="sqlite")
    print(f"\n--> Dikompilasi oleh DQL Engine ke SQLite SQL:\n{sql_select}")

    cursor.execute(sql_select)
    baris = cursor.fetchall()

    print("\n--> Hasil Data dari Database (dieksekusi di Python):")
    for r in baris:
        print(f"    - ID: {r[0]} | Nama: {r[1]} | Email: {r[2]} | Umur: {r[3]}")

    print("\n==========================================================")
    print("âœ… Integrasi Python <-> DQL Berhasil 100%!")
    print("==========================================================")

if __name__ == "__main__":
    main()