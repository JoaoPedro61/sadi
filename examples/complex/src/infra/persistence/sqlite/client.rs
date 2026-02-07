use std::sync::Mutex;

pub struct SqliteClient {
    migrated: bool,
    connection: Mutex<sqlite::Connection>,
}

impl std::fmt::Display for SqliteClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SqliteClient {{ migrated: {} }}", self.migrated)
    }
}

impl SqliteClient {
    pub fn new() -> Result<Self, String> {
        let connection = sqlite::open(":memory:").map_err(|e| e.to_string())?;
        Ok(Self {
            migrated: true,
            connection: Mutex::new(connection),
        })
    }

    pub fn is_migrated(&self) -> bool {
        self.migrated
    }

    pub fn connection(&self) -> &Mutex<sqlite::Connection> {
        &self.connection
    }
}
