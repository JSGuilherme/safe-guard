fn main() {
    if cfg!(target_os = "windows") {
        println!("cargo:rerun-if-changed=src/img/logo-sg.ico");

        let mut res = winres::WindowsResource::new();
        res.set_icon("src/img/logo-sg.ico");
        res.set_icon_with_id("src/img/logo-sg.ico", "COFRE_TRAY");
        res.compile().unwrap();

        let out_dir = std::env::var("OUT_DIR").unwrap();
        match std::env::var("CARGO_CFG_TARGET_ENV").as_deref() {
            Ok("msvc") => {
                println!("cargo:rustc-link-arg-bin=cofre_tray={out_dir}\\resource.lib");
            }
            Ok("gnu") => {
                println!("cargo:rustc-link-arg-bin=cofre_tray={out_dir}\\resource.o");
            }
            _ => {}
        }
    }
}
