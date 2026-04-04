pub(super) fn push_line(out: &mut String, line: &str) {
    if line.is_empty() {
        return;
    }
    if !out.is_empty() {
        out.push('\n');
    }
    out.push_str(line);
}

pub(super) fn truncate(s: &str, max_len: usize) -> String {
    if max_len == 0 {
        return String::new();
    }

    let count = s.chars().count();
    if count <= max_len {
        return s.to_string();
    }

    if max_len <= 3 {
        return ".".repeat(max_len);
    }

    let take_len = max_len - 3;
    let mut out = String::with_capacity(max_len);
    out.extend(s.chars().take(take_len));
    out.push_str("...");
    out
}

pub(super) fn queued_message_preview(text: &str, max_len: usize) -> String {
    let first_line = text
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("")
        .trim();
    if first_line.is_empty() {
        return "(empty)".to_string();
    }
    truncate(first_line, max_len)
}
