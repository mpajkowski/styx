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
    let header_files = find_files_with_ext("src", "inc")?;

    let mut include_paths = header_files
        .into_iter()
        .map(|p| p.parent().unwrap().to_owned())
        .collect::<Vec<_>>();
    include_paths.dedup();

    let source_files = find_files_with_ext("src", "asm")?;

    let mut build = nasm_rs::Build::new();

    for include in &include_paths {
        build.include(include);
    }

    build.flag("-felf64").target("x86_64-unknown-none");

    for asm in source_files {
        let object_file = asm.file_name().and_then(|x| x.to_str()).unwrap();
        build
            .file(&asm)
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

fn find_files_with_ext(dir: impl AsRef<Path>, ext: &str) -> io::Result<Vec<PathBuf>> {
    let mut ret = vec![];
    let path = dir.as_ref();

    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                ret.extend(find_files_with_ext(path, ext)?);
            } else if path.extension().and_then(|x| x.to_str()) == Some(ext) {
                ret.push(path);
            }
        }
    }

    Ok(ret)
}
