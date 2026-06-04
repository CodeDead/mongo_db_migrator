use crate::config::app_config::AppConfig;
use log::info;
use mongodb::Client;
use std::env;

pub struct EnvReader {}

impl EnvReader {
    /// Initialize a new EnvReader
    ///
    /// # Returns
    ///
    /// A new instance of EnvReader
    pub fn new() -> Self {
        EnvReader {}
    }

    /// Reads the server configuration from environment variables
    ///
    /// # Returns
    ///
    /// A new instance of AppConfig
    pub async fn read_server_config(&self) -> AppConfig {
        info!("Reading configuration from environment variables");

        // General
        let display_header = match env::var("DISPLAY_HEADER") {
            Ok(d) => {
                let res: bool = d.trim().parse().unwrap_or(true);
                res
            }
            Err(_) => false,
        };

        let copy_indices = match env::var("COPY_INDICES") {
            Ok(d) => {
                let res: bool = d.trim().parse().unwrap_or(false);
                res
            }
            Err(e) => panic!("COPY_INDICES has not been specified: {}", e),
        };

        let read_to_memory = match env::var("READ_TO_MEMORY") {
            Ok(d) => {
                let res: bool = d.trim().parse().unwrap_or(false);
                res
            }
            Err(_) => false,
        };

        let mongodb_origin_connection_string = match env::var("MONGODB_ORIGIN_CONNECTION_STRING") {
            Ok(d) => d,
            Err(_) => panic!("MONGODB_ORIGIN_CONNECTION_STRING has not been specified"),
        };

        let mongodb_origin_db = match env::var("MONGODB_ORIGIN_DB") {
            Ok(d) => d,
            Err(_) => panic!("MONGODB_ORIGIN_DB has not been specified"),
        };

        let mongodb_origin_collections: Vec<String> = match env::var("MONGODB_ORIGIN_COLLECTIONS") {
            Ok(d) => d.split(",").map(|s| s.trim().to_string()).collect(),
            Err(_) => panic!("MONGODB_ORIGIN_COLLECTIONS has not been specified"),
        };

        if mongodb_origin_collections.is_empty() {
            panic!("MONGODB_ORIGIN_COLLECTIONS cannot be empty");
        }

        let mongodb_destination_connection_string =
            match env::var("MONGODB_DESTINATION_CONNECTION_STRING") {
                Ok(d) => d,
                Err(_) => panic!("MONGODB_DESTINATION_CONNECTION_STRING has not been specified"),
            };

        let mongodb_destination_db = match env::var("MONGODB_DESTINATION_DB") {
            Ok(d) => d,
            Err(_) => panic!("MONGODB_DESTINATION_DB has not been specified"),
        };

        let origin_client = Client::with_uri_str(mongodb_origin_connection_string)
            .await
            .expect("Failed to initialize origin MongoDB client");

        let destination_client = Client::with_uri_str(mongodb_destination_connection_string)
            .await
            .expect("Failed to initialize destination MongoDB client");

        AppConfig::new(
            display_header,
            copy_indices,
            read_to_memory,
            mongodb_origin_db,
            mongodb_origin_collections,
            mongodb_destination_db,
            origin_client,
            destination_client,
        )
    }
}
