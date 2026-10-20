use std::env;
use std::fs;
use std::io::{self, Write};
use dql::{compile, Dialect, LiveExecutor, LiveTarget};

fn print_banner() {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║       DQL - DapCode Query Language Toolchain v0.2.0      ║");
    println!("║   Bahasa Kueri Mandiri Non-SQL & Live Database Engine    ║");
    println!("╚══════════════════════════════════════════════════════════╝");
}

fn print_help() {
    print_banner();
    println!(r#"
PENGGUNAAN:
    dql [PERINTAH] [OPSI...]

PERINTAH:
    compile "<kueri>"          Mengompilasi string DQL langsung ke SQL
    file <berkas.dql>          Membaca dan mengompilasi berkas .dql
    exec "<kueri>" [TARGET]    Eksekusi kueri DQL langsung ke database aktif
    live <TARGET...>           Masuk ke sesi interaktif REPL tersambung ke database
    repl                       Masuk ke sesi interaktif DQL REPL (compiler mode)
    help, --help, -h           Menampilkan pesan bantuan ini
    version, --version, -v     Menampilkan versi DQL

OPSI:
    -d, --dialect <nama>       Target dialek: postgres (default), sqlite, mysql
    -p, --params               Gunakan parameter binding ($1, ?), bukan inline literal

TARGET DATABASE LANGSUNG (LIVE):
    sqlite <berkas.db>         Koneksi ke berkas SQLite lokal
    mysql -u <user> -d <db>    Koneksi ke MySQL / MariaDB lokal
    postgres -U <user> -d <db> Koneksi ke PostgreSQL lokal

CONTOH:
    dql compile "buat tabel Pengguna {{ id: seri kunci_utama, nama: teks tidak_kosong }}"
    dql compile "ambil Pengguna di_mana status == 'aktif' batasi 5"
    dql exec "ambil Pengguna" sqlite ./app.db
    dql live sqlite ./database.db
    dql repl
"#);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        return;
    }

    match args[1].as_str() {
        "-h" | "--help" | "help" => {
            print_help();
        }
        "-v" | "--version" | "version" => {
            println!("DQL (DapCode Query Language) v0.2.0");
        }
        "repl" => {
            let dialect = parse_dialect_arg(&args, 2);
            let parameterized = args.iter().any(|a| a == "-p" || a == "--params");
            start_repl(dialect, parameterized, None);
        }
        "live" => {
            let target_args: Vec<&str> = args.iter().skip(2).map(|s| s.as_str()).collect();
            if target_args.is_empty() {
                eprintln!("Kesalahan: Masukkan target database untuk sesi live.");
                eprintln!("Contoh:");
                eprintln!("  dql live sqlite ./database.db");
                eprintln!("  dql live mysql -u root -d toko_db");
                eprintln!("  dql live postgres -U postgres -d toko_db");
                std::process::exit(1);
            }
            match LiveTarget::parse(&target_args) {
                Ok(target) => {
                    let executor = LiveExecutor::new(target);
                    let dialect = executor.dialect();
                    start_repl(dialect, false, Some(executor));
                }
                Err(e) => {
                    eprintln!("Kesalahan target database: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "exec" => {
            if args.len() < 3 {
                eprintln!("Kesalahan: Masukkan kueri DQL yang ingin dieksekusi.");
                eprintln!("Contoh: dql exec \"ambil Pengguna\" sqlite ./app.db");
                std::process::exit(1);
            }
            let query = &args[2];
            let target_args: Vec<&str> = args.iter().skip(3).map(|s| s.as_str()).collect();
            let target = if target_args.is_empty() {
                LiveTarget::Sqlite {
                    path: std::path::PathBuf::from("dql_database.db"),
                }
            } else {
                match LiveTarget::parse(&target_args) {
                    Ok(t) => t,
                    Err(e) => {
                        eprintln!("Kesalahan target database: {}", e);
                        std::process::exit(1);
                    }
                }
            };
            let executor = LiveExecutor::new(target);
            println!("\n-- [Target Langsung: {}] --", executor.display_name());
            match executor.execute_dql(query) {
                Ok((sql, out)) => {
                    println!("--> SQL: {}", sql);
                    println!("\nHASIL:");
                    println!("{}", out);
                }
                Err(e) => {
                    eprintln!("\n{}", e);
                    std::process::exit(1);
                }
            }
        }
        "compile" => {
            if args.len() < 3 {
                eprintln!("Kesalahan: Masukkan kueri DQL yang ingin dikompilasi.");
                eprintln!("Contoh: dql compile \"ambil Pengguna di_mana umur >= 18\"");
                std::process::exit(1);
            }

            let dialect = parse_dialect_arg(&args, 2);
            let parameterized = args.iter().any(|a| a == "-p" || a == "--params");

            let mut query = String::new();
            let mut i = 2;
            while i < args.len() {
                if args[i] == "-d" || args[i] == "--dialect" {
                    i += 2;
                } else if args[i] == "-p" || args[i] == "--params" {
                    i += 1;
                } else {
                    query = args[i].clone();
                    break;
                }
            }

            if query.is_empty() {
                eprintln!("Kesalahan: Tidak ada kueri DQL yang diberikan.");
                std::process::exit(1);
            }

            match compile(&query, dialect, parameterized) {
                Ok(res) => {
                    println!("\n-- [Target: {:?}] --", dialect);
                    println!("{}", res.sql);
                    if parameterized && !res.params.is_empty() {
                        println!("-- Parameters: {:?}", res.params);
                    }
                }
                Err(err) => {
                    eprintln!("\n{}", err);
                    std::process::exit(1);
                }
            }
        }
        "file" => {
            if args.len() < 3 {
                eprintln!("Kesalahan: Masukkan nama berkas .dql.");
                std::process::exit(1);
            }

            let file_path = &args[2];
            let dialect = parse_dialect_arg(&args, 3);
            let parameterized = args.iter().any(|a| a == "-p" || a == "--params");

            match fs::read_to_string(file_path) {
                Ok(content) => match compile(&content, dialect, parameterized) {
                    Ok(res) => {
                        println!("\n-- Berkas: {} | Target: {:?} --", file_path, dialect);
                        println!("{}", res.sql);
                        if parameterized && !res.params.is_empty() {
                            println!("-- Parameters: {:?}", res.params);
                        }
                    }
                    Err(err) => {
                        eprintln!("\nKesalahan kompilasi pada '{}':\n{}", file_path, err);
                        std::process::exit(1);
                    }
                },
                Err(e) => {
                    eprintln!("Kesalahan membaca berkas '{}': {}", file_path, e);
                    std::process::exit(1);
                }
            }
        }
        unknown => {
            eprintln!("Perintah tidak dikenal: '{}'", unknown);
            print_help();
            std::process::exit(1);
        }
    }
}

fn parse_dialect_arg(args: &[String], start: usize) -> Dialect {
    for i in start..args.len() {
        if (args[i] == "-d" || args[i] == "--dialect") && i + 1 < args.len() {
            if let Some(d) = Dialect::parse(&args[i + 1]) {
                return d;
            }
        }
    }
    Dialect::Postgres
}

fn start_repl(mut current_dialect: Dialect, mut parameterized: bool, mut live_executor: Option<LiveExecutor>) {
    print_banner();
    println!("Sesi Interaktif REPL DQL (Ketik 'keluar' atau 'exit' untuk berhenti)");
    println!("Perintah khusus:");
    println!("  :sambung <sqlite|mysql|pg> [opsi...]  Sambungkan sesi langsung ke database");
    println!("  :putus                                Putus koneksi live & kembali ke compiler");
    println!("  :status                               Periksa status koneksi aktif");
    println!("  :dialek <pg|sqlite|mysql>             Ganti dialek target");
    println!("  :param <on|off>                       Aktifkan parameter binding");
    println!("  :bantuan                              Bantuan perintah\n");

    if let Some(executor) = &live_executor {
        println!(">>> Mode Live Aktif: Tersambung ke {}\n", executor.display_name());
    }

    let stdin = io::stdin();
    let mut buffer = String::new();

    loop {
        if let Some(executor) = &live_executor {
            print!("DQL [Live: {}]> ", executor.display_name());
        } else {
            print!("DQL [{:?}]> ", current_dialect);
        }
        io::stdout().flush().unwrap();

        buffer.clear();
        if stdin.read_line(&mut buffer).is_err() {
            break;
        }

        let input = buffer.trim();
        if input.is_empty() {
            continue;
        }

        if input.eq_ignore_ascii_case("keluar") || input.eq_ignore_ascii_case("exit") {
            println!("Sampai jumpa!");
            break;
        }

        if input.starts_with(':') {
            let parts: Vec<&str> = input.split_whitespace().collect();
            match parts[0] {
                ":sambung" => {
                    if parts.len() < 2 {
                        println!("Format: :sambung <sqlite|mysql|postgres> [argumen...]");
                        println!("Contoh: :sambung sqlite ./app.db");
                        println!("Contoh: :sambung mysql -u root -d toko_db");
                        continue;
                    }
                    match LiveTarget::parse(&parts[1..]) {
                        Ok(target) => {
                            let executor = LiveExecutor::new(target);
                            current_dialect = executor.dialect();
                            println!("Berhasil tersambung ke: {}", executor.display_name());
                            live_executor = Some(executor);
                        }
                        Err(e) => println!("Gagal menyambungkan: {}", e),
                    }
                }
                ":putus" => {
                    if live_executor.is_some() {
                        live_executor = None;
                        println!("Koneksi live diputus. Kembali ke mode kompilasi kueri.");
                    } else {
                        println!("Tidak ada koneksi live yang aktif.");
                    }
                }
                ":status" => {
                    if let Some(executor) = &live_executor {
                        println!("Status: Tersambung secara LIVE ke {}", executor.display_name());
                        println!("Dialek aktif: {:?}", executor.dialect());
                    } else {
                        println!("Status: Mode Kompiler DQL (Offline)");
                        println!("Dialek aktif: {:?}", current_dialect);
                        println!("Parameter binding: {}", parameterized);
                    }
                }
                ":dialek" => {
                    if parts.len() > 1 {
                        if let Some(d) = Dialect::parse(parts[1]) {
                            current_dialect = d;
                            println!("Dialek diubah menjadi: {:?}", current_dialect);
                        } else {
                            println!("Dialek tidak valid. Pilihan: postgres, sqlite, mysql");
                        }
                    } else {
                        println!("Dialek saat ini: {:?}", current_dialect);
                    }
                }
                ":param" => {
                    if parts.len() > 1 {
                        parameterized = parts[1] == "on" || parts[1] == "benar";
                        println!("Parameter binding: {}", parameterized);
                    } else {
                        println!("Status parameter binding: {}", parameterized);
                    }
                }
                ":bantuan" => {
                    println!("Perintah REPL:");
                    println!("  :sambung <sqlite|mysql|pg> [opsi]  Sambungkan ke database live");
                    println!("  :putus                             Lepas koneksi live");
                    println!("  :status                            Cek status koneksi");
                    println!("  :dialek <pg|sqlite|mysql>          Ganti target dialek SQL");
                    println!("  :param <on|off>                    Toggle parameter binding");
                    println!("  keluar / exit                      Keluar dari REPL");
                }
                _ => println!("Perintah REPL tidak dikenal: {}", parts[0]),
            }
            continue;
        }

        // Jika terhubung ke database live, eksekusi langsung!
        if let Some(executor) = &live_executor {
            match executor.execute_dql(input) {
                Ok((sql, output)) => {
                    println!("--> SQL: {}", sql);
                    println!("{}", output);
                }
                Err(err) => {
                    eprintln!("{}", err);
                }
            }
            println!();
            continue;
        }

        // Mode standar: compile
        match compile(input, current_dialect, parameterized) {
            Ok(res) => {
                println!("SQL: {}", res.sql);
                if parameterized && !res.params.is_empty() {
                    println!("PARAMS: {:?}", res.params);
                }
            }
            Err(err) => {
                eprintln!("{}", err);
            }
        }
        println!();
    }
}
