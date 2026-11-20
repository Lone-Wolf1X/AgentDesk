fn main() {
    #[cfg(windows)]
    {
        // Require Administrator UAC prompt when running the installer on Windows
        println!("cargo:rustc-link-arg=/MANIFESTUAC:level='requireAdministrator'");
    }
}
