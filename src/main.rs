#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::{Context, Result, bail};
use openlogi_core::{
    config::ConfigFile,
    device::DeviceKind,
    device_order::{DeviceIdentity, DeviceStableId},
    hid::DeviceRoute,
};
use openlogi_ipc::{ClientKind, client::connect_as};
use serde::Deserialize;
use std::{fs, io::Write, path::PathBuf};
use tarpc::context;
use windows_sys::Win32::UI::{
    Input::KeyboardAndMouse::{MOD_NOREPEAT, RegisterHotKey, UnregisterHotKey},
    WindowsAndMessaging::{GetMessageW, MB_ICONERROR, MSG, MessageBoxW, WM_HOTKEY},
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Settings {
    device_key: String,
    key: u32,
    modifiers: u32,
}

fn directory() -> Result<PathBuf> {
    Ok(std::env::current_exe()?
        .parent()
        .context("executable directory unavailable")?
        .to_owned())
}

fn settings() -> Result<Settings> {
    let text = fs::read_to_string(directory()?.join("settings.toml"))
        .context("place settings.toml beside the executable")?;
    let value: Settings = toml::from_str(&text)?;
    if value.key == 0 || value.key > 254 || value.modifiers == 0 || value.modifiers > 15 {
        bail!("invalid hotkey: key must be 1..254 and modifiers 1..15");
    }
    if value.key == 123 && value.modifiers == 7 {
        bail!("toggle hotkey conflicts with the fixed exit shortcut");
    }
    Ok(value)
}

fn log(message: &str) {
    if let Ok(dir) = directory() {
        if let Ok(mut file) = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(dir.join("hotkey.log"))
        {
            let _ = writeln!(file, "{:?}: {message}", std::time::SystemTime::now());
        }
    }
    eprintln!("{message}");
}

async fn lighting(settings: &Settings, toggle: bool) -> Result<bool> {
    let client = connect_as(ClientKind::Cli)
        .await
        .context("OpenLogi must be running and compatible with v0.9.0")?;
    let snapshot = client.snapshot(context::current()).await?;
    let (mut config, mut file) = ConfigFile::load_or_default()?;
    let mut matches = Vec::new();
    for inventory in &snapshot.inventory {
        for paired in &inventory.paired {
            if !paired.online
                || paired.kind != DeviceKind::Keyboard
                || !paired
                    .capabilities
                    .as_ref()
                    .is_some_and(|caps| caps.lighting)
            {
                continue;
            }
            let Some(model) = &paired.model_info else {
                continue;
            };
            let Some(route) = DeviceRoute::for_slot(inventory, paired.slot) else {
                continue;
            };
            let stable = DeviceStableId::from_parts(
                Some(&route),
                paired.slot,
                model.serial_number.as_deref(),
                model.unit_id,
            );
            let identity =
                DeviceIdentity::from_parts(model.serial_number.as_deref(), model.unit_id);
            let resolved = config
                .resolve_device_key(&stable, Some(&identity))
                .map(|key| key.into_string());
            if let Some(key) = resolved {
                if settings.device_key.is_empty() || key == settings.device_key {
                    matches.push((route, stable.route_key(), key));
                }
            }
        }
    }
    if matches.len() != 1 {
        bail!(
            "expected exactly one online RGB keyboard for {:?}, found {}; set device_key in settings.toml when several are connected",
            settings.device_key,
            matches.len()
        );
    }
    let (route, route_key, device_key) = matches.remove(0);
    let device = config
        .devices
        .get(&device_key)
        .context("device has no saved OpenLogi configuration")?;
    // The GUI writes the device setting; refuse a link override that would mask it.
    if device
        .links
        .get(&route_key)
        .is_some_and(|link| link.overrides.lighting.is_some())
    {
        bail!("remove this device's per-link lighting override before using the hotkey");
    }
    let mut lighting = device
        .effective_lighting(&route_key)
        .cloned()
        .context("configure Lighting in OpenLogi first")?;
    if !toggle {
        return Ok(lighting.enabled);
    }
    lighting.enabled = !lighting.enabled;
    let enabled = lighting.enabled;
    // Match the GUI: persist first, reload agent configuration, then apply hardware.
    config.set_lighting(&device_key, lighting.clone());
    file.save(&config)
        .context("could not save; no hardware command sent")?;
    client
        .reload_config(context::current())
        .await?
        .map_err(|error| {
            anyhow::anyhow!("setting saved but agent reload failed: {}", error.message)
        })?;
    client
        .set_lighting(context::current(), route, lighting)
        .await?
        .map_err(|error| anyhow::anyhow!("setting saved but hardware write failed: {error:?}"))?;
    Ok(enabled)
}

struct Hotkey(i32);
impl Drop for Hotkey {
    fn drop(&mut self) {
        unsafe {
            UnregisterHotKey(std::ptr::null_mut(), self.0);
        }
    }
}
fn register(id: i32, modifiers: u32, key: u32) -> Result<Hotkey> {
    // Thread-scoped hotkeys; no window pointer is retained by Windows.
    if unsafe { RegisterHotKey(std::ptr::null_mut(), id, modifiers | MOD_NOREPEAT, key) } == 0 {
        bail!(
            "could not register hotkey (already running or shortcut in use): {}",
            std::io::Error::last_os_error()
        );
    }
    Ok(Hotkey(id))
}

fn run() -> Result<()> {
    let settings = settings()?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let mode = std::env::args().nth(1);
    match mode.as_deref() {
        Some("--status") | Some("--toggle") => {
            let enabled =
                runtime.block_on(lighting(&settings, mode.as_deref() == Some("--toggle")))?;
            log(if enabled {
                "Lighting ON"
            } else {
                "Lighting OFF"
            });
            return Ok(());
        }
        None | Some("--check-hotkey") => (),
        _ => bail!("usage: openlogi-lighting-hotkey [--status|--toggle|--check-hotkey]"),
    }
    let _toggle = register(1, settings.modifiers, settings.key)?;
    // Ctrl+Shift+Alt+F12 quits, without changing lighting.
    let _quit = register(2, 7, 123)?;
    if mode.is_some() {
        log("Hotkeys available");
        return Ok(());
    }
    log("Listening; Ctrl+Shift+Alt+F12 exits");
    loop {
        // GetMessageW initializes MSG before it is read; this thread owns both registrations.
        let mut message: MSG = unsafe { std::mem::zeroed() };
        let result = unsafe { GetMessageW(&mut message, std::ptr::null_mut(), 0, 0) };
        if result == -1 {
            return Err(std::io::Error::last_os_error().into());
        }
        if result == 0 {
            break;
        }
        if message.message == WM_HOTKEY {
            if message.wParam == 2 {
                break;
            }
            if message.wParam == 1 {
                match runtime.block_on(lighting(&settings, true)) {
                    Ok(enabled) => log(if enabled {
                        "Lighting ON"
                    } else {
                        "Lighting OFF"
                    }),
                    Err(error) => log(&format!("Toggle failed: {error:#}")),
                }
            }
        }
    }
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        let message = format!("{error:#}");
        log(&message);
        let wide: Vec<u16> = message.encode_utf16().chain(Some(0)).collect();
        let title: Vec<u16> = "OpenLogi Lighting Hotkey"
            .encode_utf16()
            .chain(Some(0))
            .collect();
        if std::env::args().len() == 1 {
            unsafe {
                MessageBoxW(
                    std::ptr::null_mut(),
                    wide.as_ptr(),
                    title.as_ptr(),
                    MB_ICONERROR,
                );
            }
        }
        std::process::exit(1);
    }
}
