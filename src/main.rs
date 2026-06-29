mod capture;
mod clipboard;
mod color;
mod config;
mod daemon;
mod error;
mod hotkey;
mod overlay;
mod renderer;
mod xkcd_data;

use clap::Parser;
use color::PixelAnalyzer;
use error::{AppError, Result};

/// Screen color picker — press Ctrl to capture pixel color under cursor.
#[derive(Parser)]
#[command(name = "pick", version, about)]
struct Cli {
    /// Capture once and print hex to stdout, then exit
    #[arg(long)]
    once: bool,
}

fn main() -> Result<()> {
    let _sub = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing::metadata::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .compact()
        .init();

    let cli = Cli::parse();

    if cli.once {
        run_once()
    } else {
        let cfg = config::Config::load();
        daemon::run_daemon(cfg)
    }
}

fn run_once() -> Result<()> {
    let cfg = config::Config::load();
    let capturer = capture::create_capturer()?;
    let (x, y) = cursor_pos()?;
    let buf = capturer.grab_region(x - 48, y - 48 - 1, 96, 96)?;
    let color = PixelAnalyzer::sample_center(&buf, 96, 96);
    println!("{}  {}", cfg.default_format.format_color(&color), color.name());
    Ok(())
}

fn cursor_pos() -> Result<(i32, i32)> {
    #[cfg(target_os = "windows")]
    {
        #[link(name = "user32")]
        extern "system" {
            fn GetCursorPos(pt: *mut i32) -> i32;
        }
        unsafe {
            let mut pt = [0i32; 2];
            if GetCursorPos(pt.as_mut_ptr()) == 0 {
                return Err(AppError::Capture("GetCursorPos failed".into()));
            }
            Ok((pt[0], pt[1]))
        }
    }
    #[cfg(target_os = "macos")]
    {
        Err(AppError::Capture("cursor pos: not implemented on macOS".into()))
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Err(AppError::Capture("cursor pos: not implemented on Linux".into()))
    }
}
