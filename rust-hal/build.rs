use std::env::{self, current_dir};
use std::fs::{File, self};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::str::from_utf8;

fn main() {    
    // Put `memory.x` in our output directory and ensure it's
    // on the linker search path.
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(include_bytes!("memory.x"))
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());

    // Run Python script to build fonts
    // (Script output is the source of a Rust module named `font_data`)
    let output = Command::new("python3")
        .arg("../build_font.py")
        .arg(current_dir().unwrap())
        .output()
        .expect("Python invocation to build fonts failed");
    if !output.status.success() {
        panic!(
            "Python invocation to build fonts failed:\n\nstdout:\n{}\n\nstderr:\n{}",
            from_utf8(&output.stdout[..]).unwrap(),
            from_utf8(&output.stderr[..]).unwrap(),
        );
    }

    // Write fonts to file
    fs::write(
        out.join("font_data.rs"),
        from_utf8(&output.stdout[..]).unwrap()
    ).unwrap();
    

    // TODO: Rerun only if changed (for memory.x and font stuff)
}
