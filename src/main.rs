use crate::components::env_reader;
use crate::services::db_migrator::DbMigrator;
use dotenvy::dotenv;
use env_logger::Env;
use log::{error, info};
use std::env;

mod components;
mod config;
mod services;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    // General
    let display_header = match env::var("DISPLAY_HEADER") {
        Ok(d) => {
            let res: bool = d.trim().parse().unwrap_or(true);
            res
        }
        Err(_) => true,
    };

    if display_header {
        println!(
            r#"
  __  __                         ____  ____              _                 _   _               _              _
 |  \/  | ___  _ __   __ _  ___ |  _ \| __ )   _ __ ___ (_) __ _ _ __ __ _| |_(_) ___  _ __   | |_ ___   ___ | |
 | |\/| |/ _ \| '_ \ / _` |/ _ \| | | |  _ \  | '_ ` _ \| |/ _` | '__/ _` | __| |/ _ \| '_ \  | __/ _ \ / _ \| |
 | |  | | (_) | | | | (_| | (_) | |_| | |_) | | | | | | | | (_| | | | (_| | |_| | (_) | | | | | || (_) | (_) | |
 |_|  |_|\___/|_| |_|\__, |\___/|____/|____/  |_| |_| |_|_|\__, |_|  \__,_|\__|_|\___/|_| |_|  \__\___/ \___/|_|
  ____           ____|___/     _      ____                 |___/
 | __ ) _   _   / ___|___   __| | ___|  _ \  ___  __ _  __| |
 |  _ \| | | | | |   / _ \ / _` |/ _ \ | | |/ _ \/ _` |/ _` |
 | |_) | |_| | | |__| (_) | (_| |  __/ |_| |  __/ (_| | (_| |
 |____/ \__, |  \____\___/ \__,_|\___|____/ \___|\__,_|\__,_|
        |___/                                                                                                   "#
        );
    }

    let app_config = env_reader::read_server_config().await;

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
