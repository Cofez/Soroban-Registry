use std::time::Instant;

use sqlx::PgPool;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub started_at: Instant,
}

impl AppState {
    pub fn new(db: PgPool) -> Self {
        Self {
            db,
            started_at: Instant::now(),
        }
    }
}
