//! Build script for MemRL
//!
//! Builds protoc from source if not installed.
//! This is required by LanceDB dependencies (lance-encoding).
//!
//! Using protobuf-src ensures protoc is available to the entire
//! dependency graph, not just this crate.

fn main() {
    // protobuf-src builds protoc and provides its path
    // SAFETY: Single-threaded build script, no concurrent env access
    unsafe {
        std::env::set_var("PROTOC", protobuf_src::protoc());
    }
}
