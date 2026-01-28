//! Build script for MemRL
//!
//! LanceDB dependencies (lance-encoding) require protoc.
//! Set the PROTOC environment variable to point to your protoc binary.

fn main() {
    // Verify PROTOC is set (required by lance-encoding via prost-build)
    if std::env::var("PROTOC").is_err() {
        println!("cargo:warning=PROTOC environment variable not set.");
        println!("cargo:warning=Please install protoc and set PROTOC=/path/to/protoc");
    }
}
