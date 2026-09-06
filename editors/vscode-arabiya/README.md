# Arabiya Language for VS Code

This extension makes `.ar` files first-class Arabiya source files. It provides:

- `.ar` file recognition and Arabic syntax highlighting.
- Arabic snippets for `إن`, `عرف`, `لكل`, and `جرب`.
- Python-style indentation, comment toggling, and bracket pairing.
- An RTL custom editor with right-aligned source text and RTL input.
- Arabic runtime names such as `اطبع`, `طول`, and `نطاق` highlighted as builtins.

## Use

Open this folder in VS Code and press `F5` to launch an Extension Development Host. Open an `.ar` file. It opens in the **Arabiya RTL Editor** by default.

Use **Arabiya: Open RTL Editor** from the Command Palette to reopen any `.ar` file in the RTL editor. Choose **Reopen Editor With → Text Editor** if you need VS Code's standard text editor.

The custom editor deliberately owns RTL layout: VS Code's public extension API does not expose a way to force right alignment or writing direction in the regular Monaco text editor. The fallback text editor retains language highlighting but uses the user's normal editor layout.

## Package

Install `@vscode/vsce`, then package from this directory:

```bash
npm install --global @vscode/vsce
vsce package
```
