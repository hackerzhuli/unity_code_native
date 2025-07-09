//! USS Language Server Implementation
//!
//! Provides Language Server Protocol features for USS files using tower-lsp.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use tree_sitter::Tree;

use crate::uss::parser::UssParser;
use crate::uss::highlighting::UssHighlighter;

/// USS Language Server
pub struct UssLanguageServer {
    client: Client,
    state: Arc<Mutex<UssServerState>>,
}

/// Internal state for the USS language server
struct UssServerState {
    parser: UssParser,
    highlighter: UssHighlighter,
    document_trees: HashMap<Url, Tree>,
    document_content: HashMap<Url, String>,
}

impl UssLanguageServer {
    /// Create a new USS language server
    pub fn new(client: Client) -> Self {
        let state = UssServerState {
            parser: UssParser::new().expect("Failed to create USS parser"),
            highlighter: UssHighlighter::new(),
            document_trees: HashMap::new(),
            document_content: HashMap::new(),
        };
        
        Self {
            client,
            state: Arc::new(Mutex::new(state)),
        }
    }
    
    /// Parse document and store the syntax tree
    fn parse_document(&self, uri: &Url, content: &str) {
        // Parse without incremental parsing for now to avoid borrowing conflicts
        let new_tree = {
            if let Ok(mut state) = self.state.lock() {
                state.parser.parse(content, None)
            } else {
                None
            }
        };
        
        // Store the result in a separate lock scope
        if let Some(tree) = new_tree {
            if let Ok(mut state) = self.state.lock() {
                state.document_trees.insert(uri.clone(), tree);
                state.document_content.insert(uri.clone(), content.to_string());
            }
        }
    }
    
    /// Generate semantic tokens for syntax highlighting
    fn generate_semantic_tokens(&self, uri: &Url) -> Option<Vec<SemanticToken>> {
        let state = self.state.lock().ok()?;
        let tree = state.document_trees.get(uri)?;
        let content = state.document_content.get(uri)?;
        
        Some(state.highlighter.generate_tokens(tree, content))
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for UssLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        let legend = if let Ok(state) = self.state.lock() {
            state.highlighter.legend.clone()
        } else {
            // Fallback legend if state is locked
            UssHighlighter::new().legend
        };
        
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            legend,
                            range: Some(false),
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                            ..Default::default()
                        },
                    ),
                ),
                ..Default::default()
            },
            ..Default::default()
        })
    }
    
    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "USS Language Server initialized")
            .await;
    }
    
    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
    
    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let content = params.text_document.text;
        
        // Parse the document
        self.parse_document(&uri, &content);
        
        self.client
            .log_message(MessageType::INFO, format!("Opened USS document: {}", uri))
            .await;
    }
    
    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        
        if let Some(change) = params.content_changes.into_iter().next() {
            // Re-parse the document with new content
            self.parse_document(&uri, &change.text);
        }
    }
    
    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let uri = params.text_document.uri;
        
        if let Some(tokens) = self.generate_semantic_tokens(&uri) {
            Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
                result_id: None,
                data: tokens,
            })))
        } else {
            Ok(None)
        }
    }
}

/// Create and start the USS language server
pub async fn start_uss_language_server() -> Result<()> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    
    let (service, socket) = LspService::new(UssLanguageServer::new);
    Server::new(stdin, stdout, socket).serve(service).await;
    
    Ok(())
}