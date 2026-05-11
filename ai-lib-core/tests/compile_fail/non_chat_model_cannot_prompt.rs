//! A model whose macro declaration does not include `ChatModel` cannot be
//! used with `.prompt()` — the bound `M: ChatModel` on the builder transition
//! makes this a compile error rather than a runtime panic.

use ai_lib_core::client::ClientBuilder;
use ai_lib_core::define_model;

struct NoCapProvider;

define_model!(
    name = NoCapModel,
    provider = NoCapProvider,
    model_name = "no-cap",
    capabilities = [],
);

fn main() {
    let model = NoCapModel::new(NoCapProvider);
    let _ = ClientBuilder::new().model(model).prompt("hi");
}
