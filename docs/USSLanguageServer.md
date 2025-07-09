# USS Language Server Implementation

## Overview
The USS (Unity Style Sheet) language server provides IDE features for Unity's UI Toolkit styling language. USS is very similar to CSS, allowing us to leverage existing CSS tooling with Unity-specific extensions.

## Architecture

### Parser
- **tree-sitter-css**: We use the existing CSS grammar from the `tree-sitter-css` crate as USS syntax is nearly identical to CSS
- **No custom grammar needed**: USS follows CSS syntax rules with Unity-specific properties and values
- **Incremental parsing**: tree-sitter provides efficient incremental parsing for real-time updates

### Language Server Framework
- **tower-lsp**: Provides the LSP (Language Server Protocol) implementation framework
- **Single-threaded async**: Follows project guidelines using tokio without multi-threading
- **Standard LSP features**: Implements textDocument/didOpen, didChange, didSave notifications

## Features

### 1. Syntax Highlighting (Phase 1)
- Parse USS files using tree-sitter-css
- Provide semantic tokens for:
  - Selectors (type, class, name, pseudo-classes)
  - Properties (standard CSS + Unity-specific like `-unity-font`)
  - Values (colors, lengths, keywords, asset references)
  - Comments and at-rules

### 2. Diagnostics (Phase 2)
- Validate USS syntax using parse tree
- Check for:
  - Unknown properties
  - Invalid values for known properties
  - Malformed selectors
  - Asset reference validation (url/resource functions)

### 3. Autocompletion (Phase 3)
- Property name completion
- Value completion based on property type
- Selector completion
- Asset path completion for url() and resource() functions

## Implementation Notes
- USS files use `.uss` extension
- Support Unity-specific properties (prefixed with `-unity-`)
- Handle asset references: `url()` and `resource()` functions
- Validate against USS property specification from USSLanguageSpec.md