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

    /// Asks the user for input
    ///
    /// # Arguments
    ///
    /// - `question` - The question to ask the user
    ///
    /// # Returns
    ///
    /// The user's input as a String
    pub fn ask_user_input(question: &str) -> String {
        println!("{}", question);
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");
        input.trim().to_string()
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
            Err(_) => true,
        };

        let copy_indices: bool = match env::var("COPY_INDICES") {
            Ok(d) => {
                let res: bool = d.trim().parse().unwrap_or(false);
                res
            }
            Err(_) => {
                let mut can_continue = false;
                let mut res: bool = false;
                while !can_continue {
                    let user_response =
                        Self::ask_user_input(" Would you like to copy collection indices? (y/n):")
                            .to_lowercase();

                    if user_response == "y" {
                        res = true;
                        can_continue = true;
                    } else if user_response == "n" {
                        res = false;
                        can_continue = true;
                    } else {
                        println!("Invalid input. Please enter 'y' or 'n'.");
                    }
                }

                res
            }
        };

        let read_to_memory = match env::var("READ_TO_MEMORY") {
            Ok(d) => {
                let res: bool = d.trim().parse().unwrap_or(false);
                res
            }
            Err(_) => {
                let mut can_continue = false;
                let mut res: bool = false;
                while !can_continue {
                    let user_response = Self::ask_user_input(
                        " Would you like to copy collections in-memory and use bulk-insert? (y/n):",
                    )
                    .to_lowercase();

                    if user_response == "y" {
                        res = true;
                        can_continue = true;
                    } else if user_response == "n" {
                        res = false;
                        can_continue = true;
                    } else {
                        println!("Invalid input. Please enter 'y' or 'n'.");
                    }
                }

                res
            }
        };

        let mongodb_origin_connection_string = env::var("MONGODB_ORIGIN_CONNECTION_STRING")
            .unwrap_or_else(|_| {
                Self::ask_user_input(" Please enter the origin MongoDB connection string:")
            });

        let mongodb_origin_db = env::var("MONGODB_ORIGIN_DB").unwrap_or_else(|_| {
            Self::ask_user_input(" Please enter the origin MongoDB database name:")
        });

        let mut mongodb_origin_collections: Vec<String> = match env::var(
            "MONGODB_ORIGIN_COLLECTIONS",
        ) {
            Ok(d) => d.split(",").map(|s| s.trim().to_string()).collect(),
            Err(_) => {
                let result = Self::ask_user_input(
                    " Please enter the origin MongoDB collection names (comma-separated) or leave empty to read all collections:",
                );
                if result.is_empty() {
                    vec![]
                } else {
                    result.split(",").map(|s| s.trim().to_string()).collect()
                }
            }
        };

        let mongodb_destination_connection_string =
            env::var("MONGODB_DESTINATION_CONNECTION_STRING").unwrap_or_else(|_| {
                Self::ask_user_input(" Please enter the destination MongoDB connection string:")
            });

        let mongodb_destination_db = env::var("MONGODB_DESTINATION_DB").unwrap_or_else(|_| {
            Self::ask_user_input(" Please enter the destination MongoDB database name:")
        });

        let origin_client = Client::with_uri_str(mongodb_origin_connection_string)
            .await
            .expect("Failed to initialize origin MongoDB client");

        let destination_client = Client::with_uri_str(mongodb_destination_connection_string)
            .await
            .expect("Failed to initialize destination MongoDB client");

        if mongodb_origin_collections.is_empty() {
            // Read all collections from the origin database
            let collections = match destination_client
                .database(&mongodb_destination_db)
                .list_collection_names()
                .await
            {
                Ok(collections) => collections,
                Err(e) => panic!("Failed to list collections in origin database: {}", e),
            };

            for name in collections {
                mongodb_origin_collections.push(name);
            }
        }

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
