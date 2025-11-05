#[cfg(test)]
mod test {
    use assert_cmd::prelude::*;
    use predicates::prelude::*;
    use std::process::Command;

    #[test]
    fn test_help_command() -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin("kvcli")?;

        cmd.arg("--help");
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("A distributed kv storage CLI"))
            .stdout(predicate::str::contains("Usage: kvcli"));

        Ok(())
    }

    #[test]
    fn test_version_command() -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin("kvcli")?;

        cmd.arg("--version");
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("kvcli"));

        Ok(())
    }

    #[test]
    fn test_invalid_subcommand() -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin("kvcli")?;

        cmd.arg("invalid_command");
        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("unrecognized subcommand"));

        Ok(())
    }

    #[test]
    fn test_debug_flag() -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin("kvcli")?;

        cmd.arg("--debug").arg("--help");
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("A distributed kv storage CLI"));

        Ok(())
    }

    #[test]
    fn test_config_flag() -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin("kvcli")?;

        cmd.arg("--config").arg("nonexistent.yaml").arg("--help");
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("A distributed kv storage CLI"));

        Ok(())
    }

    #[test]
    fn test_quiet_flag() -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin("kvcli")?;

        cmd.arg("--quiet").arg("--help");
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("A distributed kv storage CLI"));

        Ok(())
    }

    #[test]
    fn test_non_interactive_flag() -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin("kvcli")?;

        cmd.arg("--non-interactive").arg("--help");
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("A distributed kv storage CLI"));

        Ok(())
    }

    #[test]
    fn test_login_subcommand_help() -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin("kvcli")?;

        cmd.arg("help").arg("login");
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("login"));

        Ok(())
    }
}