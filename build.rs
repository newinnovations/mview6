fn main() {
    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo::rerun-if-changed=resources");

    glib_build_tools::compile_resources(
        &["resources"],
        "resources/mview6.gresource.xml",
        "mview6.gresource",
    );
}
