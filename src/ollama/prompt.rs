pub fn build_prompt(
    source_context: &str,
    target_context: &str,
    filename: &str,
    content: &str,
) -> String {
    format!(
        r#"You are an expert software architect.

I will give you a document from Project A. Your task is to rewrite it for Project B,
preserving all structural patterns and section headings, but adapting all
project-specific content (frameworks, libraries, patterns, terminology) to the new stack.

Project A (source): {source_context}

Project B (target): {target_context}

Document filename: {filename}
Document content:
{content}

Output only the rewritten document. Do not explain your changes."#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_contains_source_context() {
        let prompt = build_prompt("Django API", "Rust Axum", "README.md", "# Hello");
        assert!(prompt.contains("Django API"));
    }

    #[test]
    fn prompt_contains_target_context() {
        let prompt = build_prompt("Django API", "Rust Axum", "README.md", "# Hello");
        assert!(prompt.contains("Rust Axum"));
    }

    #[test]
    fn prompt_contains_filename() {
        let prompt = build_prompt("src", "tgt", "ARCH.md", "content");
        assert!(prompt.contains("ARCH.md"));
    }

    #[test]
    fn prompt_contains_content() {
        let prompt = build_prompt("src", "tgt", "f.md", "important content here");
        assert!(prompt.contains("important content here"));
    }

    #[test]
    fn prompt_has_instruction() {
        let prompt = build_prompt("src", "tgt", "f.md", "c");
        assert!(prompt.contains("Output only the rewritten document"));
    }
}
