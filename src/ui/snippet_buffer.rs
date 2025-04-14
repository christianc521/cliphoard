use crate::system::ClipboardStorage;
pub struct SnippetBuffer {
    pub snippet_indices: Vec<usize>,
}

impl SnippetBuffer {
    pub fn new() -> SnippetBuffer {
        SnippetBuffer {
            snippet_indices: Vec::new(),
        }
    }

    pub fn update(
        &mut self,
        snippet: &ClipboardStorage,
        input_buffer: &String,
    ) -> Result<bool, ()> {
        let mut matches: Vec<(usize, bool)> = Vec::new();
        for (i, snippet) in snippet.get_entries().iter().enumerate() {
            let nickname_match = snippet
                .nickname
                .as_ref()
                .map_or(false, |n| self.fuzzy_find(n, input_buffer));

            let content_match = self.fuzzy_find(&snippet.content, input_buffer);

            if nickname_match || content_match {
                matches.push((i, nickname_match));
            }
        }

        // Prioritize nickname search
        matches.sort_by(|a, b| b.1.cmp(&a.1));

        self.snippet_indices = matches.into_iter().map(|(i, _)| i).collect();
        Ok(true)
    }

    fn fuzzy_find(&self, content: &str, input_buffer: &String) -> bool {
        let query = input_buffer.to_lowercase();
        let content_lower = content.to_lowercase();
        let mut content_chars = content_lower.chars().peekable();

        for qc in query.chars() {
            loop {
                match content_chars.next() {
                    Some(sc) if sc == qc => break,
                    Some(_) => continue,
                    None => return false,
                }
            }
        }
        true
    }
}
