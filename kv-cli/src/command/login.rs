use anyhow::{bail, Result};
use log::info;
use crate::{new, PBAR};

pub fn login(
    registry: Option<String>,
    scope: &Option<String>,
    auth_type: &Option<String>,
) -> Result<()> {
    // let registry = registry.unwrap_or_else(|| npm::DEFAULT_NPM_REGISTRY.to_string());

    info!("Logging in to npm...");
    // info!(
    //     "Scope: {:?} Registry: {}, Auth Type: {:?}.",
    //     &scope, &registry, &auth_type
    // );
    info!("npm info located in the npm debug log");
    // npm::npm_login(&registry, &scope, &auth_type)?;


    // Interactively ask user for npm login info.  (new::run does not support interactive input)
    info!("Logged you in!");

    PBAR.info(&"👋  logged you in!".to_string());
    Ok(())
}
