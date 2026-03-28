use std::{path::Path, process::Command};

use crate::editorconfig::TextEditor;

// There is no standard way to do this, so we try different editors
// Ideally the user should be able his preferred editor
// Roughly sorted by popularity (least to most popular)
pub fn open_file_at_line(file: &Path, line: usize, prefered_text_editor: Option<TextEditor>) {
    let absolute_path = file
        .canonicalize()
        .expect("Failed to canonicalize path. This should not happen because the file exists")
        .display()
        .to_string();

    let opened_successfully = match prefered_text_editor {
        None => false,
        Some(TextEditor::Antigravity) => {
            let is_antigravity = which::which("antigravity").is_ok();
            if is_antigravity {
                let res = Command::new("antigravity")
                    .args(["--goto", format!("{}:{}", absolute_path, line).as_str()])
                    .spawn();
                res.is_ok()
            } else {
                false
            }
        }
        Some(TextEditor::SublimeText) => {
            let is_sublime = which::which("subl").is_ok();
            if is_sublime {
                let res = Command::new("subl")
                    .args([format!("{}:{}", absolute_path, line).as_str()])
                    .spawn();
                res.is_ok()
            } else {
                false
            }
        }
        Some(TextEditor::Zed) => {
            let is_zed = which::which("zed").is_ok();
            if is_zed {
                let res = Command::new("zed")
                    .args([format!("{}:{}", absolute_path, line).as_str()])
                    .spawn();
                res.is_ok()
            } else {
                false
            }
        }
        Some(TextEditor::VSCode) => {
            let is_code = which::which("code").is_ok();
            if is_code {
                // code --goto "path/to/file:line"
                let res = Command::new("code")
                    .args(["--goto", format!("{}:{}", absolute_path, line).as_str()])
                    .spawn();
                res.is_ok()
            } else {
                false
            }
        }
        Some(TextEditor::Cursor) => {
            let is_cursor = which::which("cursor").is_ok();
            if is_cursor {
                let res = Command::new("cursor")
                    .args(["--goto", format!("{}:{}", absolute_path, line).as_str()])
                    .spawn();
                res.is_ok()
            } else {
                false
            }
        }
        Some(TextEditor::Vim) => {
            let is_vim = which::which("vim").is_ok();
            if is_vim {
                open_in_terminal("vim", &[format!("+{}", line), absolute_path])
            } else {
                false
            }
        }
        Some(TextEditor::Neovim) => {
            let is_neovim = which::which("nvim").is_ok();
            if is_neovim {
                open_in_terminal("nvim", &[format!("+{}", line), absolute_path])
            } else {
                false
            }
        }
        Some(TextEditor::Emacs) => {
            let is_emacs = which::which("emacsclient").is_ok();
            if is_emacs {
                let res = Command::new("emacsclient")
                    .args([format!("+{}", line), absolute_path])
                    .spawn();
                res.is_ok()
            } else {
                false
            }
        }
    };

    if !opened_successfully {
        let _ = open::that(file);
    }
}

fn open_in_terminal(command: &str, args: &[String]) -> bool {
    #[cfg(target_os = "macos")]
    {
        // On macOS, we use AppleScript to tell Terminal to run the command

        // Escape for shell: wrap in single quotes, escape single quotes inside
        fn shell_escape(s: &str) -> String {
            format!("'{}'", s.replace("'", "'\\''"))
        }

        let mut shell_cmd_parts = Vec::with_capacity(args.len() + 1);
        shell_cmd_parts.push(shell_escape(command));
        for arg in args {
            shell_cmd_parts.push(shell_escape(arg));
        }
        let shell_cmd = shell_cmd_parts.join(" ");

        // Escape backslashes and double quotes for the AppleScript string
        let apple_script_cmd = shell_cmd.replace('\\', "\\\\").replace('"', "\\\"");

        let script = format!(
            "tell application \"Terminal\"
                do script \"{}\"
                activate
            end tell",
            apple_script_cmd
        );

        Command::new("osascript")
            .arg("-e")
            .arg(script)
            .spawn()
            .map(|_| true)
            .unwrap_or(false)
    }

    #[cfg(target_os = "windows")]
    {
        // On Windows, we can use `start` command in cmd
        // start cmd /c "command args"
        // We join args because `start` behaves weirdly with multiple quoted args if the first arg is quoted (it treats it as title)
        // But here we are passing arguments to the command being started.

        let mut cmd_args = Vec::new();
        cmd_args.push("/C".to_string());
        cmd_args.push("start".to_string());
        cmd_args.push(command.to_string());
        cmd_args.extend_from_slice(args);

        Command::new("cmd")
            .args(&cmd_args)
            .spawn()
            .map(|_| true)
            .unwrap_or(false)
    }

    #[cfg(target_os = "linux")]
    {
        // On Linux, as this is a bad OS, there is no standard way to open a terminal
        // We try a list of common terminal emulators
        // We prioritize $TERMINAL if set
        let mut terminals: Vec<std::borrow::Cow<'static, str>> = Vec::new();

        if let Ok(term) = std::env::var("TERMINAL") {
            terminals.push(term.into());
        }

        let candidates = [
            "x-terminal-emulator",
            "gnome-terminal",
            "konsole",
            "xfce4-terminal",
            "xterm",
            "kitty",
            "alacritty",
        ];

        for c in candidates {
            terminals.push(c.into());
        }

        for terminal in terminals {
            if which::which(&*terminal).is_ok() {
                // Determine the flag to execute a command
                let arg_flag = if terminal.contains("gnome-terminal") {
                    Some("--")
                } else if terminal.contains("xfce4-terminal") || terminal.contains("terminator") {
                    Some("-x")
                } else if terminal.contains("kitty") {
                    None // kitty uses NO flag, just `kitty command`
                } else {
                    Some("-e") // Default standard (xterm, konsole, alacritty, x-terminal-emulator)
                };

                let mut cmd_args = Vec::new();
                if let Some(flag) = arg_flag {
                    cmd_args.push(flag.to_string());
                }
                cmd_args.push(command.to_string());
                cmd_args.extend_from_slice(args);

                if Command::new(&*terminal).args(&cmd_args).spawn().is_ok() {
                    return true;
                }
            }
        }
        false
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        false
    }
}
