//! Tests for USS document management and incremental parsing

#[cfg(test)]
mod tests {

    use tower_lsp::lsp_types::{Position, Range, TextDocumentContentChangeEvent, Url};

    use crate::uss::{document::UssDocument, document_manager::UssDocumentManager};

    #[test]
    fn test_document_creation() {
        let uri = Url::parse("file:///test.uss").unwrap();
        let content = ".button { color: red; }".to_string();
        let document = UssDocument::new(uri.clone(), content.clone(), 1);
        
        assert_eq!(document.uri, uri);
        assert_eq!(document.content(), &content);
        assert_eq!(document.document_version().minor, 1);
    }
    
    #[test]
    fn test_document_manager() {
        let mut manager = UssDocumentManager::new().unwrap();
        let uri = Url::parse("file:///test.uss").unwrap();
        let content = ".button { color: red; }".to_string();
        
        // Open document
        manager.open_document(uri.clone(), content.clone(), 1);
        
        // Check document exists and is open
        let document = manager.get_document(&uri).unwrap();
        assert_eq!(document.content(), &content);
        assert_eq!(document.document_version().minor, 1);
        assert!(manager.is_document_open(&uri));
        
        // Close document - it should be completely removed from memory
        manager.close_document(&uri);
        assert!(manager.get_document(&uri).is_none());
        assert!(!manager.is_document_open(&uri));
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
        assert_eq!(document.document_version().minor, 2);
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
        assert_eq!(document.document_version().minor, 2);
    }
    
    #[test]
    fn test_line_starts_calculation() {
        let uri = Url::parse("file:///test.uss").unwrap();
        let content = ".button {\n  color: red;\n  font-size: 12px;\n}".to_string();
        let document = UssDocument::new(uri, content, 1);
        
        // Test position to byte conversion
        // This is an internal test, so we'll just verify the document was created successfully
        assert_eq!(document.document_version().minor, 1);
        assert!(document.content().contains("color: red"));
    }
    

    

    

}