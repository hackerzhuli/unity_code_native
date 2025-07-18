//! USS Language Server Implementation
//!
//! Provides Language Server Protocol features for USS files using tower-lsp.

use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use url::Url;

use crate::language::asset_url::project_url_to_path;
use crate::unity_project_manager::UnityProjectManager;
use crate::uss::color_provider::UssColorProvider;
use crate::uss::completion::UssCompletionProvider;
use crate::uss::constants::*;
use crate::uss::diagnostics::UssDiagnostics;
use crate::uss::document_manager::UssDocumentManager;
use crate::uss::formatter::UssFormatter;
use crate::uss::highlighting::UssHighlighter;
use crate::uss::hover::UssHoverProvider;
use crate::uss::refactor::UssRefactorProvider;
use crate::uxml_schema_manager::UxmlSchemaManager;

/// USS Language Server
pub struct UssLanguageServer {
    client: Client,
    /// Arc<Mutex> is required here despite single-threaded async for three reasons:
    /// 1. tower-lsp requires LanguageServer implementations to be Send + Sync
    /// 2. Interior mutability is needed to modify state from &self methods
    /// 3. Async method boundaries require thread-safe primitives even in single-threaded context
    state: Arc<Mutex<UssServerState>>,
    uxml_schema_manager: Arc<tokio::sync::Mutex<UxmlSchemaManager>>,
}

/// Internal state for the USS language server
struct UssServerState {
    document_manager: UssDocumentManager,
    highlighter: UssHighlighter,
    diagnostics: UssDiagnostics,
    hover_provider: UssHoverProvider,
    color_provider: UssColorProvider,
    completion_provider: UssCompletionProvider,
    formatter: UssFormatter,
    refactor_provider: UssRefactorProvider,
    unity_manager: UnityProjectManager,
}

impl UssLanguageServer {
    /// Create a new USS language server
    pub fn new(client: Client, project_path: std::path::PathBuf, uxml_schema_manager: UxmlSchemaManager) -> Self {
        let state = UssServerState {
            document_manager: UssDocumentManager::new()
                .expect("Failed to create USS document manager"),
            highlighter: UssHighlighter::new(),
            diagnostics: UssDiagnostics::new(),
            hover_provider: UssHoverProvider::new(),
            color_provider: UssColorProvider::new(),
            completion_provider: UssCompletionProvider::new_with_project_root(&project_path),
            formatter: UssFormatter::new(),
            refactor_provider: UssRefactorProvider::new(),
            unity_manager: UnityProjectManager::new(project_path.clone()),
        };

        Self {
            uxml_schema_manager: Arc::new(tokio::sync::Mutex::new(uxml_schema_manager)),
            client,
            state: Arc::new(Mutex::new(state)),
        }
    }

    /// Open and parse a new document
    async fn open_document(&self, uri: &Url, content: &str, version: i32) {
        if let Ok(mut state) = self.state.lock() {
            state
                .document_manager
                .open_document(uri.clone(), content.to_string(), version);
            let project_url = state.unity_manager.convert_to_project_url(&uri);
            if project_url.is_none() {
                log::warn!("[open_document] Failed to convert URI to project URL");
            }

            if let Some(document) = state.document_manager.get_document_mut(uri) {
                document.extract_variables_with_source_url(project_url.as_ref());
            } else {
                log::warn!("[open_document] Document not found in manager after opening");
            }
        } else {
            log::error!("[open_document] Failed to acquire state lock");
        }
    }

    /// Update a document with incremental changes
    async fn update_document(
        &self,
        uri: &Url,
        changes: Vec<TextDocumentContentChangeEvent>,
        version: i32,
    ) {
        if let Ok(mut state) = self.state.lock() {
            state
                .document_manager
                .update_document(uri, changes, version);
            let project_url = state.unity_manager.convert_to_project_url(&uri);
            if project_url.is_none() {
                log::warn!("[update_document] Failed to convert URI to project URL");
            }

            if let Some(document) = state.document_manager.get_document_mut(uri) {
                document.extract_variables_with_source_url(project_url.as_ref());
            } else {
                log::warn!("[update_document] Document not found in manager after update");
            }
        } else {
            log::error!("[update_document] Failed to acquire state lock");
        }
    }

    /// Generate semantic tokens for syntax highlighting
    fn generate_semantic_tokens(&self, uri: &Url) -> Option<Vec<SemanticToken>> {
        let state = self.state.lock().ok()?;
        let document = state.document_manager.get_document(uri)?;
        let tree = document.tree()?;
        let content = document.content();

        Some(state.highlighter.generate_tokens(tree, content))
    }

    /// Extract Visual Element class names from the schema manager
    async fn update_and_get_element_names(&self) -> HashSet<String> {
        let mut manager = self.uxml_schema_manager.lock().await;
        if let Err(e) = manager.update().await {
            log::warn!("Failed to update UXML schemas: {}", e);
        }
        manager.get_all_elements().iter().map(|element| element.name.clone()).collect::<HashSet<String>>()
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
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![
                        ":".to_string(), // for property values and pseudo classes
                        ",".to_string(), // for properties with multiple values(ie. comma seperated values)
                        "/".to_string(), // for url completion
                        "?".to_string(), // for query parameters in url
                        "@".to_string() // for import statement
                    ]),
                    all_commit_characters: None,
                    work_done_progress_options: WorkDoneProgressOptions::default(),
                    completion_item: None,
                }),
                document_formatting_provider: Some(OneOf::Left(true)),
                document_range_formatting_provider: Some(OneOf::Left(true)),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                rename_provider: Some(OneOf::Right(RenameOptions {
                    prepare_provider: Some(true),
                    work_done_progress_options: WorkDoneProgressOptions::default(),
                })),
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

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        // Get UXML element information
        let uxml_elements = {
            let mut manager = self.uxml_schema_manager.lock().await;
            if let Err(e) = manager.update().await {
                log::warn!("Failed to update UXML schemas: {}", e);
            }
            manager.get_all_elements().iter().map(|element| {
                (element.name.clone(), element.fully_qualified_name.clone())
            }).collect::<std::collections::HashMap<String, String>>()
        };

        let state = self.state.lock().ok();
        if let Some(state) = state {
            if let Some(document) = state.document_manager.get_document(&uri) {
                if let Some(tree) = document.tree() {
                    let project_url = state.unity_manager.convert_to_project_url(&uri);

                    let hover = state.hover_provider.hover(
                        tree,
                        document.content(),
                        position,
                        &state.unity_manager,
                        project_url.as_ref(),
                        Some(&uxml_elements),
                    );
                    return Ok(hover);
                }
            }
        }

        Ok(None)
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

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        // Debug logging
        log::info!(
            "Completion requested at {}:{} in {}",
            position.line,
            position.character,
            uri
        );

        let uxml_class_names = self.update_and_get_element_names().await;

        // Perform all operations within a single lock scope
        let completions = {
            if let Ok(state) = self.state.lock() {
                // Get document and validate
                let document = match state.document_manager.get_document(&uri) {
                    Some(doc) => doc,
                    None => {
                        log::warn!("Document not found for URI: {}", uri);
                        return Ok(None);
                    }
                };

                let tree = match document.tree() {
                    Some(tree) => tree,
                    None => {
                        log::warn!("No syntax tree available for URI: {}", uri);
                        return Ok(None);
                    }
                };

                let document_content = document.content();
                let project_url = state.unity_manager.convert_to_project_url(&uri);

                // Generate completions
                state.completion_provider.complete(
                    tree,
                    document_content,
                    position,
                    project_url.as_ref(),
                    Some(&uxml_class_names),
                    Some(&state.unity_manager),
                )
            } else {
                log::error!("Failed to lock state");
                return Ok(None);
            }
        };

        // Debug: log completion results
        log::info!("Generated {} completion items", completions.len());

        if completions.is_empty() {
            log::info!("Returning no completions");
            Ok(None)
        } else {
            log::info!("Returning {} completions", completions.len());
            Ok(Some(CompletionResponse::Array(completions)))
        }
    }

    async fn diagnostic(
        &self,
        params: DocumentDiagnosticParams,
    ) -> Result<DocumentDiagnosticReportResult> {
        let uri = params.text_document.uri;

        let uxml_class_names = self.update_and_get_element_names().await;

        // Extract necessary data from state and release lock quickly
        let (mut diagnostics, url_references, _, project_root) = {
            if let Ok(state) = self.state.lock() {
                // Generate diagnostics immediately
                let (tree_clone, content, doc_version) =
                    if let Some(document) = state.document_manager.get_document(&uri) {
                        if let Some(tree) = document.tree() {
                            (
                                Some(tree.clone()),
                                document.content().to_string(),
                                document.document_version(),
                            )
                        } else {
                            (None, String::new(), document.document_version())
                        }
                    } else {
                        (
                            None,
                            String::new(),
                            crate::language::document::DocumentVersion { major: 0, minor: 0 },
                        )
                    };

                if let Some(tree) = tree_clone {
                    let project_url = state.unity_manager.convert_to_project_url(&uri);

                    // Get the variable resolver from the document for enhanced diagnostics
                    let variable_resolver =
                        if let Some(document) = state.document_manager.get_document(&uri) {
                            Some(&document.variable_resolver)
                        } else {
                            None
                        };

                    let (diagnostics, url_references) = state.diagnostics.analyze_with_variables_and_classes(
                        &tree,
                        &content,
                        project_url.as_ref(),
                        variable_resolver,
                        Some(&uxml_class_names),
                    );

                    let project_root = state.unity_manager.project_path().clone();

                    (diagnostics, url_references, doc_version, project_root)
                } else {
                    (
                        Vec::new(),
                        Vec::new(),
                        doc_version,
                        state.unity_manager.project_path().clone(),
                    )
                }
            } else {
                (
                    Vec::new(),
                    Vec::new(),
                    crate::language::document::DocumentVersion { major: 0, minor: 0 },
                    std::path::PathBuf::new(),
                )
            }
        }; // Lock is released here

        // Perform async asset validation outside the lock (inline, no task spawning)
        for url_ref in &url_references {
            // Handle project:// URLs manually since to_file_path() doesn't work with custom schemes
            if url_ref.url.scheme() == PROJECT_SCHEME {
                if let Some(full_path) = project_url_to_path(&project_root, &url_ref.url) {
                    // Check if the asset file exists using async try_exists for better error handling
                    match tokio::fs::try_exists(&full_path).await {
                        Ok(false) => {
                            diagnostics.push(Diagnostic {
                                range: url_ref.range,
                                severity: Some(DiagnosticSeverity::WARNING),
                                code: Some(NumberOrString::String("asset-not-found".to_string())),
                                source: Some("uss".to_string()),
                                message: format!(
                                    "Asset doesn't exist on path: {}",
                                    full_path.display()
                                ),
                                ..Default::default()
                            });
                        }
                        Err(e) => {
                            // Log the error but don't create a diagnostic for permission/access issues
                            log::debug!(
                                "Cannot check asset existence for {}: {}",
                                full_path.display(),
                                e
                            );
                        }
                        Ok(true) => {
                            // File exists, no diagnostic needed
                        }
                    }
                }
            }
        }

        // Asset validation is now performed synchronously above and included in diagnostics

        Ok(DocumentDiagnosticReportResult::Report(
            DocumentDiagnosticReport::Full(RelatedFullDocumentDiagnosticReport {
                related_documents: None,
                full_document_diagnostic_report: FullDocumentDiagnosticReport {
                    result_id: None,
                    items: diagnostics,
                },
            }),
        ))
    }

    async fn document_color(&self, params: DocumentColorParams) -> Result<Vec<ColorInformation>> {
        let uri = params.text_document.uri;

        let colors = if let Ok(state) = self.state.lock() {
            if let Some(document) = state.document_manager.get_document(&uri) {
                if let Some(tree) = document.tree() {
                    state
                        .color_provider
                        .provide_document_colors(tree, document.content())
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
            let presentations = state
                .color_provider
                .provide_color_presentations(&params.color, params.range);
            Ok(presentations)
        } else {
            Ok(Vec::new())
        }
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = params.text_document.uri;

        let result = if let Ok(state) = self.state.lock() {
            if let Some(document) = state.document_manager.get_document(&uri) {
                if let Some(tree) = document.tree() {
                    match state.formatter.format_document(document.content(), tree) {
                        Ok(edits) => {
                            if edits.is_empty() {
                                log::debug!("No formatting changes needed for {}", uri);
                                None
                            } else {
                                log::info!("Applied {} formatting edits to {}", edits.len(), uri);
                                Some(edits)
                            }
                        }
                        Err(e) => {
                            log::warn!("Failed to format document {}: {}", uri, e);
                            None
                        }
                    }
                } else {
                    log::warn!("No syntax tree available for formatting: {}", uri);
                    None
                }
            } else {
                log::warn!("Document not found for formatting: {}", uri);
                None
            }
        } else {
            log::error!("Failed to acquire state lock for formatting");
            None
        };

        Ok(result)
    }

    async fn range_formatting(
        &self,
        params: DocumentRangeFormattingParams,
    ) -> Result<Option<Vec<TextEdit>>> {
        let uri = params.text_document.uri;
        let range = params.range;

        let result = if let Ok(state) = self.state.lock() {
            if let Some(document) = state.document_manager.get_document(&uri) {
                if let Some(tree) = document.tree() {
                    match state.formatter.format_range(document.content(), tree, range) {
                        Ok(edits) => {
                            if edits.is_empty() {
                                log::debug!("No range formatting changes needed for {} at {:?}", uri, range);
                                None
                            } else {
                                log::info!("Applied {} range formatting edits to {} at {:?}", edits.len(), uri, range);
                                Some(edits)
                            }
                        }
                        Err(e) => {
                            log::warn!("Failed to format range in document {}: {}", uri, e);
                            None
                        }
                    }
                } else {
                    log::warn!("No syntax tree available for range formatting: {}", uri);
                    None
                }
            } else {
                log::warn!("Document not found for range formatting: {}", uri);
                None
            }
        } else {
            log::error!("Failed to acquire state lock for range formatting");
            None
        };

        Ok(result)
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let uri = params.text_document.uri;
        let range = params.range;
        
        if let Ok(state) = self.state.lock() {
            if let Some(document) = state.document_manager.get_document(&uri) {
                if let Some(tree) = document.tree() {
                    if let Some(actions) = state.refactor_provider.get_code_actions(tree, document.content(), &uri, range) {
                        return Ok(Some(CodeActionResponse::from(actions)));
                    }
                }
            }
        }
        
        Ok(None)
    }

    async fn prepare_rename(&self, params: TextDocumentPositionParams) -> Result<Option<PrepareRenameResponse>> {
        let uri = params.text_document.uri;
        let position = params.position;
        
        if let Ok(state) = self.state.lock() {
            if let Some(document) = state.document_manager.get_document(&uri) {
                if let Some(tree) = document.tree() {
                    return Ok(state.refactor_provider.prepare_rename(tree.root_node(), document.content(), position));

                }
            }
        }
        
        Ok(None)
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let new_name = params.new_name;
        
        if let Ok(state) = self.state.lock() {
            if let Some(document) = state.document_manager.get_document(&uri) {
                if let Some(tree) = document.tree() {
                    return Ok(state.refactor_provider.handle_rename(tree.root_node(), document.content(), &uri, position, &new_name));
                }
            }
        }
        
        Ok(None)
    }
}

/// Create and start the USS language server
pub async fn start_uss_language_server(project_path: std::path::PathBuf, uxml_schema_manager: UxmlSchemaManager) -> Result<()> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) =
        LspService::new(|client| UssLanguageServer::new(client, project_path.clone(), uxml_schema_manager));
    Server::new(stdin, stdout, socket).serve(service).await;

    Ok(())
}
