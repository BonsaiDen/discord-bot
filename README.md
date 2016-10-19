## Getting started

1. Get rust nightly via [rustup](https://www.rustup.rs/).
2. Clone the repository
3. Setup the project to use rust nightly `rustup override add nightly`
4. Create a bot application at https://discordapp.com/developers/applications/me
5. Create a `.env` file containing the general bot configuration:

  ```env
  DISCORD_BOT_TOKEN=<YOUR_BOT_TOKEN>
  SERVER_WHITELIST=<YOUR_SERVER_ID>
  CONFIG_DIRECTORY=<DIRECTORY_WHERE_BOT_CONFIGURATION_IS_STORED>
  ```

6. Create a initial `config/<your_server_id>/config.toml` for your server configuration:

  ```toml
  admins = ["YourName#Id"]
  uploaders = ["YourName#Id"]
  ```
  
7. Compile and run the bot via `cargo run`

8. Then message `!help` in a channel of your server where the bot is present.

## License

Licensed under either of
 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
at your option.

