use serve::infrastructure::config::AppConfig;
use serve::infrastructure::db::postgres::Database;
use serve::infrastructure::logging::init_logging;

use tracing::info;

#[tokio::main]
async fn main() {
    // Initialize logging
    init_logging();

    println!("============================================================");
    println!("⚡️ Serve Database Seeder");
    println!("============================================================");

    // Load environment config
    let config = AppConfig::from_env();
    info!(
        target: "serve::seed",
        driver = %config.database.driver.as_str(),
        url = %config.database.url,
        "Connecting to database..."
    );

    // 3. Connect to database
    let db = Database::connect(&config.database)
        .await
        .expect("Failed to connect to database");

    // 4. Run schema migrations
    info!(target: "serve::seed", "Checking and applying schema migrations...");
    db.init_tables()
        .await
        .expect("Failed to initialize database tables");

    // 5. Seed dummy / test data
    info!(target: "serve::seed", "Inserting dummy users, posts, comments, stories, and follows...");
    db.seed_test_data()
        .await
        .expect("Failed to seed database test data");

    println!("============================================================");
    println!("✅ Database seeding complete!");
    println!("   - Default Accounts:");
    println!("     1. ferro_dev (ferro@example.com / password123)");
    println!("     2. alex_coder (alex@example.com / password123)");
    println!("     3. design_guru (sophia@example.com / password123)");
    println!("============================================================");
}
