fn main() {
    // Tell Cargo that if the given file changes, to rerun this build script.
    println!(
        "cargo:rustc-link-search=native=/atomskygaze/build/buildroot-2024.02/output/target/usr/lib"
    );
    println!("cargo:rustc-link-search=native=/src/mxu");
    //println!("cargo:rustc-link-lib=uClibc");
    //println!("cargo:rustc-link-lib=lduClibc");
    //println!("cargo:rustc-link-lib=udl");
    println!("cargo:rustc-link-lib=sysutils");
    println!("cargo:rustc-link-lib=imp");
    println!("cargo:rustc-link-lib=alog");
    println!("cargo:rustc-link-lib=audioProcess");
    println!("cargo:rustc-link-lib=mxu_imgproc");
}
