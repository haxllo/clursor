use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crossbeam_channel::bounded;
use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::{Icon, TrayIconBuilder, TrayIconEvent};
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowLevel};

use crate::capture;
use crate::clipboard;
use crate::color::{Color, PixelAnalyzer};
use crate::config::Config;
use crate::error::{AppError, Result};
use crate::hotkey;
use crate::overlay;
use crate::renderer;

struct CaptureFrame {
    buf: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum State {
    Idle,
    Active,
}

pub fn run_daemon(config: Config) -> Result<()> {
    let event_loop = EventLoop::new().map_err(|e| AppError::Other(format!("EventLoop: {e}")))?;

    // Hidden window required by winit on some platforms for event delivery
    #[allow(deprecated)]
    let _hidden = event_loop.create_window(
        WindowAttributes::default()
            .with_visible(false)
            .with_title("pick"),
    ).map_err(|e| AppError::Other(format!("Hidden window: {e}")))?;

    let format = config.default_format;

    // --- System tray ---
    let quit_item = MenuItem::with_id("quit", "Quit", true, None);
    let settings_item = MenuItem::with_id("settings", "Settings...", true, None);
    let menu = Menu::new();
    menu.append(&settings_item)
        .map_err(|e| AppError::Other(format!("Menu: {e}")))?;
    menu.append(&quit_item)
        .map_err(|e| AppError::Other(format!("Menu: {e}")))?;

    let tray = TrayIconBuilder::new()
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
    let ctrl_state = Arc::new(AtomicBool::new(false));
    let ctrl_clone = ctrl_state.clone();

    std::thread::spawn(move || {
        loop {
            let held = hotkey::is_ctrl_held();
            ctrl_clone.store(held, Ordering::Relaxed);

            if held {
                let (x, y) = cursor_pos();
                            if let Ok(buf) = capturer.grab_region(x - 48, y - 48, 96, 96) {
                    let _ = capture_tx.send(CaptureFrame { buf });
                }
            }

            std::thread::sleep(Duration::from_millis(4));
        }
    });

    // --- Event loop state ---
    let mut state = State::Idle;
    let mut last_color: Option<Color> = None;
    let mut clipboard = clipboard::Clipboard::new().ok();
    let mut overlay_win: Option<Window> = None;
    let mut renderer = renderer::Renderer::new();

    #[allow(deprecated)]
    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Poll);

        match event {
            Event::AboutToWait => {
                let is_held = ctrl_state.load(Ordering::Relaxed);

                // --- State transitions with overlay management ---
                match (state, is_held) {
                    (State::Idle, true) => {
                        state = State::Active;
                        last_color = None;

                        // Create overlay window on first activation
                        if overlay_win.is_none() {
                            let attrs = WindowAttributes::default()
                                .with_title("pick overlay")
                                .with_decorations(false)
                                .with_visible(false)
                                .with_transparent(false)
                                .with_window_level(WindowLevel::AlwaysOnTop)
                                .with_resizable(false)
                                .with_inner_size(LogicalSize::new(
                                    overlay::OVERLAY_W as f64,
                                    overlay::OVERLAY_H as f64,
                                ))
                                .with_enabled_buttons(
                                    winit::window::WindowButtons::CLOSE
                                    | winit::window::WindowButtons::MINIMIZE
                                    | winit::window::WindowButtons::MAXIMIZE,
                                );
                            match elwt.create_window(attrs) {
                                Ok(w) => {
                                    overlay::apply_platform_styles(&w);
                                    overlay::apply_rounded_corners(&w);
                                    overlay_win = Some(w);
                                }
                                Err(e) => {
                                    tracing::error!("overlay window: {e}");
                                }
                            }
                        }
                        if let Some(ref w) = overlay_win {
                            let _ = w.set_visible(true);
                        }
                        tracing::info!("Ctrl pressed — picker active");
                    }
                    (State::Active, false) => {
                        state = State::Idle;
                        if let Some(ref w) = overlay_win {
                            let _ = w.set_visible(false);
                        }
                        // Copy last captured color on release
                        if let Some(color) = last_color {
                            let text = format.format_color(&color);
                            if let Some(ref mut cb) = clipboard {
                                let _ = cb.copy_text(&text);
                                let _ = tray.set_tooltip(Some(format!("pick — {}", text)));
                                tracing::info!(color = %text, name = %color.name(), "Ctrl released — copied");
                            }
                        } else {
                            tracing::info!("Ctrl released — no color captured");
                        }
                    }
                    _ => {}
                }

                // --- Reposition overlay while active ---
                if state == State::Active {
                    if let Some(ref w) = overlay_win {
                        let (cx, cy) = cursor_pos();
                        overlay::position_near_cursor(w, cx, cy);
                        w.request_redraw();
                    }

                    // Drain capture frames
                    while let Ok(frame) = capture_rx.try_recv() {
                        let color = PixelAnalyzer::sample_center(&frame.buf, 96, 96);
                        tracing::debug!(hex = %color.to_hex(), name = %color.name(), "tracking");
                        last_color = Some(color);
                        renderer.update_capture(&frame.buf);
                    }
                }

                // --- Process tray clicks (pick without holding Ctrl) ---
                while let Ok(tray_event) = tray_rx.try_recv() {
                    if let TrayIconEvent::Click { .. } = tray_event {
                        if let Ok(capturer) = capture::create_capturer() {
                            let (x, y) = cursor_pos();
                            if let Ok(buf) = capturer.grab_region(x - 48, y - 48, 96, 96) {
                                let color = PixelAnalyzer::sample_center(&buf, 96, 96);
                                let text = format.format_color(&color);
                                if let Some(ref mut cb) = clipboard {
                                    let _ = cb.copy_text(&text);
                                    let _ = tray.set_tooltip(Some(format!("pick — {}", text)));
                                }
                                tracing::info!(color = %text, name = %color.name(), "tray pick");
                            }
                        }
                    }
                }

                // --- Process menu events ---
                while let Ok(menu_event) = menu_rx.try_recv() {
                    match menu_event.id() {
                        id if id == quit_item.id() => {
                            tracing::info!("Quit requested");
                            elwt.exit();
                        }
                        id if id == settings_item.id() => {
                            let path = Config::config_path()
                                .unwrap_or_else(|_| {
                                    let mut p = std::path::PathBuf::from(
                                        std::env::var("USERPROFILE")
                                            .unwrap_or_else(|_| ".".into()),
                                    );
                                    p.push(".config");
                                    p.push("pick");
                                    p.push("config.toml");
                                    p
                                });
                            tracing::info!(config = %path.display(), "Settings — config file");
                            let _ = open::that(path);
                        }
                        _ => {}
                    }
                }
            }

            Event::WindowEvent {
                window_id,
                event: WindowEvent::RedrawRequested,
            } => {
                if let Some(ref w) = overlay_win {
                    if w.id() == window_id {
                        if let Some(color) = last_color {
                            let name = color.name();
                            renderer.paint(w, color, name);
                        }
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
