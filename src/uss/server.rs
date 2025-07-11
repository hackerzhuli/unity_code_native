//! USS Language Server Implementation
//!
//! Provides Language Server Protocol features for USS files using tower-lsp.

use std::sync::{Arc, Mutex};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use url::Url;

use crate::uss::document_manager::UssDocumentManager;
use crate::uss::highlighting::UssHighlighter;
use crate::uss::diagnostics::UssDiagnostics;
use crate::uss::hover::UssHoverProvider;
use crate::uss::color_provider::UssColorProvider;
use crate::unity_project_manager::UnityProjectManager;
use crate::language::asset_url;

/// USS Language Server
pub struct UssLanguageServer {
    client: Client,
    /// Arc<Mutex> is required here despite single-threaded async for three reasons:
    /// 1. tower-lsp requires LanguageServer implementations to be Send + Sync
    /// 2. Interior mutability is needed to modify state from &self methods
    /// 3. Async method boundaries require thread-safe primitives even in single-threaded context
    state: Arc<Mutex<UssServerState>>,
}

/// Internal state for the USS language server
struct UssServerState {
    document_manager: UssDocumentManager,
    highlighter: UssHighlighter,
    diagnostics: UssDiagnostics,
    hover_provider: UssHoverProvider,
    color_provider: UssColorProvider,
    unity_manager: UnityProjectManager,
}

impl UssLanguageServer {
    /// Create a new USS language server
    pub fn new(client: Client, project_path: std::path::PathBuf) -> Self {
        let state = UssServerState {
            document_manager: UssDocumentManager::new().expect("Failed to create USS document manager"),
            highlighter: UssHighlighter::new(),
            diagnostics: UssDiagnostics::new(),
            hover_provider: UssHoverProvider::new(),
            color_provider: UssColorProvider::new(),
            unity_manager: UnityProjectManager::new(project_path),
        };
        
        Self {
            client,
            state: Arc::new(Mutex::new(state)),
        }
    }
    
    /// Open and parse a new document
    async fn open_document(&self, uri: &Url, content: &str, version: i32) {
        log::info!("[open_document] Starting to open document: {}", uri);
        
        if let Ok(mut state) = self.state.lock() {
            state.document_manager.open_document(uri.clone(), content.to_string(), version);
            log::info!("[open_document] Document opened in manager");
            
            // Extract variables with proper source URL for relative URL resolution
            let project_url = if uri.scheme() == "file" {
                log::info!("[open_document] URI scheme is 'file', attempting to convert to project URL");
                if let Ok(file_path) = uri.to_file_path() {
                    log::info!("[open_document] File path extracted: {:?}", file_path);
                    let project_root = state.unity_manager.project_path();
                    log::info!("[open_document] Project root: {:?}", project_root);
                    
                    match asset_url::create_project_url_with_normalization(&file_path, &project_root) {
                        Ok(url) => {
                            log::info!("[open_document] Successfully created project URL: {}", url);
                            Some(url)
                        }
                        Err(e) => {
                            log::warn!("[open_document] Failed to create project URL: {}", e);
                            None
                        }
                    }
                } else {
                    log::warn!("[open_document] Failed to convert URI to file path");
                    None
                }
            } else {
                log::info!("[open_document] URI scheme is '{}', using URI directly as project URL", uri.scheme());
                Some(uri.clone())
            };
            
            log::info!("[open_document] Final project_url for variable extraction: {:?}", project_url);
            
            if let Some(document) = state.document_manager.get_document_mut(uri) {
                log::info!("[open_document] Document found, extracting variables with source URL");
                document.extract_variables_with_source_url(project_url.as_ref());
                log::info!("[open_document] Variable extraction completed");
            } else {
                log::warn!("[open_document] Document not found in manager after opening");
            }
        } else {
            log::error!("[open_document] Failed to acquire state lock");
        }
        
        log::info!("[open_document] Completed opening document: {}", uri);
    }
    
    /// Update a document with incremental changes
    async fn update_document(&self, uri: &Url, changes: Vec<TextDocumentContentChangeEvent>, version: i32) {
        log::info!("[update_document] Starting to update document: {}", uri);
        log::info!("[update_document] Number of changes: {}, version: {}", changes.len(), version);
        
        if let Ok(mut state) = self.state.lock() {
            state.document_manager.update_document(uri, changes, version);
            log::info!("[update_document] Document updated in manager");
            
            // Re-extract variables with proper source URL after changes
            let project_url = if uri.scheme() == "file" {
                log::info!("[update_document] URI scheme is 'file', attempting to convert to project URL");
                if let Ok(file_path) = uri.to_file_path() {
                    log::info!("[update_document] File path extracted: {:?}", file_path);
                    let project_root = state.unity_manager.project_path();
                    log::info!("[update_document] Project root: {:?}", project_root);
                    
                    match asset_url::create_project_url_with_normalization(&file_path, &project_root) {
                        Ok(url) => {
                            log::info!("[update_document] Successfully created project URL: {}", url);
                            Some(url)
                        }
                        Err(e) => {
                            log::warn!("[update_document] Failed to create project URL: {}", e);
                            None
                        }
                    }
                } else {
                    log::warn!("[update_document] Failed to convert URI to file path");
                    None
                }
            } else {
                log::info!("[update_document] URI scheme is '{}', using URI directly as project URL", uri.scheme());
                Some(uri.clone())
            };
            
            log::info!("[update_document] Final project_url for variable extraction: {:?}", project_url);
            
            if let Some(document) = state.document_manager.get_document_mut(uri) {
                log::info!("[update_document] Document found, re-extracting variables with source URL");
                document.extract_variables_with_source_url(project_url.as_ref());
                log::info!("[update_document] Variable re-extraction completed");
            } else {
                log::warn!("[update_document] Document not found in manager after update");
            }
        } else {
            log::error!("[update_document] Failed to acquire state lock");
        }
        
        log::info!("[update_document] Completed updating document: {}", uri);
    }
    
    /// Generate semantic tokens for syntax highlighting
    fn generate_semantic_tokens(&self, uri: &Url) -> Option<Vec<SemanticToken>> {
        let state = self.state.lock().ok()?;
        let document = state.document_manager.get_document(uri)?;
        let tree = document.tree()?;
        let content = document.content();
        
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
                    TextDocumentSyncKind::INCREMENTAL,
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
        let version = params.text_document.version;
        
        // Open and parse the document
        self.open_document(&uri, &content, version).await;
        
        self.client
            .log_message(MessageType::INFO, format!("Opened USS document: {}", uri))
            .await;
    }
    
    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;
        let changes = params.content_changes;
        
        // Update the document with incremental changes
        self.update_document(&uri, changes, version).await;
    }
    
    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        
        if let Ok(mut state) = self.state.lock() {
            state.document_manager.close_document(&uri);
        }
        
        self.client
            .log_message(MessageType::INFO, format!("Closed USS document: {}", uri))
            .await;
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
            if let Some(document) = state.document_manager.get_document(&uri) {
                if let Some(tree) = document.tree() {
                    let hover = state.hover_provider.hover(
                        tree,
                        document.content(),
                        position,
                        &state.unity_manager
                    );
                    return Ok(hover);
                }
            }
        }
        
        Ok(None)
    }
    
    async fn diagnostic(
        &self,
        params: DocumentDiagnosticParams,
    ) -> Result<DocumentDiagnosticReportResult> {
        let uri = params.text_document.uri;
        
        let diagnostics = if let Ok(mut state) = self.state.lock() {
            // First, check if we have cached diagnostics
            let cached_result = if let Some(document) = state.document_manager.get_document(&uri) {
                document.get_cached_diagnostics().cloned()
            } else {
                None
            };
            
            if let Some(cached_diagnostics) = cached_result {
                cached_diagnostics
            } else {
                // Need to generate new diagnostics
                let (tree_clone, content) = if let Some(document) = state.document_manager.get_document(&uri) {
                    if let Some(tree) = document.tree() {
                        (Some(tree.clone()), document.content().to_string())
                    } else {
                        (None, String::new())
                    }
                } else {
                    (None, String::new())
                };
                
                if let Some(tree) = tree_clone {
                    // Convert file system URI to project scheme URL for Unity compatibility
                    let project_url = if uri.scheme() == "file" {
                        // Convert file:// URI to project:// URI
                        if let Ok(file_path) = uri.to_file_path() {
                            let project_root = state.unity_manager.project_path();
                            asset_url::create_project_url_with_normalization(&file_path, &project_root).ok()
                        } else {
                            None
                        }
                    } else {
                        // If it's already a project:// URI or other scheme, use as-is
                        Some(uri.clone())
                    };
                    
                    // Get the variable resolver from the document for enhanced diagnostics
                    let variable_resolver = if let Some(document) = state.document_manager.get_document(&uri) {
                        Some(&document.variable_resolver)
                    } else {
                        None
                    };
                    
                    let (new_diagnostics, _url_references) = state.diagnostics.analyze_with_variables(&tree, &content, project_url.as_ref(), variable_resolver);
                    
                    // Cache the diagnostics
                    if let Some(document) = state.document_manager.get_document_mut(&uri) {
                        document.cache_diagnostics(new_diagnostics.clone());
                    }
                    
                    new_diagnostics
                } else {
                    Vec::new()
                }
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
            if let Some(document) = state.document_manager.get_document(&uri) {
                if let Some(tree) = document.tree() {
                    state.color_provider.provide_document_colors(tree, document.content())
                } else {
                    Vec::new()
                }
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