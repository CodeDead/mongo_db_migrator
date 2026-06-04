use crate::config::app_config::AppConfig;
use log::info;
use mongodb::Client;
use std::env;

/// Asks the user for input
///
/// # Arguments
///
/// - `question` - The question to ask the user
///
/// # Returns
///
/// The user's input as a String
fn ask_user_input(question: &str) -> String {
    println!("{}", question);
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");
    input.trim().to_string()
}

/// Asks the user a yes/no question
///
/// # Arguments
///
/// - `question` - The question to ask the user
///
/// # Returns
///
/// `true` if the user answered 'y', `false` if the user answered 'n'
fn ask_yes_no(question: &str) -> bool {
    loop {
        let response = ask_user_input(question).to_lowercase();
        match response.as_str() {
            "y" => return true,
            "n" => return false,
            _ => println!("Invalid input. Please enter 'y' or 'n'."),
        }
    }
}

/// Reads the server configuration from environment variables
///
/// # Returns
///
/// A new instance of AppConfig
pub async fn read_server_config() -> AppConfig {
    info!("Reading configuration from environment variables");

    let copy_indices: bool = match env::var("COPY_INDICES") {
        Ok(d) => {
            let res: bool = d.trim().parse().unwrap_or(false);
            res
        }
        Err(_) => ask_yes_no("Would you like to copy collection indices? (y/n):"),
    };

    let read_to_memory = match env::var("READ_TO_MEMORY") {
        Ok(d) => {
            let res: bool = d.trim().parse().unwrap_or(false);
            res
        }
        Err(_) => {
            ask_yes_no("Would you like to copy collections in-memory and use bulk-insert? (y/n):")
        }
    };

    let mongodb_origin_connection_string = env::var("MONGODB_ORIGIN_CONNECTION_STRING")
        .unwrap_or_else(|_| ask_user_input("Please enter the origin MongoDB connection string:"));

    let mongodb_origin_db = env::var("MONGODB_ORIGIN_DB")
        .unwrap_or_else(|_| ask_user_input("Please enter the origin MongoDB database name:"));

    let mut mongodb_origin_collections: Vec<String> = match env::var("MONGODB_ORIGIN_COLLECTIONS") {
        Ok(d) => {
            if d.is_empty() {
                vec![]
            } else {
                d.split(",")
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            }
        }
        Err(_) => {
            let result = ask_user_input(
                "Please enter the origin MongoDB collection names (comma-separated) or leave empty to read all collections:",
            );
            if result.is_empty() {
                vec![]
            } else {
                result
                    .split(",")
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            }
        }
    };

    let mongodb_destination_connection_string = env::var("MONGODB_DESTINATION_CONNECTION_STRING")
        .unwrap_or_else(|_| {
            ask_user_input("Please enter the destination MongoDB connection string:")
        });

    let mongodb_destination_db = env::var("MONGODB_DESTINATION_DB")
        .unwrap_or_else(|_| ask_user_input("Please enter the destination MongoDB database name:"));

    let origin_client = Client::with_uri_str(mongodb_origin_connection_string)
        .await
        .expect("Failed to initialize origin MongoDB client");

    let destination_client = Client::with_uri_str(mongodb_destination_connection_string)
        .await
        .expect("Failed to initialize destination MongoDB client");

    if mongodb_origin_collections.is_empty() {
        // Read all collections from the origin database
        let collections = match origin_client
            .database(&mongodb_origin_db)
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
        copy_indices,
        read_to_memory,
        mongodb_origin_db,
        mongodb_origin_collections,
        mongodb_destination_db,
        origin_client,
        destination_client,
    )
}
