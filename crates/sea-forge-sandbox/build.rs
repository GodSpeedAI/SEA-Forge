//! Embeds the bundle identifier of the macOS jail launcher.
//!
//! The App Sandbox needs a bundle identifier for the process it confines. A
//! plain command-line binary has none, so an Info.plist is linked into the
//! `__TEXT,__info_plist` section. Other targets do not need it.

use std::{env, fs, path::PathBuf};

const LAUNCHER_INFO_PLIST: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>CFBundleIdentifier</key>
	<string>ai.godspeed.sea-forge.jail-launcher</string>
	<key>CFBundleName</key>
	<string>sea-forge-jail-launcher</string>
	<key>CFBundleVersion</key>
	<string>1</string>
	<key>CFBundlePackageType</key>
	<string>APPL</string>
</dict>
</plist>
"#;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("macos") {
        return;
    }
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap_or_default());
    let plist = out_dir.join("sea-forge-jail-launcher-Info.plist");
    if let Err(error) = fs::write(&plist, LAUNCHER_INFO_PLIST) {
        panic!("write {}: {error}", plist.display());
    }
    println!(
        "cargo:rustc-link-arg-bin=sea-forge-jail-launcher=-Wl,-sectcreate,__TEXT,__info_plist,{}",
        plist.display()
    );
}
