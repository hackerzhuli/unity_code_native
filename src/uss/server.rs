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
use crate::uss::diagnostics::UssDiagnostics;
use crate::uss::hover::UssHoverProvider;
use crate::uss::color_provider::UssColorProvider;
use crate::unity_project_manager::UnityProjectManager;

/// USS Language Server
pub struct UssLanguageServer {
    client: Client,
    state: Arc<Mutex<UssServerState>>,
}

/// Internal state for the USS language server
struct UssServerState {
    parser: UssParser,
    highlighter: UssHighlighter,
    diagnostics: UssDiagnostics,
    hover_provider: UssHoverProvider,
    color_provider: UssColorProvider,
    unity_manager: UnityProjectManager,
    document_trees: HashMap<Url, Tree>,
    document_content: HashMap<Url, String>,
}

impl UssLanguageServer {
    /// Create a new USS language server
    pub fn new(client: Client, project_path: std::path::PathBuf) -> Self {
        let state = UssServerState {
            parser: UssParser::new().expect("Failed to create USS parser"),
            highlighter: UssHighlighter::new(),
            diagnostics: UssDiagnostics::new(),
            hover_provider: UssHoverProvider::new(),
            color_provider: UssColorProvider::new(),
            unity_manager: UnityProjectManager::new(project_path),
            document_trees: HashMap::new(),
            document_content: HashMap::new(),
        };
        
        Self {
            client,
            state: Arc::new(Mutex::new(state)),
        }
    }
    
    /// Parse document and store the syntax tree
    async fn parse_document(&self, uri: &Url, content: &str) {
        // Parse without incremental parsing for now to avoid borrowing conflicts
        let new_tree = {
            if let Ok(mut state) = self.state.lock() {
                state.parser.parse(content, None)
            } else {
                None
            }
        };
        
        // Store the result and generate diagnostics in a separate lock scope
        if let Some(tree) = new_tree {
            let diagnostics = {
                if let Ok(mut state) = self.state.lock() {
                    state.document_trees.insert(uri.clone(), tree.clone());
                    state.document_content.insert(uri.clone(), content.to_string());
                    state.diagnostics.analyze(&tree, content)
                } else {
                    Vec::new()
                }
            };
            
            // Publish diagnostics
            self.client.publish_diagnostics(uri.clone(), diagnostics, None).await;
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
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions {
                        identifier: Some("uss".to_string()),
                        inter_file_dependencies: false,
                        workspace_diagnostics: false,
                        ..Default::default()
                    },
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                color_provider: Some(ColorProviderCapability::Simple(true)),
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
        
        // Parse the document and generate diagnostics
        self.parse_document(&uri, &content).await;
        
        self.client
            .log_message(MessageType::INFO, format!("Opened USS document: {}", uri))
            .await;
    }
    
    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        
        if let Some(change) = params.content_changes.into_iter().next() {
            // Re-parse the document with new content and generate diagnostics
            self.parse_document(&uri, &change.text).await;
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
    
    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        
        let state = self.state.lock().ok();
        if let Some(state) = state {
            if let (Some(tree), Some(content)) = (
                state.document_trees.get(&uri),
                state.document_content.get(&uri)
            ) {
                let hover = state.hover_provider.hover(
                    tree,
                    content,
                    position,
                    &state.unity_manager
                );
                return Ok(hover);
            }
        }
        
        Ok(None)
    }
    
    async fn diagnostic(
        &self,
        params: DocumentDiagnosticParams,
    ) -> Result<DocumentDiagnosticReportResult> {
        let uri = params.text_document.uri;
        
        let diagnostics = if let Ok(state) = self.state.lock() {
            if let (Some(tree), Some(content)) = (
                state.document_trees.get(&uri),
                state.document_content.get(&uri)
            ) {
                state.diagnostics.analyze(tree, content)
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };
        
        Ok(DocumentDiagnosticReportResult::Report(
            DocumentDiagnosticReport::Full(
                RelatedFullDocumentDiagnosticReport {
                    related_documents: None,
                    full_document_diagnostic_report: FullDocumentDiagnosticReport {
                        result_id: None,
                        items: diagnostics,
                    },
                }
            )
        ))
    }
    
    async fn document_color(&self, params: DocumentColorParams) -> Result<Vec<ColorInformation>> {
        let uri = params.text_document.uri;
        
        let colors = if let Ok(state) = self.state.lock() {
            if let (Some(tree), Some(content)) = (
                state.document_trees.get(&uri),
                state.document_content.get(&uri)
            ) {
                state.color_provider.provide_document_colors(tree, content)
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };
        
        Ok(colors)
    }
    
    async fn color_presentation(
        &self,
        params: ColorPresentationParams,
    ) -> Result<Vec<ColorPresentation>> {
        let state = self.state.lock().ok();
        if let Some(state) = state {
            let presentations = state.color_provider.provide_color_presentations(
                &params.color,
                params.range,
            );
            Ok(presentations)
        } else {
            Ok(Vec::new())
        }
    }
}

/// Create and start the USS language server
pub async fn start_uss_language_server(project_path: std::path::PathBuf) -> Result<()> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    
    let (service, socket) = LspService::new(|client| UssLanguageServer::new(client, project_path.clone()));
    Server::new(stdin, stdout, socket).serve(service).await;
    
    Ok(())
}