use rusqlite::Connection;
use async_channel::Sender;

type SqlJob = Box<dyn FnOnce(&mut Connection) + Send>;

pub struct SqlActor {
    tx: Sender<SqlJob>,
}

impl Default for SqlActor {
    fn default() -> Self {
        let (tx, rx) = async_channel::unbounded::<SqlJob>();
        
        std::thread::Builder::new()
            .name("gray-meadows-sql".into())
            .spawn(move || {
                let state_dir = crate::utils::filesystem::get_local_state_directory();
                let db_path = format!("{}/sqlite.db", state_dir);
            
                if !std::path::Path::new(&state_dir).exists() {
                    std::fs::create_dir_all(&state_dir).expect("Failed to create state directory");
                }
                
                let mut conn = Connection::open(&db_path)
                    .expect("Failed to open database connection");
        
                let _ = conn.execute_batch("
                    PRAGMA journal_mode = WAL;
                    PRAGMA wal_autocheckpoint = 500;
                    PRAGMA synchronous = NORMAL;
                    PRAGMA foreign_keys = ON;
                    PRAGMA busy_timeout = 5000;
                ");
        
                while let Ok(job) = rx.recv_blocking() {
                    job(&mut conn);
                }
            })
            .expect("Failed to spawn DB thread");
        
        Self { tx }
    }
}

impl SqlActor {
    /// Returns the result of a SQL operation.
    pub async fn with<F, R>(&self, func: F) -> anyhow::Result<R>
    where
        F: FnOnce(&mut Connection) -> R + Send + 'static,
        R: Send + 'static,
    {
        let (resp_tx, resp_rx) = async_channel::bounded(1);
        let job = Box::new(move |conn: &mut Connection| {
            let result = func(conn);
            let _ = resp_tx.send_blocking(result);
        });
    
        self.tx.send(job)
            .await
            .map_err(|_| anyhow::anyhow!("DB thread dead"))?;
    
        resp_rx.recv()
            .await
            .map_err(|_| anyhow::anyhow!("DB query cancelled"))
    }
}