mod login;

use clap::Subcommand;
use anyhow::Result;
use log::info;
use crate::command::login::login;

/// The various kinds of commands that `command` can execute.
#[derive(Debug, PartialEq, Subcommand)]
pub enum Command {
    #[clap(name = "login", alias = "adduser", alias = "add-user")]
    /// 👤  login sys and check user account!
    Login {
        #[clap(long = "registry", short = 'r')]
        /// The base URL of the kv server package registry.
        registry: Option<String>,

        #[clap(long = "scope", short = 's')]
        /// Default: none.
        /// If specified, the user and login credentials given will be
        /// associated with the specified scope.
        scope: Option<String>,

        #[clap(long = "auth-type", short = 't')]
        /// Default: 'legacy'.
        /// Type: 'legacy', 'sso', 'saml', 'oauth'.
        /// What authentication strategy to use with adduser/login. Some npm
        /// registries (for example, npmE) might support alternative auth
        /// strategies besides classic username/password entry in legacy npm.
        auth_type: Option<String>,
    },
}

/// Run a command with the given logger!
pub fn run_pack(command: Command) -> Result<()> {
    // Run the correct command based off input and store the result of it so that we can clear the progress bar then return it
    match command {
        Command::Login {
            registry,
            scope,
            auth_type,
        } => {
            info!("Running login command...");
            info!(
                "Registry: {:?}, Scope: {:?}, Auth Type: {:?}",
                &registry, &scope, &auth_type
            );

            login(registry, &scope, &auth_type)
        }
        _ => {
            Ok(())
        }
    }
}
