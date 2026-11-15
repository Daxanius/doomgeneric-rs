// Example custom build script.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-env-changed=DOOMGENERIC_RESX");
    println!("cargo:rerun-if-env-changed=DOOMGENERIC_RESY");
    let resx = std::env::var("DOOMGENERIC_RESX").unwrap_or_else(|_| "320".to_string());
    let resy = std::env::var("DOOMGENERIC_RESY").unwrap_or_else(|_| "200".to_string());
    println!("cargo:rustc-env=DOOMGENERIC_RESX={resx}");
    println!("cargo:rustc-env=DOOMGENERIC_RESY={resy}");

    let dg_src_dir = &std::path::PathBuf::from("doomgeneric/doomgeneric");
    let mut dg_c_paths = vec![];
    let mut dg_header_paths = vec![];

    // Find most c and h files
    for entry in std::fs::read_dir(dg_src_dir)? {
        let entry = entry?;
        if let Some(filename) = entry.file_name().to_str() {
            if filename.starts_with("doomgeneric_")
                || filename.contains("_allegro")
                || filename.contains("_sdl")
                || filename == "i_main.c"
                || filename == "doomdef.c"
                || filename == "icon.c" 
            {
                continue;
            }

            if std::path::Path::new(filename)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("h"))
            {
                dg_header_paths.push(dg_src_dir.join(filename));
            } else if std::path::Path::new(filename)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("c"))
            {
                dg_c_paths.push(dg_src_dir.join(filename));
            }
        }
    }
    dg_c_paths
        .iter()
        .chain(dg_header_paths.iter())
        .for_each(|path| println!("cargo:rerun-if-changed={}", path.to_str().unwrap()));

    if cfg!(target_os = "windows") {
        println!("cargo:rustc-link-lib=user32");
    }

    let cmap256 = std::env::var_os("CARGO_FEATURE_CMAP256").is_some();
    let mut build = cc::Build::new();
    build
        .define("DOOMGENERIC_RESX", resx.as_str())
        .define("DOOMGENERIC_RESY", resy.as_str())
        .files(dg_c_paths);

    if cmap256 {
        build.define("CMAP256", None);
    }

    build.compile("doomgeneric");

    //println!("cargo:rustc-link-lib=static=doomgeneric");
    Ok(())
}
