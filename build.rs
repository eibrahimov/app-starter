use std::process::Command;

/// Builds the frontend so `rust-embed` can bundle `interface/dist` into the
/// binary. Skipped when SKIP_FRONTEND_BUILD is set (CI, Docker: the frontend
/// is built in its own stage) or when node_modules is missing.
fn main() {
    println!("cargo:rerun-if-changed=interface/src");
    println!("cargo:rerun-if-changed=interface/index.html");
    println!("cargo:rerun-if-changed=interface/package.json");
    println!("cargo:rerun-if-env-changed=SKIP_FRONTEND_BUILD");

    if std::env::var("SKIP_FRONTEND_BUILD").is_ok() {
        ensure_dist();
        return;
    }

    let interface = std::path::Path::new("interface");
    if !interface.join("node_modules").exists() {
        println!(
            "cargo:warning=interface/node_modules missing, skipping frontend build (run `bun install` in interface/)"
        );
        ensure_dist();
        return;
    }

    match Command::new("bun")
        .args(["run", "build"])
        .current_dir(interface)
        .status()
    {
        Ok(s) if s.success() => {}
        Ok(s) => println!(
            "cargo:warning=frontend build failed ({s}); the binary will serve a stale or empty UI"
        ),
        Err(e) => {
            println!("cargo:warning=could not run bun ({e}); install bun to build the frontend");
            ensure_dist();
        }
    }
}

fn ensure_dist() {
    // rust-embed requires the folder to exist, even empty.
    std::fs::create_dir_all("interface/dist").ok();
}
