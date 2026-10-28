fn main() {
    println!("cargo:rustc-check-cfg=cfg(esp32c6)");
    println!("cargo:rustc-check-cfg=cfg(esp_idf_esp_bringup_bypass_random_setting)");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("espidf") {
        embuild::espidf::sysenv::output();
    }
}
