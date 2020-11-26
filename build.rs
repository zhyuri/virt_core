extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn sdk_path() -> Result<String, std::io::Error> {
    use std::process::Command;

    let target = &std::env::var("TARGET").unwrap();
    let sdk = if target == "aarch64-apple-darwin" {
        "macosx11.0"
    } else {
        unreachable!();
    };

    let output = Command::new("xcrun")
        .args(&["--sdk", sdk, "--show-sdk-path"])
        .output()?
        .stdout;
    let prefix_str = std::str::from_utf8(&output).expect("invalid output from `xcrun`");
    Ok(prefix_str.trim_end().to_string())
}

fn main() {
    let path = sdk_path().unwrap();
    let clang_args = vec!["-x", "objective-c", "-fblocks", "-isysroot", &path];

    println!("cargo:rerun-if-env-changed=BINDGEN_EXTRA_CLANG_ARGS");
    // Tell cargo to tell rustc to link the framework
    println!("cargo:rustc-link-lib=framework=Virtualization");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        .clang_args(&clang_args)
        .objc_extern_crate(true)
        .block_extern_crate(true)
        .generate_block(true)
        .rustfmt_bindings(true)
        // time.h as has a variable called timezone that conflicts with some of the objective-c
        // calls from NSCalendar.h in the Foundation framework. This removes that one variable.
        .blacklist_item("timezone")
        // https://github.com/rust-lang/rust-bindgen/issues/1705
        .blacklist_item("IUIStepper")
        .blacklist_function("dividerImageForLeftSegmentState_rightSegmentState_")
        .blacklist_item("objc_object")
        // The input header we would like to generate
        // bindings for.
        .header_contents(
            "Virtualization.h",
            "#include<Virtualization/Virtualization.h>",
        )
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
