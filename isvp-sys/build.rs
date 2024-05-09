fn main() {
    // Tell Cargo that if the given file changes, to rerun this build script.
    
    let build_dir = env::var("GITHUB_WORKSPACE").unwrap_or("/atomskygaze".to_string());
    let src_dir = env::var("GITHUB_WORKSPACE").unwrap_or("/src".to_string());
    println!(
        "cargo:rustc-link-search=native={}/build/buildroot-2024.02/output/target/usr/lib",
        build_dir
    );
    println!("cargo:rustc-link-search=native={}/mxu", src_dir);
    //println!("cargo:rustc-link-lib=uClibc");
    //println!("cargo:rustc-link-lib=lduClibc");
    //println!("cargo:rustc-link-lib=udl");
    println!("cargo:rustc-link-lib=sysutils");
    println!("cargo:rustc-link-lib=imp");
    println!("cargo:rustc-link-lib=alog");
    println!("cargo:rustc-link-lib=audioProcess");
    println!("cargo:rustc-link-lib=mxu_imgproc");
}
