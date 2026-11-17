# OpenLogi Lighting Hotkey

A small Windows background app that toggles a Logitech keyboard's **Lighting** switch in [OpenLogi](https://github.com/AprilNEA/OpenLogi). It keeps the saved colour and brightness, so turning lighting back on restores them. OpenLogi is installed separately; this helper connects to its existing agent.

Keep OpenLogi running, then double-click `openlogi-lighting-hotkey.exe` in `dist` (with `settings.toml` beside it).

- **Ctrl+Shift+F12:** toggle lighting.
- **Ctrl+Shift+Alt+F12:** exit the wrapper, leaving the lights as they are.
- Errors and toggle results are written to `hotkey.log` beside the executable.

There is no tray icon. Starting a second copy reports that the shortcut is already registered. No administrator access is needed.

## Install

Run `openlogi-lighting-hotkey-setup-0.1.0.exe`. The installer installs for the current Windows user without administrator access, adds Start menu shortcuts, and offers desktop and **Start automatically when I sign in to Windows** options. Startup is selected by default; uncheck it if you prefer to launch manually. OpenLogi must also be running when you use the shortcut. The helper can start before OpenLogi and will connect on each hotkey press.

Existing settings are preserved when reinstalling. Use Windows Installed apps or the Start menu uninstaller to remove the app and its shortcuts, including startup. Your settings and log are kept in `%LOCALAPPDATA%/OpenLogiLightingHotkey` so they can be reused. Exit with Ctrl+Shift+Alt+F12 before uninstalling or upgrading.

For a script-only installation after building, run `./install.ps1` (or `./install.ps1 -Startup` to launch at sign-in). That alternative creates no Windows uninstall entry; remove its folder and shortcuts manually after exiting the helper.

## Change the shortcut or keyboard

Edit `settings.toml` beside the executable, then restart it. `key` is a Windows virtual-key code (F12 = 123). Add modifier values: Alt = 1, Ctrl = 2, Shift = 4, Win = 8. The exit shortcut is fixed at Ctrl+Shift+Alt+F12; do not use that as the toggle bind.

An empty `device_key` automatically selects the only online RGB keyboard with a saved OpenLogi configuration. With several keyboards, set it to the keyboard's physical key in `~/.config/openlogi/config.toml`. Configure its lighting colour and brightness in OpenLogi first. A disconnected keyboard or ambiguous identity is refused. Per-link lighting overrides are refused to avoid masking the setting being toggled. Tested with a G513 that OpenLogi reports as a G512 over USB.

## Build

Clone with `git clone https://github.com/mynameistito/openlogi-lighting-hotkey.git`. Install Rust and the MSVC C++ build tools, then run `./build.ps1`. The output is in `dist`. No submodule setup is needed.

To produce the Windows installer, install [Inno Setup 6](https://jrsoftware.org/isdl.php), then run `./build-installer.ps1`. You can pass `-Compiler C:/path/to/ISCC.exe`. The setup executable is written to `dist`. Pull requests also build an installer through GitHub Actions; download the `windows-installer` artifact from the workflow run.

The two OpenLogi crates are Git dependencies pinned to the v0.9.0 release commit. Cargo fetches their source during builds; the wrapper imports the real IPC and config types rather than reimplementing the wire protocol. Tokio supplies the runtime required by that IPC connection. TOML/Serde parse settings, and windows-sys supplies Windows API declarations. These compile into the executable: end users install no Rust runtime, packages, or upstream source tree. Release builds optimize for size and strip symbols.

This checkout builds with Rust 1.98 using `--ignore-rust-version`; upstream declares 1.99 as its workspace minimum. Cargo.lock pins the dependencies.

For diagnostics, run the executable with `--status`, `--toggle`, or `--check-hotkey`. Release builds have no console; results go to `hotkey.log`.

## How changes reach OpenLogi

Each press connects and checks the agent protocol, discovers the configured physical keyboard, and reads the latest saved lighting state. It flips only `enabled`, saves through OpenLogi's atomic, revision-checked config writer, reloads the agent config, then sends the same lighting RPC as the GUI. Concurrent config writes are refused. An agent update with a different protocol requires updating the pinned dependency and rebuilding.

As in OpenLogi's own UI, persistence happens before the hardware write. If reload or hardware application fails, the saved setting may already have changed; the log reports that failure. The wrapper does not claim hardware readback verification. OpenLogi's UI receives config revision changes through the agent.

Changes are welcome as small patches. Verify with `cargo check --locked --ignore-rust-version`, `--check-hotkey`, and an on/off cycle with the target keyboard connected.
