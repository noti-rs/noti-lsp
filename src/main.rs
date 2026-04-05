mod ast;
mod backend;
mod consts;
mod document;
mod features;
mod parser;
mod schema;
mod utils;

use crate::backend::Backend;
use dashmap::DashMap;
use tower_lsp::{LspService, Server};

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        docs: DashMap::new(),
    });

    Server::new(stdin, stdout, socket).serve(service).await;
}
