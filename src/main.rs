mod backend;
use backend::Backend;
use tower_lsp::{LspService, Server};

const LSP_NAME: &str = env!("CARGO_PKG_NAME");
const LSP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend { client });

    Server::new(stdin, stdout, socket).serve(service).await;
}
