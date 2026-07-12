"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const vscode = require("vscode");
const child_process_1 = require("child_process");
const util_1 = require("util");
const execFileAsync = (0, util_1.promisify)(child_process_1.execFile);
function activate(context) {
    const runFile = vscode.commands.registerCommand('flux.runFile', async () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor || editor.document.languageId !== 'flux') {
            vscode.window.showWarningMessage('Open a .fx file to run.');
            return;
        }
        const config = vscode.workspace.getConfiguration('flux');
        const executable = config.get('executablePath', 'fx');
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
        const executable = config.get('executablePath', 'fx');
        const filePath = editor.document.uri.fsPath;
        const terminal = vscode.window.createTerminal('FLUX');
        terminal.show();
        terminal.sendText(`${executable} build "${filePath}"`);
    });
    context.subscriptions.push(runFile, buildFile);
}
function deactivate() { }
//# sourceMappingURL=extension.js.map