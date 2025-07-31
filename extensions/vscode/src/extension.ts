import * as vscode from 'vscode';
import * as path from 'path';
import * as net from 'net';
import { LanguageClient, LanguageClientOptions, ServerOptions, TransportKind } from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: vscode.ExtensionContext) {
  console.log('Auto-Coder Companion extension is now active');

  // Register commands
  const openAutoCoderCommand = vscode.commands.registerCommand('auto-coder-companion.openAutoCoder', () => {
    vscode.window.showInformationMessage('Opening Auto-Coder Companion...');
    launchAutoCoderApp(context);
  });

  const analyzeCodeCommand = vscode.commands.registerCommand('auto-coder-companion.analyzeCode', () => {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
      vscode.window.showErrorMessage('No active editor found');
      return;
    }

    const selection = editor.selection;
    const text = editor.document.getText(selection.isEmpty ? undefined : selection);
    const language = editor.document.languageId;

    // Send to Auto-Coder for analysis
    analyzeCodeWithAutoCoder(text, language);
  });

  const analyzeRepositoryCommand = vscode.commands.registerCommand('auto-coder-companion.analyzeRepository', async () => {
    const workspaceFolders = vscode.workspace.workspaceFolders;
    if (!workspaceFolders || workspaceFolders.length === 0) {
      vscode.window.showErrorMessage('No workspace folder found');
      return;
    }

    // If multiple workspace folders, ask user to select one
    let folderPath: string;
    if (workspaceFolders.length === 1) {
      folderPath = workspaceFolders[0].uri.fsPath;
    } else {
      const selected = await vscode.window.showQuickPick(
        workspaceFolders.map(folder => ({
          label: folder.name,
          description: folder.uri.fsPath,
          folderPath: folder.uri.fsPath
        })),
        { placeHolder: 'Select workspace folder to analyze' }
      );
      
      if (!selected) {
        return;
      }
      
      folderPath = selected.folderPath;
    }

    // Send to Auto-Coder for repository analysis
    analyzeRepositoryWithAutoCoder(folderPath);
  });

  // Start LSP client
  startLspClient(context);

  context.subscriptions.push(
    openAutoCoderCommand,
    analyzeCodeCommand,
    analyzeRepositoryCommand
  );
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}

function startLspClient(context: vscode.ExtensionContext) {
  // The server is implemented in Rust
  const serverOptions: ServerOptions = {
    run: {
      command: 'auto-coder-lsp',
      transport: TransportKind.stdio
    },
    debug: {
      command: 'auto-coder-lsp',
      args: ['--debug'],
      transport: TransportKind.stdio
    }
  };

  // Options to control the language client
  const clientOptions: LanguageClientOptions = {
    // Register the server for all relevant file types
    documentSelector: [
      { scheme: 'file', language: 'javascript' },
      { scheme: 'file', language: 'typescript' },
      { scheme: 'file', language: 'python' },
      { scheme: 'file', language: 'rust' },
      { scheme: 'file', language: 'go' },
      { scheme: 'file', language: 'java' },
      { scheme: 'file', language: 'c' },
      { scheme: 'file', language: 'cpp' },
      { scheme: 'file', language: 'csharp' }
    ],
    synchronize: {
      // Notify the server about file changes to project files
      fileEvents: vscode.workspace.createFileSystemWatcher('**/*')
    }
  };

  // Create the language client and start it
  client = new LanguageClient(
    'auto-coder-lsp',
    'Auto-Coder LSP',
    serverOptions,
    clientOptions
  );

  // Start the client
  client.start();
}

function launchAutoCoderApp(context: vscode.ExtensionContext) {
  // This would launch the Tauri app or connect to an existing instance
  // In a real implementation, this would use IPC or a local server to communicate
  vscode.window.showInformationMessage('Auto-Coder Companion app launched');
}

function analyzeCodeWithAutoCoder(code: string, language: string) {
  // This would send the code to the Auto-Coder app for analysis
  // In a real implementation, this would use IPC or a local server to communicate
  vscode.window.showInformationMessage(`Analyzing ${language} code with Auto-Coder Companion`);
}

function analyzeRepositoryWithAutoCoder(repositoryPath: string) {
  // This would send the repository path to the Auto-Coder app for analysis
  // In a real implementation, this would use IPC or a local server to communicate
  vscode.window.showInformationMessage(`Analyzing repository at ${repositoryPath} with Auto-Coder Companion`);
}
