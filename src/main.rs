use crate::components::env_reader;
use crate::services::db_migrator::DbMigrator;
use dotenvy::dotenv;
use env_logger::Env;
use log::{error, info};

mod components;
mod config;
mod services;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let app_config = env_reader::read_server_config().await;

    if app_config.display_header {
        info!(
            r#"
/**
 *    ___  ___                       ____________             _                 _   _               _____           _
 *    |  \/  |                       |  _  \ ___ \           (_)               | | (_)             |_   _|         | |
 *    | .  . | ___  _ __   __ _  ___ | | | | |_/ /  _ __ ___  _  __ _ _ __ __ _| |_ _  ___  _ __     | | ___   ___ | |
 *    | |\/| |/ _ \| '_ \ / _` |/ _ \| | | | ___ \ | '_ ` _ \| |/ _` | '__/ _` | __| |/ _ \| '_ \    | |/ _ \ / _ \| |
 *    | |  | | (_) | | | | (_| | (_) | |/ /| |_/ / | | | | | | | (_| | | | (_| | |_| | (_) | | | |   | | (_) | (_) | |
 *    \_|  |_/\___/|_| |_|\__, |\___/|___/ \____/  |_| |_| |_|_|\__, |_|  \__,_|\__|_|\___/|_| |_|   \_/\___/ \___/|_|
 *                         __/ |                                 __/ |
 *                        |___/                                 |___/
 *    ______         _____           _     ______               _
 *    | ___ \       /  __ \         | |    |  _  \             | |
 *    | |_/ /_   _  | /  \/ ___   __| | ___| | | |___  __ _  __| |
 *    | ___ \ | | | | |    / _ \ / _` |/ _ \ | | / _ \/ _` |/ _` |
 *    | |_/ / |_| | | \__/\ (_) | (_| |  __/ |/ /  __/ (_| | (_| |
 *    \____/ \__, |  \____/\___/ \__,_|\___|___/ \___|\__,_|\__,_|
 *            __/ |
 *           |___/
 */"#
        );
    }

    let migrator = DbMigrator::new(app_config);
    match migrator.migrate_db().await {
        Ok(()) => {
            info!("Database migration completed successfully");
            Ok(())
        }
        Err(err) => {
            error!("Database migration failed: {}", err);
            Err(err)
        }
    }
}
