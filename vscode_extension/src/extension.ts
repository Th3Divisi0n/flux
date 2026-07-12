import * as vscode from 'vscode';
import { execFile } from 'child_process';
import { promisify } from 'util';

const execFileAsync = promisify(execFile);

export function activate(context: vscode.ExtensionContext) {
    const runFile = vscode.commands.registerCommand('flux.runFile', async () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor || editor.document.languageId !== 'flux') {
            vscode.window.showWarningMessage('Open a .fx file to run.');
            return;
        }

        const config = vscode.workspace.getConfiguration('flux');
        const executable = config.get<string>('executablePath', 'fx');
        const filePath = editor.document.uri.fsPath;

        const terminal = vscode.window.createTerminal('FLUX');
        terminal.show();
        terminal.sendText(`${executable} run "${filePath}"`);
    });

    const buildFile = vscode.commands.registerCommand('flux.buildFile', async () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor || editor.document.languageId !== 'flux') {
            vscode.window.showWarningMessage('Open a .fx file to build.');
            return;
        }

        const config = vscode.workspace.getConfiguration('flux');
        const executable = config.get<string>('executablePath', 'fx');
        const filePath = editor.document.uri.fsPath;

        const terminal = vscode.window.createTerminal('FLUX');
        terminal.show();
        terminal.sendText(`${executable} build "${filePath}"`);
    });

    context.subscriptions.push(runFile, buildFile);
}

export function deactivate() {}
