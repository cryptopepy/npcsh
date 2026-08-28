use std::io::IsTerminal;
use std::time::{Duration, Instant};

/// Render a markdown string to an ANSI-styled string using the terminal width.
pub fn render_block(md: &str) -> String {
    if md.trim().is_empty() {
        return md.to_string();
    }
    let skin = termimad::MadSkin::default();
    format!("{}", skin.term_text(md))
}

/// Renderer that streams markdown deltas to stderr.
///
/// To avoid the duplication/offset bugs that come from in-place overwriting of
/// wrapped markdown, this renderer simply emits each genuinely new text suffix
/// once.  Markdown styling is applied to small chunks as they arrive, accepting
/// that complex block formatting may only finalize once a newline or final flush
/// occurs.
pub struct StreamRenderer {
    /// Raw accumulated source text (matches the server-side content string).
    buffer: String,
    /// Number of bytes of `buffer` that have already been emitted.
    emitted_len: usize,
    skin: termimad::MadSkin,
    last_render: Instant,
    min_interval: Duration,
    disabled: bool,
}

impl StreamRenderer {
    pub fn new() -> Self {
        let disabled = !std::io::stderr().is_terminal();
        Self {
            buffer: String::new(),
            emitted_len: 0,
            skin: termimad::MadSkin::default(),
            last_render: Instant::now(),
            min_interval: Duration::from_millis(50),
            disabled,
        }
    }

    /// Append a raw markdown delta and emit only the genuinely new suffix.
    pub fn push(&mut self, text: &str) {
        if self.disabled {
            eprint!("{}", text);
            self.buffer.push_str(text);
            let _ = std::io::Write::flush(&mut std::io::stderr());
            return;
        }

        self.buffer.push_str(text);

        let ends_with_newline = self.buffer.ends_with('\n') || self.buffer.ends_with("\r\n");
        let now = Instant::now();
        if now.duration_since(self.last_render) >= self.min_interval || ends_with_newline {
            self.emit_new(ends_with_newline);
            if ends_with_newline {
                self.buffer.clear();
                self.emitted_len = 0;
            }
        }
    }

    /// Force a final flush of any remaining unemitted text.
    pub fn flush(&mut self) {
        if self.disabled {
            return;
        }
        self.emit_new(true);
        self.emitted_len = self.buffer.len();
    }

    /// Clear the accumulated buffer and forget emitted progress.
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.emitted_len = 0;
    }

    fn emit_new(&mut self, ends_with_newline: bool) {
        if self.buffer.len() <= self.emitted_len {
            return;
        }

        let new_raw = &self.buffer[self.emitted_len..];
        let trim_raw = new_raw.trim_end_matches(['\n', '\r']);
        if trim_raw.is_empty() {
            self.emitted_len = self.buffer.len();
            return;
        }

        // Render the new suffix by itself.  For small inline deltas this gives the
        // right result.  When a newline arrives, render the full accumulated line
        // that just completed so markdown on that line resolves properly.
        let to_render = if ends_with_newline {
            // Find the start of the line that just ended.
            let line_start = self.buffer[..self.emitted_len]
                .rfind('\n')
                .map(|i| i + 1)
                .unwrap_or(0);
            let full_line = &self.buffer[line_start..].trim_end_matches(['\n', '\r']);
            // Only re-render if the completed line hasn't been emitted as a whole yet.
            if line_start >= self.emitted_len {
                full_line.to_string()
            } else {
                trim_raw.to_string()
            }
        } else {
            trim_raw.to_string()
        };

        let rendered = format!("{}", self.skin.term_text(&to_render));
        let rendered = rendered.trim_end_matches(['\n', '\r']).to_string();
        if !rendered.is_empty() {
            eprint!("{}", rendered);
        }

        self.emitted_len = self.buffer.len();
        self.last_render = Instant::now();
    }
}

impl Default for StreamRenderer {
    fn default() -> Self {
        Self::new()
    }
}
