use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crossbeam_channel::bounded;
use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::{Icon, TrayIconBuilder, TrayIconEvent};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowAttributes;

use crate::capture;
use crate::color::PixelAnalyzer;
use crate::config::Config;
use crate::error::{AppError, Result};
use crate::hotkey;

struct CaptureFrame {
    buf: Vec<u8>,
}

pub fn run_daemon(_config: Config) -> Result<()> {
    let event_loop = EventLoop::new().map_err(|e| AppError::Other(format!("EventLoop: {e}")))?;

    // Hidden window keeps winit's event loop running on all platforms
    #[allow(deprecated)]
    let _window = event_loop.create_window(
        WindowAttributes::default()
            .with_visible(false)
            .with_title("pick"),
    ).map_err(|e| AppError::Other(format!("Hidden window: {e}")))?;

    // --- System tray ---
    let quit_item = MenuItem::with_id("quit", "Quit", true, None);
    let settings_item = MenuItem::with_id("settings", "Settings...", true, None);
    let menu = Menu::new();
    menu.append(&settings_item)
        .map_err(|e| AppError::Other(format!("Menu: {e}")))?;
    menu.append(&quit_item)
        .map_err(|e| AppError::Other(format!("Menu: {e}")))?;

    let _tray = TrayIconBuilder::new()
        .with_tooltip("pick")
        .with_icon(load_tray_icon())
        .with_menu(Box::new(menu))
        .build()
        .map_err(|e| AppError::Other(format!("Tray: {e}")))?;

    let tray_rx = TrayIconEvent::receiver();
    let menu_rx = MenuEvent::receiver();

    // --- Background capture thread ---
    let capturer = capture::create_capturer()?;
    let (capture_tx, capture_rx) = bounded::<CaptureFrame>(2);
    let alt_state = Arc::new(AtomicBool::new(false));
    let alt_clone = alt_state.clone();

    std::thread::spawn(move || {
        loop {
            let held = hotkey::is_alt_held();
            alt_clone.store(held, Ordering::Relaxed);

            if held {
                let (x, y) = cursor_pos();
                if let Ok(buf) = capturer.grab_region(x - 48, y - 48, 96, 96) {
                    let _ = capture_tx.send(CaptureFrame { buf });
                }
            }

            std::thread::sleep(Duration::from_millis(4));
        }
    });

    // --- Event loop ---
    let mut was_held = false;

    #[allow(deprecated)]
    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Poll);

        match event {
            Event::AboutToWait => {
                let is_held = alt_state.load(Ordering::Relaxed);

                if is_held && !was_held {
                    tracing::info!("ALT pressed — capturing");
                }
                if !is_held && was_held {
                    tracing::info!("ALT released");
                }
                was_held = is_held;

                if is_held {
                    if let Ok(frame) = capture_rx.try_recv() {
                        let color = PixelAnalyzer::sample_center(&frame.buf, 96, 96);
                        tracing::info!(
                            hex = %color.to_hex(),
                            rgb = %color.to_rgb_string(),
                            hsl = %color.to_hsl_string(),
                            name = %color.name(),
                            "picked"
                        );
                    }
                }

                // Process tray events
                while let Ok(tray_event) = tray_rx.try_recv() {
                    if let TrayIconEvent::Click { .. } = tray_event {
                        tracing::debug!("tray click");
                    }
                }

                // Process menu events
                while let Ok(menu_event) = menu_rx.try_recv() {
                    if menu_event.id() == quit_item.id() {
                        tracing::info!("Quit requested");
                        elwt.exit();
                    }
                }
            }

            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                elwt.exit();
            }

            _ => {}
        }
    })
    .map_err(|e| AppError::Other(format!("Event loop: {e}")))?;

    Ok(())
}

fn cursor_pos() -> (i32, i32) {
    #[cfg(target_os = "windows")]
    {
        #[link(name = "user32")]
        extern "system" {
            fn GetCursorPos(pt: *mut i32) -> i32;
        }
        unsafe {
            let mut pt = [0i32; 2];
            if GetCursorPos(pt.as_mut_ptr()) != 0 {
                (pt[0], pt[1])
            } else {
                (0, 0)
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        (0, 0)
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        (0, 0)
    }
}

fn load_tray_icon() -> Icon {
    let size = 32u32;
    let cx = 16i32;
    let cy = 16i32;
    let r = 14i32;
    let rr = r * r;
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);

    for y in 0..size {
        for x in 0..size {
            let dx = x as i32 - cx;
            let dy = y as i32 - cy;
            let dist = dx * dx + dy * dy;
            if dist <= rr {
                let edge = (dist as f64).sqrt();
                let alpha = if edge <= (r - 1) as f64 {
                    255
                } else {
                    ((r as f64 - edge).max(0.0).min(1.0) * 255.0) as u8
                };
                rgba.extend_from_slice(&[180, 130, 255, alpha]);
            } else {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            }
        }
    }

    Icon::from_rgba(rgba, size, size).expect("valid icon")
}
