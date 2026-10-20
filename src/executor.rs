//! Executor Basis Data Langsung (Live Database Engine) untuk DQL
//!
//! Menjalankan kueri SQL hasil kompilasi DQL secara real-time ke database aktif
//! (SQLite, MySQL, PostgreSQL) melalui CLI driver lokal atau soket database.

use std::path::{Path, PathBuf};
use std::process::Command;
use crate::compiler::{compile, Dialect};

#[derive(Debug, Clone)]
pub enum LiveTarget {
    Sqlite {
        path: PathBuf,
    },
    Mysql {
        user: String,
        password: Option<String>,
        host: String,
        port: u16,
        database: String,
    },
    Postgres {
        user: String,
        host: String,
        port: u16,
        database: String,
    },
}

impl LiveTarget {
    pub fn parse(args: &[&str]) -> Result<Self, String> {
        if args.is_empty() {
            return Err("Format koneksi tidak lengkap. Contoh: ':sambung sqlite ./app.db'".to_string());
        }

        let first = args[0].to_lowercase();
        if first == "sqlite" {
            let path = if args.len() > 1 {
                args[1]
            } else {
                "dql_database.db"
            };
            return Ok(LiveTarget::Sqlite {
                path: PathBuf::from(path),
            });
        }

        if first.ends_with(".db") || first.ends_with(".sqlite") || first.ends_with(".sqlite3") {
            return Ok(LiveTarget::Sqlite {
                path: PathBuf::from(args[0]),
            });
        }

        if first == "mysql" {
            let mut user = "root".to_string();
            let mut password = None;
            let mut host = "localhost".to_string();
            let mut port = 3306;
            let mut database = "test".to_string();

            let mut i = 1;
            while i < args.len() {
                match args[i] {
                    "-u" if i + 1 < args.len() => {
                        user = args[i + 1].to_string();
                        i += 2;
                    }
                    "-p" if i + 1 < args.len() => {
                        password = Some(args[i + 1].to_string());
                        i += 2;
                    }
                    "-h" if i + 1 < args.len() => {
                        host = args[i + 1].to_string();
                        i += 2;
                    }
                    "-P" if i + 1 < args.len() => {
                        port = args[i + 1].parse().unwrap_or(3306);
                        i += 2;
                    }
                    "-d" if i + 1 < args.len() => {
                        database = args[i + 1].to_string();
                        i += 2;
                    }
                    other => {
                        if !other.starts_with('-') {
                            database = other.to_string();
                        }
                        i += 1;
                    }
                }
            }

            return Ok(LiveTarget::Mysql {
                user,
                password,
                host,
                port,
                database,
            });
        }

        if first == "postgres" || first == "pg" {
            let mut user = "postgres".to_string();
            let mut host = "localhost".to_string();
            let mut port = 5432;
            let mut database = "postgres".to_string();

            let mut i = 1;
            while i < args.len() {
                match args[i] {
                    "-U" | "-u" if i + 1 < args.len() => {
                        user = args[i + 1].to_string();
                        i += 2;
                    }
                    "-h" if i + 1 < args.len() => {
                        host = args[i + 1].to_string();
                        i += 2;
                    }
                    "-p" | "-P" if i + 1 < args.len() => {
                        port = args[i + 1].parse().unwrap_or(5432);
                        i += 2;
                    }
                    "-d" if i + 1 < args.len() => {
                        database = args[i + 1].to_string();
                        i += 2;
                    }
                    other => {
                        if !other.starts_with('-') {
                            database = other.to_string();
                        }
                        i += 1;
                    }
                }
            }

            return Ok(LiveTarget::Postgres {
                user,
                host,
                port,
                database,
            });
        }

        Err(format!(
            "Target database tidak didukung: '{}'. Pilihan: sqlite, mysql, postgres",
            args[0]
        ))
    }
}

#[derive(Debug, Clone)]
pub struct LiveExecutor {
    pub target: LiveTarget,
}

impl LiveExecutor {
    pub fn new(target: LiveTarget) -> Self {
        Self { target }
    }

    pub fn dialect(&self) -> Dialect {
        match &self.target {
            LiveTarget::Sqlite { .. } => Dialect::Sqlite,
            LiveTarget::Mysql { .. } => Dialect::Mysql,
            LiveTarget::Postgres { .. } => Dialect::Postgres,
        }
    }

    pub fn display_name(&self) -> String {
        match &self.target {
            LiveTarget::Sqlite { path } => {
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("database.db");
                format!("Sqlite({})", file_name)
            }
            LiveTarget::Mysql { user, host, database, .. } => {
                format!("Mysql({}@{} / {})", user, host, database)
            }
            LiveTarget::Postgres { user, host, database, .. } => {
                format!("Postgres({}@{} / {})", user, host, database)
            }
        }
    }

    /// Menjalankan SQL langsung ke database aktif
    pub fn execute_sql(&self, sql: &str) -> Result<String, String> {
        match &self.target {
            LiveTarget::Sqlite { path } => self.execute_sqlite(path, sql),
            LiveTarget::Mysql { user, password, host, port, database } => {
                self.execute_mysql(user, password.as_deref(), host, *port, database, sql)
            }
            LiveTarget::Postgres { user, host, port, database } => {
                self.execute_postgres(user, host, *port, database, sql)
            }
        }
    }

    /// Mengompilasi DQL lalu mengeksekusinya ke database aktif
    pub fn execute_dql(&self, dql_source: &str) -> Result<(String, String), String> {
        let compiled = compile(dql_source, self.dialect(), false)?;
        let output = self.execute_sql(&compiled.sql)?;
        Ok((compiled.sql, output))
    }

    fn execute_sqlite(&self, db_path: &Path, sql: &str) -> Result<String, String> {
        let path_str = db_path.to_string_lossy().to_string();
        let bin = find_executable(&[
            "sqlite3.exe",
            "sqlite3",
            "C:\\laragon\\bin\\laragon\\utils\\sqlite3.exe",
        ]);

        let mut cmd = Command::new(&bin);
        cmd.arg("-header").arg("-column").arg(&path_str).arg(sql);

        match cmd.output() {
            Ok(out) => {
                if !out.status.success() {
                    let err = String::from_utf8_lossy(&out.stderr);
                    return Err(format!("Kesalahan SQLite: {}", err.trim()));
                }
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let trimmed = stdout.trim();
                if trimmed.is_empty() {
                    Ok("Perintah berhasil dijalankan.".to_string())
                } else {
                    Ok(trimmed.to_string())
                }
            }
            Err(e) => Err(format!(
                "Gagal menjalankan klien sqlite3 ('{}'): {}. Pastikan 'sqlite3' terpasang.",
                bin, e
            )),
        }
    }

    fn execute_mysql(
        &self,
        user: &str,
        password: Option<&str>,
        host: &str,
        port: u16,
        database: &str,
        sql: &str,
    ) -> Result<String, String> {
        let bin = find_executable(&[
            "mysql.exe",
            "mysql",
            "C:\\laragon\\bin\\mysql\\mariadb-10.6.5-winx64\\bin\\mysql.exe",
        ]);

        let mut cmd = Command::new(&bin);
        cmd.arg("-u").arg(user);
        if let Some(pass) = password {
            if !pass.is_empty() {
                cmd.arg(format!("-p{}", pass));
            }
        }
        cmd.arg("-h").arg(host);
        cmd.arg("-P").arg(port.to_string());
        cmd.arg("-t"); // Output format tabel bergaris
        cmd.arg("-e").arg(sql);
        cmd.arg(database);

        match cmd.output() {
            Ok(out) => {
                if !out.status.success() {
                    let err = String::from_utf8_lossy(&out.stderr);
                    return Err(format!("Kesalahan MySQL: {}", err.trim()));
                }
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let trimmed = stdout.trim();
                if trimmed.is_empty() {
                    Ok("Perintah berhasil dijalankan.".to_string())
                } else {
                    Ok(trimmed.to_string())
                }
            }
            Err(e) => Err(format!(
                "Gagal menjalankan klien mysql ('{}'): {}. Pastikan server MySQL berjalan dan CLI 'mysql' terpasang.",
                bin, e
            )),
        }
    }

    fn execute_postgres(
        &self,
        user: &str,
        host: &str,
        port: u16,
        database: &str,
        sql: &str,
    ) -> Result<String, String> {
        let bin = find_executable(&["psql.exe", "psql"]);

        let mut cmd = Command::new(&bin);
        cmd.arg("-U").arg(user);
        cmd.arg("-h").arg(host);
        cmd.arg("-p").arg(port.to_string());
        cmd.arg("-d").arg(database);
        cmd.arg("-c").arg(sql);

        match cmd.output() {
            Ok(out) => {
                if !out.status.success() {
                    let err = String::from_utf8_lossy(&out.stderr);
                    return Err(format!("Kesalahan PostgreSQL: {}", err.trim()));
                }
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let trimmed = stdout.trim();
                if trimmed.is_empty() {
                    Ok("Perintah berhasil dijalankan.".to_string())
                } else {
                    Ok(trimmed.to_string())
                }
            }
            Err(e) => Err(format!(
                "Gagal menjalankan klien psql ('{}'): {}. Pastikan 'psql' terpasang.",
                bin, e
            )),
        }
    }
}

fn find_executable(candidates: &[&str]) -> String {
    for c in candidates {
        if Path::new(c).exists() {
            return c.to_string();
        }
    }
    candidates[0].to_string()
}
