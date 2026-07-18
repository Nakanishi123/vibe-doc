fn main() {
    // The release build will replace this with embedding `frontend/dist`.
    // Keeping the build script now makes that integration point explicit.
    println!("cargo:rerun-if-changed=../../frontend/src");
    println!("cargo:rerun-if-changed=../../frontend/public");
}
