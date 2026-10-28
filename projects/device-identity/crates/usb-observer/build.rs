#[cfg(target_os = "macos")]
fn native_bindings() {
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;

    use sha2::{Digest, Sha256};

    fn sdk_query(argument: &str) -> String {
        let output = Command::new("xcrun")
            .args(["--sdk", "macosx", argument])
            .output()
            .unwrap_or_else(|_| panic!("cannot query macOS SDK"));
        assert!(output.status.success(), "cannot query macOS SDK");
        String::from_utf8(output.stdout)
            .unwrap_or_else(|_| panic!("invalid macOS SDK query"))
            .trim()
            .to_owned()
    }

    let sdk_version = sdk_query("--show-sdk-version");
    assert_eq!(sdk_version, "27.0", "native observer requires SDK 27.0");
    let sdk = PathBuf::from(sdk_query("--show-sdk-path"));
    assert!(
        bindgen::clang_version()
            .full
            .split_whitespace()
            .any(|token| token == "21.1.8"),
        "native observer requires libclang 21.1.8"
    );
    let headers = [
        "IOKit.framework/Headers/usb/IOUSBLib.h",
        "IOKit.framework/Headers/usb/USB.h",
        "IOKit.framework/Headers/IOCFPlugIn.h",
        "IOKit.framework/Headers/IOKitLib.h",
        "CoreFoundation.framework/Headers/CFPlugInCOM.h",
        "CoreFoundation.framework/Headers/CFUUID.h",
        "CoreFoundation.framework/Headers/CFNumber.h",
        "CoreFoundation.framework/Headers/CFString.h",
    ];
    let mut provenance = String::from(
        "{\"sdk_version\":\"27.0\",\"bindgen\":\"0.72.1\",\"libclang\":\"21.1.8\",\"headers\":[",
    );
    for (index, header) in headers.iter().enumerate() {
        let path = sdk.join("System/Library/Frameworks").join(header);
        println!("cargo:rerun-if-changed={}", path.display());
        let bytes = fs::read(path).unwrap_or_else(|_| panic!("cannot read native SDK input"));
        if index != 0 {
            provenance.push(',');
        }
        provenance.push_str(&format!(
            "{{\"path\":\"{header}\",\"sha256\":\"{:x}\"}}",
            Sha256::digest(bytes)
        ));
    }
    provenance.push(']');
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo provides OUT_DIR"));

    let mut builder = bindgen::Builder::default()
        .header_contents(
            "usb-observer-sdk.h",
            "#include <CoreFoundation/CoreFoundation.h>\n#include <IOKit/usb/IOUSBLib.h>\n#include <IOKit/IOCFPlugIn.h>\n",
        )
        .clang_arg("-isysroot")
        .clang_arg(sdk.to_string_lossy())
        .allowlist_type("IOUSBDeviceInterface942")
        .allowlist_type("IOCFPlugInInterface")
        .allowlist_type("IOUSBDevRequestTO")
        .allowlist_var("kIOMainPortDefault")
        .allowlist_var("kCFStringEncodingUTF8")
        .allowlist_var("kCFNumberSInt32Type")
        .derive_default(true)
        .generate_comments(false)
        .layout_tests(true);
    for function in [
        "IOCreatePlugInInterfaceForService",
        "IOServiceMatching",
        "IOServiceGetMatchingServices",
        "IOIteratorNext",
        "IOIteratorIsValid",
        "IOObjectRelease",
        "IORegistryEntryCreateCFProperty",
        "CFStringCreateWithCString",
        "CFGetTypeID",
        "CFNumberGetTypeID",
        "CFNumberGetValue",
        "CFRelease",
        "CFUUIDGetConstantUUIDWithBytes",
        "CFUUIDGetUUIDBytes",
    ] {
        builder = builder.allowlist_function(function);
    }
    let bindings_path = output.join("native-bindings.rs");
    builder
        .generate()
        .unwrap_or_else(|_| panic!("cannot generate native SDK bindings"))
        .write_to_file(&bindings_path)
        .unwrap_or_else(|_| panic!("cannot save native SDK bindings"));
    let generated =
        fs::read(bindings_path).unwrap_or_else(|_| panic!("cannot read native SDK bindings"));
    provenance.push_str(&format!(
        ",\"bindings_sha256\":\"{:x}\"}}\n",
        Sha256::digest(generated)
    ));
    fs::write(output.join("native-inputs.json"), provenance)
        .unwrap_or_else(|_| panic!("cannot save native input provenance"));
    println!("cargo:rustc-link-lib=framework=IOKit");
    println!("cargo:rustc-link-lib=framework=CoreFoundation");
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=LIBCLANG_PATH");
    println!("cargo:rerun-if-env-changed=SDKROOT");
    #[cfg(target_os = "macos")]
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        native_bindings();
    }
}
