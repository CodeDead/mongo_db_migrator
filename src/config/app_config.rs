pub struct AppConfig {
    pub read_to_memory: bool,
    pub display_header: bool,
    pub copy_indices: bool,
    pub mongodb_origin_db: String,
    pub mongodb_origin_collections: Vec<String>,
    pub mongodb_destination_db: String,

    pub origin_client: mongodb::Client,
    pub destination_client: mongodb::Client,
}

impl AppConfig {
    /// Creates a new `AppConfig` instance.
    ///
    /// # Arguments
    ///
    /// - `display_header` - Boolean to indicate whether to display the application header
    /// - `copy_indices` - Boolean to indicate whether indices should be re-created in the destination collections
    /// - `read_to_memory` - Boolean to indicate whether to read documents to memory for faster migration
    /// - `mongodb_origin_db` - The origin MongoDB database name
    /// - `mongodb_origin_collections` - The origin MongoDB collections to migrate
    /// - `mongodb_destination_db` - The destination MongoDB database name
    /// - `origin_client` - The origin MongoDB client
    /// - `destination_client` - The destination MongoDB client
    ///
    /// # Returns
    ///
    /// A new `AppConfig` instance with the provided configuration settings.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        display_header: bool,
        copy_indices: bool,
        read_to_memory: bool,
        mongodb_origin_db: String,
        mongodb_origin_collections: Vec<String>,
        mongodb_destination_db: String,
        origin_client: mongodb::Client,
        destination_client: mongodb::Client,
    ) -> Self {
        Self {
            display_header,
            copy_indices,
            read_to_memory,
            mongodb_origin_db,
            mongodb_origin_collections,
            mongodb_destination_db,
            origin_client,
            destination_client,
        }
    }
}
