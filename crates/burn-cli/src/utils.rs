use std::process;

#[cfg(unix)]
pub fn check_cargo() -> anyhow::Result<bool> {
    let output = process::Command::new("which").arg("cargo").output()?;

    if output.stdout.len() > 0 {
        Ok(true)
    } else {
        Ok(false)
    }
}

#[cfg(windows)]
pub fn check_cargo() -> anyhow::Result<bool> {
    unimplemented!()
}
