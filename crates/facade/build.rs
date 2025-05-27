fn main() {
    cc::Build::new()
        .file("quicklook-shim/quicklook_shim.m")
        .flag("-fobjc-arc")
        .flag("-fmodules")
        .compile("quicklook_shim");

    println!("cargo:rustc-link-lib=framework=QuickLookUI");
    println!("cargo:rustc-link-lib=framework=Cocoa");
}
