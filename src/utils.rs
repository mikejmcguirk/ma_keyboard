use {
    core::fmt::Display,
    std::{fs::File, io::Write as _},
};

use {anyhow::Result, chrono::Local};

pub fn write_log<T: Display>(handle: &mut File, log: &T) -> Result<()> {
    let timestamp: String = Local::now().format("%Y-%m-%d, %H:%M:%S").to_string();
    writeln!(handle, "{timestamp}: {log}")?;
    handle.flush()?;

    return Ok(());
}
