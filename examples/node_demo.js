/**
 * Demo Pemanggilan DQL dari Node.js / JavaScript.
 * Menggunakan binary DQL untuk mentranspilasi kueri DQL ke SQL.
 */

const { execFileSync } = require("child_process");
const path = require("path");

const dqlBinary = path.join(__dirname, "..", "target", "release", "dql.exe");

function compileDQL(query, dialect = "postgres", parameterized = false) {
    const args = ["compile", "-d", dialect];
    if (parameterized) {
        args.push("-p");
    }
    args.push(query);

    const output = execFileSync(dqlBinary, args, { encoding: "utf-8" });
    return output.trim();
}

console.log("==========================================================");
console.log("   DEMO DQL (DapCode Query Language) di Node.js           ");
console.log("==========================================================");

const query = `
ambil Pesanan sebagai o
pilih o.id, o.total, p.nama sebagai nama_pelanggan
gabung Pengguna sebagai p pada o.pengguna_id == p.id
di_mana o.total > 150000 dan o.status == 'dibayar'
urutkan o.total turun
batasi 5
`;

console.log("\n[Kueri Asli DQL]:", query.trim());

const hasilSQL = compileDQL(query, "postgres");
console.log("\n[Hasil Kompilasi ke PostgreSQL]:\n", hasilSQL);

const hasilMySQL = compileDQL(query, "mysql");
console.log("\n[Hasil Kompilasi ke MySQL]:\n", hasilMySQL);

console.log("\n[SUKSES] Node.js berhasil mengintegrasikan DQL!");
