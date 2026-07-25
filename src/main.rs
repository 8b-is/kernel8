use std::process::Command;

fn main() {
    let bios_image = env!("BIOS_IMAGE");
    let mut cmd = Command::new("qemu-system-x86_64");
    cmd.arg("-drive")
        .arg(format!("format=raw,file={bios_image}"))
        .arg("-serial")
        .arg("stdio")
        .arg("-display")
        .arg("none");
    if std::env::args().any(|a| a == "--no-reboot") {
        cmd.arg("-no-reboot");
    }
    let status = cmd.status().expect("failed to launch qemu-system-x86_64");
    std::process::exit(status.code().unwrap_or(1));
}
