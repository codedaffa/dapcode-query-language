use dql::{compile, Dialect, LiteralValue};

#[test]
fn test_select_basic() {
    let q = "ambil Pengguna di_mana umur >= 18 urutkan nama naik batasi 5";
    let res = compile(q, Dialect::Postgres, false).unwrap();
    assert_eq!(res.sql, "SELECT * FROM \"Pengguna\" WHERE (\"umur\" >= 18) ORDER BY \"nama\" ASC LIMIT 5;");
}

#[test]
fn test_select_sqlite_parameterized() {
    let q = "ambil Produk pilih id, nama di_mana harga < 10000";
    let res = compile(q, Dialect::Sqlite, true).unwrap();
    assert_eq!(res.sql, "SELECT \"id\", \"nama\" FROM \"Produk\" WHERE (\"harga\" < ?1);");
    assert_eq!(res.params, vec![LiteralValue::Int(10000)]);
}

#[test]
fn test_mysql_join() {
    let q = "ambil Pesanan sebagai o pilih o.id, u.nama gabung Pengguna sebagai u pada o.uid == u.id";
    let res = compile(q, Dialect::Mysql, false).unwrap();
    assert_eq!(res.sql, "SELECT `o`.`id`, `u`.`nama` FROM `Pesanan` AS `o` INNER JOIN `Pengguna` AS `u` ON (`o`.`uid` = `u`.`id`);");
}

#[test]
fn test_insert_mutation() {
    let q = "tambah_ke Pengguna { nama: 'Budi', umur: 25 }";
    let res = compile(q, Dialect::Postgres, false).unwrap();
    assert_eq!(res.sql, "INSERT INTO \"Pengguna\" (\"nama\", \"umur\") VALUES ('Budi', 25);");
}

#[test]
fn test_update_mutation() {
    let q = "perbarui Pengguna atur status = 'nonaktif' di_mana id == 1";
    let res = compile(q, Dialect::Postgres, false).unwrap();
    assert_eq!(res.sql, "UPDATE \"Pengguna\" SET \"status\" = 'nonaktif' WHERE (\"id\" = 1);");
}

#[test]
fn test_delete_mutation() {
    let q = "hapus_dari Pengguna di_mana status == 'banned'";
    let res = compile(q, Dialect::Postgres, false).unwrap();
    assert_eq!(res.sql, "DELETE FROM \"Pengguna\" WHERE (\"status\" = 'banned');");
}

#[test]
fn test_create_table_ddl_postgres() {
    let q = "buat tabel jika_belum_ada Pengguna { id: seri kunci_utama, nama: teks tidak_kosong, umur: bulat bawaan 18 }";
    let res = compile(q, Dialect::Postgres, false).unwrap();
    assert_eq!(
        res.sql,
        "CREATE TABLE IF NOT EXISTS \"Pengguna\" (\"id\" SERIAL PRIMARY KEY, \"nama\" TEXT NOT NULL, \"umur\" INTEGER DEFAULT 18);"
    );
}

#[test]
fn test_create_table_ddl_mysql() {
    let q = "buat tabel Pengguna { id: seri kunci_utama, nama: teks tidak_kosong, email: teks unik }";
    let res = compile(q, Dialect::Mysql, false).unwrap();
    assert_eq!(
        res.sql,
        "CREATE TABLE `Pengguna` (`id` INT AUTO_INCREMENT PRIMARY KEY, `nama` VARCHAR(255) NOT NULL, `email` VARCHAR(255) UNIQUE);"
    );
}

#[test]
fn test_create_table_ddl_sqlite() {
    let q = "buat tabel jika_belum_ada Pengguna { id: seri kunci_utama, nama: teks tidak_kosong, aktif: boolean bawaan 1 }";
    let res = compile(q, Dialect::Sqlite, false).unwrap();
    assert_eq!(
        res.sql,
        "CREATE TABLE IF NOT EXISTS \"Pengguna\" (\"id\" INTEGER PRIMARY KEY AUTOINCREMENT, \"nama\" TEXT NOT NULL, \"aktif\" INTEGER DEFAULT 1);"
    );
}

#[test]
fn test_drop_table_ddl() {
    let q = "hapus tabel jika_ada Pengguna";
    let res = compile(q, Dialect::Sqlite, false).unwrap();
    assert_eq!(res.sql, "DROP TABLE IF EXISTS \"Pengguna\";");
}

#[test]
fn test_show_tables_and_structure() {
    let q1 = "tampilkan tabel";
    let res1 = compile(q1, Dialect::Mysql, false).unwrap();
    assert_eq!(res1.sql, "SHOW TABLES;");

    let q2 = "struktur Pengguna";
    let res2 = compile(q2, Dialect::Sqlite, false).unwrap();
    assert_eq!(res2.sql, "PRAGMA table_info(\"Pengguna\");");
}