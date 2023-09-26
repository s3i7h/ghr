use anyhow::{anyhow, Result};
use clap::Parser;

use crate::config::Config;
use crate::root::Root;
use crate::url::Url;

#[cfg(windows)]
fn open_url(url: &url::Url) -> Result<()> {
    use std::ffi::CString;

    use windows::core::{s, PCSTR};
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::Shell::ShellExecuteA;
    use windows::Win32::UI::WindowsAndMessaging::SHOW_WINDOW_CMD;

    // https://web.archive.org/web/20150421233040/https://support.microsoft.com/en-us/kb/224816
    unsafe {
        ShellExecuteA(
            HWND::default(),
            s!("open"),
            PCSTR::from_raw(CString::new(url.to_string().as_str())?.as_ptr() as *const u8),
            PCSTR::null(),
            PCSTR::null(),
            SHOW_WINDOW_CMD(0),
        );
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn open_url(url: &url::Url) -> Result<()> {
    std::process::Command::new("open")
        .arg(url.to_string())
        .spawn()?;

    Ok(())
}

#[cfg(all(not(windows), not(target_os = "macos")))]
fn open_url(url: &url::Url) -> Result<()> {
    std::process::Command::new("xdg-open")
        .arg(url.to_string())
        .spawn()?;

    Ok(())
}

#[derive(Debug, Parser)]
pub struct Cmd {
    /// URL or pattern of the repository to be browsed.
    repo: String,
}

impl Cmd {
    pub async fn run(self) -> Result<()> {
        let root = Root::find()?;
        let config = Config::load_from(&root)?;

        let url = Url::from_str(
            &self.repo,
            &config.patterns,
            config.defaults.owner.as_deref(),
        )?;

        let platform = config
            .platforms
            .find(&url)
            .ok_or_else(|| anyhow!("Could not find a platform to browse on."))?
            .try_into_platform()?;

        let url = platform.get_browsable_url(&url).await?;

        open_url(&url)?;
        Ok(())
    }
}
