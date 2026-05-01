use rusqlite::Connection;
pub mod dedup; pub mod schema; pub mod symbols;

pub fn open(path: &str) -> anyhow::Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    Ok(conn)
}
pub fn migrate(conn: &Connection) -> anyhow::Result<()> { schema::run(conn) }