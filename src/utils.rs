use {
    core::fmt::Display,
    std::{fs::File, io::Write as _},
};

use {anyhow::Result, chrono::Local};

// TODO: pass an option to not write to console
// TODO: should have write_log as well
pub fn write_log<T: Display>(handle: &mut File, err: &T) -> Result<()> {
    let timestamp: String = Local::now().format("%Y-%m-%d, %H:%M:%S").to_string();
    writeln!(handle, "{timestamp}: {err}")?;
    handle.flush()?;

    return Ok(());
}
