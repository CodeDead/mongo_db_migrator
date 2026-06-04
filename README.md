# MongoDB database migration tool

This is a simple tool to migrate MongoDB databases from one database to another.
It is written in Rust for simplicity and ease of use.

## Usage

1. Create a `.env` file in the root of the project (see [Configuration](#configuration) for details)
2. Set the required environment variables
    1. *Please make sure your MongoDB connection strings are correct and that the required permissions are granted*
3. Run the tool: `cargo run --release`

## Configuration

The tool uses environment variables to configure its behavior:

| Variable                                | Description                                                                             | Type     | Default |
|-----------------------------------------|-----------------------------------------------------------------------------------------|----------|---------|
| `DISPLAY_HEADER`                        | Display a header logo                                                                   | `bool`   | `true`  |
| `COPY_INDICES`                          | Boolean to indicate whether indices should be re-created in the destination collections | `bool`   | `false` |
| `READ_TO_MEMORY`                        | Read collection documents to memory for faster document migration                       | `bool`   | `false` |
| `MONGODB_ORIGIN_CONNECTION_STRING`      | The origin MongoDB connection string                                                    | `string` | N/A     |
| `MONGODB_ORIGIN_DB`                     | The origin MongoDB database name                                                        | `string` | N/A     |
| `MONGODB_ORIGIN_COLLECTIONS`            | A comma-separated list of collections to migrate (example: `collection1,collection2`)   | `string` | N/A     |
| `MONGODB_DESTINATION_CONNECTION_STRING` | The destination MongoDB connection string                                               | `string` | N/A     |
| `MONGODB_DESTINATION_DB`                | The destination MongoDB database name                                                   | `string` | N/A     |

## Dependencies

- `dotenvy`
- `env_logger`
- `futures`
- `log`
- `mongodb`
- `tokio`

## About

This library is maintained by CodeDead. You can find more about us using the following links:

* [Website](https://codedead.com)
* [Bluesky](https://bsky.app/profile/codedead.com)
* [Facebook](https://facebook.com/deadlinecodedead)

Copyright © 2026 CodeDead
