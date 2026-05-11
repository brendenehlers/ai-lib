//! `.prompt()` requires a model to be set first (`NoModel` → `HasModel`).

use ai_lib_core::client::ClientBuilder;

fn main() {
    let _ = ClientBuilder::new().prompt("hi");
}
