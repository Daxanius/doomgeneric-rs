// Example custom build script.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dg_src_dir = &std::path::PathBuf::from("doomgeneric/doomgeneric");
    let mut dg_c_paths = vec![];
    let mut dg_header_paths = vec![];

    // Find most c and h files
    for entry in std::fs::read_dir(dg_src_dir)? {
        let entry = entry?;
        if let Some(filename) = entry.file_name().to_str() {
            if filename.starts_with("doomgeneric")
                || filename.contains("_allegro")
                || filename.contains("_sdl")
                || filename == "i_main.c"
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

    cc::Build::new()
        .flag("-w") // Disable warnings
        .define("CMAP256", None)
        .files(dg_c_paths)
        .compile("doomgeneric");

    //println!("cargo:rustc-link-lib=static=doomgeneric");
    Ok(())
}
