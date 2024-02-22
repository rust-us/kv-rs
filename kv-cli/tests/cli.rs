
#[cfg(test)]
mod test {
    use assert_cmd::prelude::*;
    use predicates::prelude::*;
    use std::process::Command;
    use assert_fs::prelude::*;

    #[test]
    fn file_cmd_doesnt_exist() -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin("kv-cli")?;

        cmd.arg("foobar").arg("test/file/doesnt/exist");
        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("Found argument 'test/file/doesnt/exist' which wasn't expected, or isn't valid in this context"));

        Ok(())
    }

    #[test]
    fn file_scmd_path_doesnt_exist() -> Result<(), Box<dyn std::error::Error>> {
        let file = assert_fs::NamedTempFile::new("sample.txt")?;
        file.write_str("A test\nActual content\nMore content\nAnother test")?;

        let mut cmd = Command::cargo_bin("kv-cli")?;

        cmd.arg(file.path()).arg("-p test");
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("A test\n"));

        Ok(())
    }
}
