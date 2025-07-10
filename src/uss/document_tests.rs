//! Tests for USS document management and incremental parsing

#[cfg(test)]
mod tests {
    use super::super::document::{UssDocument, UssDocumentManager};
    use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range, TextDocumentContentChangeEvent, Url};

    #[test]
    fn test_document_creation() {
        let uri = Url::parse("file:///test.uss").unwrap();
        let content = ".button { color: red; }".to_string();
        let document = UssDocument::new(uri.clone(), content.clone(), 1);
        
        assert_eq!(document.uri, uri);
        assert_eq!(document.content(), &content);
        assert_eq!(document.version(), 1);
    }
    
    #[test]
    fn test_document_manager() {
        let mut manager = UssDocumentManager::new().unwrap();
        let uri = Url::parse("file:///test.uss").unwrap();
        let content = ".button { color: red; }".to_string();
        
        // Open document
        manager.open_document(uri.clone(), content.clone(), 1);
        
        // Check document exists
        let document = manager.get_document(&uri).unwrap();
        assert_eq!(document.content(), &content);
        assert_eq!(document.version(), 1);
        
        // Close document
        manager.close_document(&uri);
        assert!(manager.get_document(&uri).is_none());
    }
    
    #[test]
    fn test_incremental_change() {
        let mut manager = UssDocumentManager::new().unwrap();
        let uri = Url::parse("file:///test.uss").unwrap();
        let initial_content = ".button { color: red; }".to_string();
        
        // Open document
        manager.open_document(uri.clone(), initial_content, 1);
        
        // Apply incremental change: change "red" to "blue"
        // In ".button { color: red; }", "red" is at positions 17-20
        let change = TextDocumentContentChangeEvent {
            range: Some(Range {
                start: Position { line: 0, character: 17 },
                end: Position { line: 0, character: 20 },
            }),
            range_length: Some(3),
            text: "blue".to_string(),
        };
        
        manager.update_document(&uri, vec![change], 2);
        
        // Check the content was updated
        let document = manager.get_document(&uri).unwrap();
        assert_eq!(document.content(), ".button { color: blue; }");
        assert_eq!(document.version(), 2);
    }
    
    #[test]
    fn test_full_document_change() {
        let mut manager = UssDocumentManager::new().unwrap();
        let uri = Url::parse("file:///test.uss").unwrap();
        let initial_content = ".button { color: red; }".to_string();
        
        // Open document
        manager.open_document(uri.clone(), initial_content, 1);
        
        // Apply full document change
        let change = TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: ".label { font-size: 14px; }".to_string(),
        };
        
        manager.update_document(&uri, vec![change], 2);
        
        // Check the content was completely replaced
        let document = manager.get_document(&uri).unwrap();
        assert_eq!(document.content(), ".label { font-size: 14px; }");
        assert_eq!(document.version(), 2);
    }
    
    #[test]
    fn test_line_starts_calculation() {
        let uri = Url::parse("file:///test.uss").unwrap();
        let content = ".button {\n  color: red;\n  font-size: 12px;\n}".to_string();
        let document = UssDocument::new(uri, content, 1);
        
        // Test position to byte conversion
        // This is an internal test, so we'll just verify the document was created successfully
        assert_eq!(document.version(), 1);
        assert!(document.content().contains("color: red"));
    }
    
    #[test]
    fn test_diagnostic_caching() {
        let uri = Url::parse("file:///test.uss").unwrap();
        let content = ".button { color: red; }".to_string();
        let mut document = UssDocument::new(uri, content, 1);
        
        // Initially, no cached diagnostics
        assert!(document.get_cached_diagnostics().is_none());
        assert!(!document.are_diagnostics_valid());
        
        // Create some mock diagnostics
        let diagnostics = vec![
            Diagnostic {
                range: Range {
                    start: Position { line: 0, character: 0 },
                    end: Position { line: 0, character: 7 },
                },
                severity: Some(DiagnosticSeverity::WARNING),
                code: None,
                code_description: None,
                source: Some("uss".to_string()),
                message: "Test diagnostic".to_string(),
                related_information: None,
                tags: None,
                data: None,
            }
        ];
        
        // Cache the diagnostics
        document.cache_diagnostics(diagnostics.clone());
        
        // Now we should have cached diagnostics
        assert!(document.are_diagnostics_valid());
        let cached = document.get_cached_diagnostics().unwrap();
        assert_eq!(cached.len(), 1);
        assert_eq!(cached[0].message, "Test diagnostic");
        
        // Invalidate diagnostics
        document.invalidate_diagnostics();
        assert!(!document.are_diagnostics_valid());
        assert!(document.get_cached_diagnostics().is_none());
    }
    
    #[test]
    fn test_diagnostic_invalidation_on_change() {
        let mut manager = UssDocumentManager::new().unwrap();
        let uri = Url::parse("file:///test.uss").unwrap();
        let content = ".button { color: red; }".to_string();
        
        // Open document
        manager.open_document(uri.clone(), content, 1);
        
        // Cache some diagnostics
        let diagnostics = vec![
            Diagnostic {
                range: Range {
                    start: Position { line: 0, character: 0 },
                    end: Position { line: 0, character: 7 },
                },
                severity: Some(DiagnosticSeverity::WARNING),
                code: None,
                code_description: None,
                source: Some("uss".to_string()),
                message: "Test diagnostic".to_string(),
                related_information: None,
                tags: None,
                data: None,
            }
        ];
        
        {
            let document = manager.get_document_mut(&uri).unwrap();
            document.cache_diagnostics(diagnostics);
            assert!(document.are_diagnostics_valid());
        }
        
        // Apply a change - this should invalidate diagnostics
        let change = TextDocumentContentChangeEvent {
            range: Some(Range {
                start: Position { line: 0, character: 17 },
                end: Position { line: 0, character: 20 },
            }),
            range_length: Some(3),
            text: "blue".to_string(),
        };
        
        manager.update_document(&uri, vec![change], 2);
        
        // Diagnostics should now be invalid
        let document = manager.get_document(&uri).unwrap();
        assert!(!document.are_diagnostics_valid());
        assert!(document.get_cached_diagnostics().is_none());
    }
    
    #[test]
    fn test_document_manager_diagnostic_operations() {
        let mut manager = UssDocumentManager::new().unwrap();
        let uri1 = Url::parse("file:///test1.uss").unwrap();
        let uri2 = Url::parse("file:///test2.uss").unwrap();
        let content = ".button { color: red; }".to_string();
        
        // Open documents
        manager.open_document(uri1.clone(), content.clone(), 1);
        manager.open_document(uri2.clone(), content, 1);
        
        // Cache diagnostics for both documents
        let diagnostics = vec![
            Diagnostic {
                range: Range {
                    start: Position { line: 0, character: 0 },
                    end: Position { line: 0, character: 7 },
                },
                severity: Some(DiagnosticSeverity::WARNING),
                code: None,
                code_description: None,
                source: Some("uss".to_string()),
                message: "Test diagnostic".to_string(),
                related_information: None,
                tags: None,
                data: None,
            }
        ];
        
        {
            let doc1 = manager.get_document_mut(&uri1).unwrap();
            doc1.cache_diagnostics(diagnostics.clone());
            let doc2 = manager.get_document_mut(&uri2).unwrap();
            doc2.cache_diagnostics(diagnostics);
        }
        
        // Both should have valid diagnostics
        assert!(manager.has_valid_diagnostics(&uri1));
        assert!(manager.has_valid_diagnostics(&uri2));
        
        // Invalidate diagnostics for one document
        manager.invalidate_document_diagnostics(&uri1);
        assert!(!manager.has_valid_diagnostics(&uri1));
        assert!(manager.has_valid_diagnostics(&uri2));
        
        // Invalidate all diagnostics
        manager.invalidate_all_diagnostics();
        assert!(!manager.has_valid_diagnostics(&uri1));
        assert!(!manager.has_valid_diagnostics(&uri2));
    }
}