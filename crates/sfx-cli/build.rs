use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let target = env::var("TARGET").unwrap();
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let root_dir = Path::new(&manifest_dir).parent().unwrap().parent().unwrap();
    
    let stub_target_dir = root_dir.join("target").join("sfx-stub-build");

    println!("cargo:rerun-if-changed=../sfx-stub/src");
    println!("cargo:rerun-if-changed=../sfx-stub/Cargo.toml");

    let mut cmd = Command::new("cargo");
    cmd.arg("build")
        .arg("--release")
        .arg("-p")
        .arg("sfx-stub")
        .arg("--target")
        .arg(&target)
        .arg("--target-dir")
        .arg(&stub_target_dir)
        .current_dir(root_dir);

    let status = cmd.status().expect("Failed to run cargo build for sfx-stub");

    if !status.success() {
        panic!("Failed to build sfx-stub");
    }

    let exe_ext = if target.contains("windows") { ".exe" } else { "" };
    let stub_bin_name = format!("sfx-stub{}", exe_ext);
    let stub_path = stub_target_dir.join(&target).join("release").join(&stub_bin_name);

    if !stub_path.exists() {
         panic!("Built stub not found at {}", stub_path.display());
    }
    
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let dest_path = out_dir.join("sfx-stub.bin"); 
    std::fs::copy(&stub_path, &dest_path).expect("Failed to copy stub binary");
}
