fn main() {
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rerun-if-changed=assets/app.rc");
        cc::Build::new().file("assets/app.res").compile("app_res");
    }
}
