mod categories;
mod products;
mod carts;

pub use categories::*;
pub use products::*;
pub use carts::*;

use colourful_logger::Logger;
use sea_orm::{Database, DatabaseConnection};

pub async fn establish_connection() -> DatabaseConnection {
    let logger = Logger::default();

    logger.info_single("🔌 Initializing database connection...", "DATABASE");

    // let database_url = env::var("DATABASE_URL")
    //     .expect("DATABASE_URL must be set in .env");

    let database_url = "postgresql://postgres.reknitbzbqqwnpnqzkfw:25ANMzrkD13FKAd6@aws-0-ap-southeast-1.pooler.supabase.com:5432/postgres";

    let conn = Database::connect(database_url)
        .await
        .expect("❌ Failed to connect to database");

    logger.info_single("✅ Database connected", "DATABASE");

    conn
}
