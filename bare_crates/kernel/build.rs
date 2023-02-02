use std::{
    env, io,
    path::{Path, PathBuf},
};

fn main() {
    let krate = env::var("CARGO_PKG_NAME").unwrap();

    build_nasm().expect("Failed to compile nasm");
    pass_linker_script(&krate);
}

fn build_nasm() -> io::Result<()> {
    for asm in collect_files("src", |path| {
        path.extension().and_then(|x| x.to_str()) == Some("asm")
    })? {
        eprintln!("Compiling {}", asm.display());

        let object_file = asm.file_name().and_then(|x| x.to_str()).unwrap();
        let mut build = nasm_rs::Build::new();

        build
            .file(&asm)
            .flag("-felf64")
            .target("x86_64-unknown-none")
            .compile(object_file)
            .expect("Failed to compile nasm file");

        // Link as a static library
        println!("cargo:rustc-link-lib=static={object_file}");
    }

    Ok(())
}

fn pass_linker_script(krate: &str) {
    // Tell rustc to pass the linker script to the linker.
    println!("cargo:rustc-link-arg-bin={krate}=--script=linker.ld");

    // Have cargo rerun this script if the linker script or CARGO_PKG_ENV changes.
    println!("cargo:rerun-if-changed=linker.ld");
    println!("cargo:rerun-if-env-changed=CARGO_PKG_NAME");
}

fn collect_files(
    dir: impl AsRef<Path>,
    filter: impl Fn(&Path) -> bool + Copy,
) -> io::Result<Vec<PathBuf>> {
    let mut ret = vec![];
    let path = dir.as_ref();

    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                ret.extend(collect_files(path, filter)?);
            } else if filter(&path) {
                ret.push(path);
            }
        }
    }

    Ok(ret)
}
