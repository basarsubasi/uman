use std::io::Write;
use std::process::{Command, Stdio};

use crate::error::UnimanError;

/// Returns Err(FzfNotFound) if fzf is not on PATH.
pub fn require_fzf() -> anyhow::Result<()> {
    if Command::new("fzf")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        Ok(())
    } else {
        Err(UnimanError::FzfNotFound.into())
    }
}

/// Runs fzf in browse mode: fzf stays open after each selection.
///
/// When the user presses Enter, fzf suspends and runs `execute_template`
/// as a shell command (with fzf field references like `{1}`, `{2}`
/// substituted). When that command exits, fzf resumes in exactly the
/// same state — same query, cursor position and scroll.
///
/// The user exits by pressing Escape or Ctrl-C.
pub fn browse(header: &str, execute_template: &str, with_nth: Option<&str>, lines: &[String]) -> anyhow::Result<()> {
    let bind_arg = format!("enter:execute({})", execute_template);

    let mut cmd = Command::new("fzf");
    cmd.args([
        "--header",
        header,
        "--delimiter",
        "\t",
        "--layout",
        "reverse",
        "--bind",
        &bind_arg,
    ]);

    if let Some(nth) = with_nth {
        cmd.arg("--with-nth");
        cmd.arg(nth);
    }

    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::null()) // stdout unused in browse mode
        .stderr(Stdio::inherit())
        .spawn()?;

    {
        let stdin = child.stdin.as_mut().expect("fzf stdin should be piped");
        for line in lines {
            writeln!(stdin, "{}", line)?;
        }
    }

    match child.wait()?.code() {
        Some(0) | Some(1) | Some(130) => Ok(()),
        Some(code) => anyhow::bail!("fzf exited with unexpected code {}", code),
        None => anyhow::bail!("fzf was terminated by a signal"),
    }
}
