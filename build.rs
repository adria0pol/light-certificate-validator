use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = std::fs::remove_dir_all("target/buf-exports");

    // Execute buf export at the beginning
    let output = Command::new("buf")
        .args(&["export", "--output", "target/buf-exports"])
        .output()
        .expect("Failed to execute buf export");

    if !output.status.success() {
        eprintln!("buf export failed:");
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        return Err("buf export compilation failed".into());
    }

    // Compile protobuf files
    tonic_prost_build::configure()
        .include_file("mod.rs")
        .build_client(true)
        .build_server(true)
        .compile_protos(&["proto/validator.proto"], &["target/buf-exports"])?;

    // Tell cargo to rerun this build script if buf.yaml or proto/validator.proto changes
    println!("cargo:rerun-if-changed=buf.yaml");
    println!("cargo:rerun-if-changed=proto/validator.proto");

    Ok(())
}
