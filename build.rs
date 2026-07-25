use std::{env, path::PathBuf};

fn main() {
    let kernel_path = PathBuf::from(env::var_os("CARGO_BIN_FILE_KERNEL_kernel").unwrap());
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let bios_path = out_dir.join("bios.img");

    bootloader::BiosBoot::new(&kernel_path)
        .create_disk_image(&bios_path)
        .expect("failed to create BIOS disk image");

    println!("cargo:rustc-env=BIOS_IMAGE={}", bios_path.display());
}
