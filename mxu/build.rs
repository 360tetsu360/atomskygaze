use std::env;
use std::process::Command;

fn main() {
    println!("cargo::rerun-if-changed=mxu_imgproc.c");

    let build_dir = env::var("GITHUB_WORKSPACE").unwrap_or("/atomskygaze".to_string());
    let src_dir = env::var("GITHUB_WORKSPACE").unwrap_or("/src".to_string());
    let gcc_path = format!(
        "{}/build/mips-gcc472-glibc216-64bit/bin/mips-linux-gnu-gcc",
        build_dir
    );
    let output = Command::new(gcc_path)
        .args([
            "-c",
            "mxu_imgproc.c",
            "-mmxu2",
            "-fPIC",
            "-O2",
            "-o",
            "libmxu_imgproc.a",
        ])
        .output()
        .expect("Failed to execute GCC");

    if !output.status.success() {
        panic!("GCC command executed with non-zero exit status");
    }

    println!("cargo:rustc-link-search=native={}/mxu", src_dir);
    println!("cargo:rustc-link-lib=mxu_imgproc");
}
