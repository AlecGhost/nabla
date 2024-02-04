use nabla_frontend::{self, token::TextRange};
use tower_lsp::{
    jsonrpc::Result,
    lsp_types::{
        Diagnostic, DiagnosticSeverity, DidChangeTextDocumentParams, DidOpenTextDocumentParams,
        InitializeParams, InitializeResult, MessageType, Position, Range, ServerCapabilities,
        ServerInfo, TextDocumentSyncCapability, TextDocumentSyncKind, TextDocumentSyncOptions, Url,
    },
    Client, LanguageServer, LspService, Server,
};

#[derive(Debug)]
struct NablaLS {
    client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for NablaLS {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "Nabla LS".to_string(),
                version: Some("0.1".to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::FULL),
                        ..Default::default()
                    },
                )),
                ..Default::default()
            },
        })
    }
    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file opened!")
            .await;
        self.on_change(params.text_document.uri, params.text_document.text)
            .await
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file changed!")
            .await;
        let mut changes = params.content_changes;
        self.on_change(
            params.text_document.uri,
            std::mem::take(&mut changes[0].text),
        )
        .await
    }
    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

impl NablaLS {
    async fn on_change(&self, uri: Url, text: String) {
        let mut diagnostics = Vec::new();
        let (tokens, errors) = nabla_frontend::lexer::lex(&text);
        for error in errors {
            let range = convert_text_range(&text, &error.range);
            let diagnostic = new_diagnostic(range, error.message.to_string());
            diagnostics.push(diagnostic);
        }
        let (program, errors) = nabla_frontend::parser::parse(&tokens);
        for error in errors {
            let text_range =
                tokens[error.range.start].range.start..tokens[error.range.end].range.end;
            let range = convert_text_range(&text, &text_range);
            let diagnostic = new_diagnostic(range, error.message.to_string());
            diagnostics.push(diagnostic);
        }
        let (_, _, errors) = nabla_frontend::semantics::analyze(&program);
        for error in errors {
            let text_range =
                tokens[error.range.start].range.start..tokens[error.range.end].range.end;
            let range = convert_text_range(&text, &text_range);
            let diagnostic = new_diagnostic(range, error.message.to_string());
            diagnostics.push(diagnostic);
        }
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| NablaLS { client });
    Server::new(stdin, stdout, socket).serve(service).await;
}

fn convert_text_range(text: &str, range: &TextRange) -> Range {
    let before_range = &text[..range.start];
    let start = before_range
        .split('\n')
        .enumerate()
        .last()
        .map(|(line_number, last_line)| Position {
            line: line_number as u32,
            character: last_line.len() as u32,
        })
        .expect("Split must yield at least one element");
    let in_range = &text[range.clone()];
    let end = in_range
        .split('\n')
        .enumerate()
        .last()
        .map(|(line_number, last_line)| Position {
            line: line_number as u32 + start.line,
            character: if line_number == 0 {
                // end is on the same line as start, therefore the char positions must be added
                start.character + last_line.len() as u32
            } else {
                last_line.len() as u32
            },
        })
        .expect("Split must yield at least one element");
    Range { start, end }
}

fn new_diagnostic(range: Range, message: String) -> Diagnostic {
    Diagnostic {
        range,
        message,
        severity: Some(DiagnosticSeverity::ERROR),
        ..Default::default()
    }
}
