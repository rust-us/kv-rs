mod login;

use clap::Subcommand;
use anyhow::Result;
use log::info;
use crate::command::login::login;

/// The various kinds of commands that `command` can execute.
#[derive(Debug, PartialEq, Subcommand)]
pub enum Command {

    None,

    #[clap(name = "login", alias = "adduser", alias = "add-user")]
    /// 👤  Add an npm registry user account! (aliases: adduser, add-user)
    Login {
        #[clap(long = "registry", short = 'r')]
        /// Default: 'https://registry.npmjs.org/'.
        /// The base URL of the npm package registry. If scope is also
        /// specified, this registry will only be used for packages with that
        /// scope. scope defaults to the scope of the project directory you're
        /// currently in, if any.
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

impl Default for Command {
    fn default() -> Self {
        Command::None
    }
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
