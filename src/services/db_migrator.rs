use crate::config::app_config::AppConfig;
use futures::stream::TryStreamExt;
use log::{error, info, warn};
use mongodb::bson::{Document, doc};
use mongodb::error::{ErrorKind, WriteFailure};
use mongodb::options::InsertManyOptions;
use mongodb::{Database, IndexModel};

pub struct DbMigrator {
    pub app_config: AppConfig,
}

impl DbMigrator {
    /// Initialize a new DbMigrator instance with the provided AppConfig.
    ///
    /// # Arguments
    ///
    /// - `app_config`: The application configuration used for database migration.
    ///
    /// # Returns
    ///
    /// A new instance of DbMigrator with the provided AppConfig.
    pub fn new(app_config: AppConfig) -> Self {
        Self { app_config }
    }

    /// Migrate a MongoDB database
    ///
    /// # Returns
    ///
    /// - `Result<(), Box<dyn std::error::Error>>` - Result indicating success or failure of the database migration.
    pub async fn migrate_db(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting database migration process");

        let origin_db = &self
            .app_config
            .origin_client
            .database(&self.app_config.mongodb_origin_db);

        let destination_db = &self
            .app_config
            .destination_client
            .database(&self.app_config.mongodb_destination_db);

        match self
            .migrate_collections(origin_db, destination_db, self.app_config.copy_indices)
            .await
        {
            Ok(_) => {
                info!("Collections migration completed successfully");
            }
            Err(err) => {
                error!("Failed to migrate collections: {}", err);
                return Err(err);
            }
        }

        Ok(())
    }

    /// Migrate collections from origin to destination database.
    ///
    /// # Arguments
    ///
    /// - `origin_db` - The origin database.
    /// - `target_db` - The target database.
    /// - `migrate_indices` - Whether to migrate indices along with collections.
    ///
    /// # Returns
    ///
    /// - `Result<(), Box<dyn std::error::Error>>` - Result indicating success or failure of the collection migration.
    pub async fn migrate_collections(
        &self,
        origin_db: &Database,
        target_db: &Database,
        migrate_indices: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting collection migration process");

        let target_collections = match target_db.list_collection_names().await {
            Ok(collections) => collections,
            Err(err) => {
                error!(
                    "Failed to list collections in destination database: {}",
                    err
                );
                return Err(Box::new(err));
            }
        };

        let mut collections_to_create = Vec::new();
        for collection in &self.app_config.mongodb_origin_collections {
            if !target_collections.contains(collection) {
                collections_to_create.push(collection.clone());
            }
        }

        for collection in collections_to_create {
            info!(
                "Collection does not exist in destination database: {}",
                collection
            );

            match target_db.create_collection(&collection).await {
                Ok(_) => {
                    info!("Collection {} created", collection);
                }
                Err(err) => {
                    error!("Failed to create collection {}: {}", collection, err);
                    return Err(Box::new(err));
                }
            }
        }

        let mut handles = Vec::new();
        for collection in &self.app_config.mongodb_origin_collections {
            let origin_db = origin_db.clone();
            let destination_db = target_db.clone();
            let read_to_memory = self.app_config.read_to_memory;
            let collection = collection.clone();

            let handle = tokio::spawn(async move {
                if migrate_indices {
                    match Self::migrate_indices(&collection, &origin_db, &destination_db).await {
                        Ok(_) => {
                            info!("Indices migrated for collection {}", collection);
                        }
                        Err(err) => {
                            error!(
                                "Failed to migrate indices for collection {}: {}",
                                collection, err
                            );
                        }
                    }
                }

                // Migrate documents
                match Self::migrate_documents(
                    &collection,
                    &origin_db,
                    &destination_db,
                    read_to_memory,
                )
                .await
                {
                    Ok(_) => {
                        info!("Documents migrated for collection {}", collection);
                    }
                    Err(err) => {
                        error!(
                            "Failed to migrate documents for collection {}: {}",
                            collection, err
                        );
                    }
                };
            });
            handles.push(handle);
        }

        for handle in handles {
            match handle.await {
                Ok(_) => {}
                Err(err) => {
                    error!("Error occurred during collection migration: {}", err);
                    return Err(Box::new(err));
                }
            }
        }

        Ok(())
    }

    /// Migrate the indices for a given collection from the origin database to the target database.
    ///
    /// # Arguments
    ///
    /// - `collection_name` - The name of the collection to migrate indices from.
    /// - `origin_db` - The origin database.
    /// - `target_db` - The destination database
    ///
    /// # Returns
    ///
    /// - `Result<(), Box<dyn std::error::Error>>` - Result indicating success or failure of the index migration process.
    pub async fn migrate_indices(
        collection_name: &str,
        origin_db: &Database,
        target_db: &Database,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting index migration process");

        let source_collection = origin_db.collection::<Document>(collection_name);
        let target_collection = target_db.collection::<Document>(collection_name);

        // Get source index definitions
        let indices: Vec<IndexModel> = source_collection
            .list_indexes()
            .await?
            .try_collect()
            .await?;

        // Get existing target index names
        let existing_indices: Vec<IndexModel> = target_collection
            .list_indexes()
            .await?
            .try_collect()
            .await?;

        let existing_names: std::collections::HashSet<String> = existing_indices
            .into_iter()
            .filter_map(|idx| idx.options.as_ref().and_then(|opts| opts.name.clone()))
            .collect();

        for index in indices {
            let index_name = index
                .options
                .as_ref()
                .and_then(|opts| opts.name.clone())
                .unwrap_or_else(|| "<unnamed>".to_string());

            // Skip MongoDB's default _id index
            if index_name == "_id_" {
                continue;
            }

            if existing_names.contains(&index_name) {
                info!(
                    "Index '{}' already exists in collection '{}', skipping",
                    index_name, collection_name
                );
                continue;
            }

            match target_collection.create_index(index).await {
                Ok(_) => {
                    info!(
                        "Successfully created index '{}' on collection '{}'",
                        index_name, collection_name
                    );
                }
                Err(err) => {
                    error!(
                        "Failed to create index '{}' on collection '{}': {}",
                        index_name, collection_name, err
                    );
                    return Err(Box::new(err));
                }
            }
        }

        Ok(())
    }

    /// Migrates documents from the origin database to the target database for a given collection.
    ///
    /// # Arguments
    ///
    /// - `collection` - The name of the collection to migrate documents from.
    /// - `origin_db` - The origin database
    /// - `target_db` - The target database
    /// - `in_memory` - Whether to read documents to memory for faster migration
    ///
    /// # Returns
    ///
    /// - `Result<(), Box<dyn std::error::Error>>` - Result indicating success or failure of the document migration.
    pub async fn migrate_documents(
        collection: &str,
        origin_db: &Database,
        target_db: &Database,
        in_memory: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!(
            "Starting document migration process (in memory: {})",
            in_memory
        );

        let source_collection = origin_db.collection::<Document>(collection);
        let target_collection = target_db.collection::<Document>(collection);

        let mut cursor = source_collection.find(doc! {}).await?;

        if in_memory {
            // Bulk insert
            let documents: Vec<Document> = cursor.try_collect().await?;
            let opts = InsertManyOptions::builder().ordered(false).build();

            if documents.is_empty() {
                // Nothing to do
                return Ok(());
            }

            match target_collection
                .insert_many(documents)
                .with_options(opts)
                .await
            {
                Ok(_) => {}
                Err(err) => {
                    let is_duplicate_key = match err.kind.as_ref() {
                        ErrorKind::Write(WriteFailure::WriteError(write_error)) => {
                            write_error.code == 11000
                        }
                        ErrorKind::BulkWrite(bulk_write_error) => bulk_write_error
                            .write_errors
                            .values()
                            .all(|e| e.code == 11000),
                        _ => false,
                    };

                    if is_duplicate_key {
                        warn!("Duplicate key error(s) occurred");
                        return Ok(());
                    }

                    error!(
                        "Failed to insert documents into collection '{}': {}",
                        collection, err
                    );
                    return Err(Box::new(err));
                }
            }
        } else {
            while let Some(document) = cursor.try_next().await? {
                match target_collection.insert_one(document).await {
                    Ok(_) => {}
                    Err(err) => {
                        let is_duplicate_key = match err.kind.as_ref() {
                            ErrorKind::Write(WriteFailure::WriteError(write_error)) => {
                                write_error.code == 11000
                            }
                            ErrorKind::BulkWrite(bulk_write_error) => bulk_write_error
                                .write_errors
                                .values()
                                .all(|e| e.code == 11000),
                            _ => false,
                        };

                        if is_duplicate_key {
                            continue;
                        }

                        error!(
                            "Failed to insert document into collection '{}': {}",
                            collection, err
                        );
                        return Err(Box::new(err));
                    }
                }
            }
        }

        Ok(())
    }
}
