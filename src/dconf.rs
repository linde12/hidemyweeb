use std::process::Command;

pub fn set_bool(key: &str, value: bool) -> Result<(), std::io::Error> {
    let status = if value { "true" } else { "false" };
    Command::new("dconf")
        .arg("write")
        .arg(key)
        .arg(status)
        .status()?;
    Ok(())
}
