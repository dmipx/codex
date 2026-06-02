use std::time::Duration;

use codex_protocol::protocol::ExecCommandStatus;

use super::ContextualUserFragment;

const OUTPUT_PREVIEW_MAX_BYTES: usize = 4096;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct BackgroundTerminalNotification {
    pub(crate) call_id: String,
    pub(crate) process_id: Option<String>,
    pub(crate) command: Vec<String>,
    pub(crate) cwd: String,
    pub(crate) exit_code: i32,
    pub(crate) status: ExecCommandStatus,
    pub(crate) duration: Duration,
    pub(crate) stdout_bytes: usize,
    pub(crate) stderr_bytes: usize,
    pub(crate) output_preview: String,
    pub(crate) output_truncated: bool,
}

impl BackgroundTerminalNotification {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        call_id: String,
        process_id: Option<String>,
        command: Vec<String>,
        cwd: String,
        exit_code: i32,
        status: ExecCommandStatus,
        duration: Duration,
        stdout: &str,
        stderr: &str,
        formatted_output: String,
    ) -> Self {
        let (output_preview, output_truncated) =
            truncate_output_preview(&formatted_output, OUTPUT_PREVIEW_MAX_BYTES);
        Self {
            call_id,
            process_id,
            command,
            cwd,
            exit_code,
            status,
            duration,
            stdout_bytes: stdout.len(),
            stderr_bytes: stderr.len(),
            output_preview,
            output_truncated,
        }
    }
}

impl ContextualUserFragment for BackgroundTerminalNotification {
    fn role() -> &'static str {
        "user"
    }

    fn markers(&self) -> (&'static str, &'static str) {
        Self::type_markers()
    }

    fn type_markers() -> (&'static str, &'static str) {
        (
            "<background_terminal_notification>",
            "</background_terminal_notification>",
        )
    }

    fn body(&self) -> String {
        let duration_ms = u64::try_from(self.duration.as_millis()).unwrap_or(u64::MAX);
        format!(
            "\n{}\n",
            serde_json::json!({
                "call_id": &self.call_id,
                "process_id": &self.process_id,
                "command": &self.command,
                "cwd": &self.cwd,
                "exit_code": self.exit_code,
                "status": &self.status,
                "duration_ms": duration_ms,
                "stdout_bytes": self.stdout_bytes,
                "stderr_bytes": self.stderr_bytes,
                "output_truncated": self.output_truncated,
                "output_preview": &self.output_preview,
            })
        )
    }
}

fn truncate_output_preview(text: &str, max_bytes: usize) -> (String, bool) {
    if text.len() <= max_bytes {
        return (text.to_string(), false);
    }

    let mut end = max_bytes;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    (text[..end].to_string(), true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notification_output_preview_is_bounded() {
        let text = "a".repeat(OUTPUT_PREVIEW_MAX_BYTES + 1);

        let (preview, truncated) = truncate_output_preview(&text, OUTPUT_PREVIEW_MAX_BYTES);

        assert_eq!(OUTPUT_PREVIEW_MAX_BYTES, preview.len());
        assert!(truncated);
    }

    #[test]
    fn notification_output_preview_preserves_utf8_boundaries() {
        let text = "é".repeat(OUTPUT_PREVIEW_MAX_BYTES);

        let (preview, truncated) = truncate_output_preview(&text, OUTPUT_PREVIEW_MAX_BYTES - 1);

        assert!(preview.is_char_boundary(preview.len()));
        assert!(truncated);
    }
}
