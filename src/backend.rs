use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::{LSP_NAME, LSP_VERSION};

#[derive(Debug)]
pub struct Backend {
    pub client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                // We'll fill these in as we add features
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: LSP_NAME.to_string(),
                version: Some(LSP_VERSION.to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, format!("{} initialized!", LSP_NAME))
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}
