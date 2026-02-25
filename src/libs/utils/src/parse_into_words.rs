pub fn parse_into_words(raw_text: String) -> Vec<String> {
    if raw_text.is_empty() {
        return vec![];
    }

    let cleaned_chars: String = raw_text
        .chars()
        .map(|c| {
            if c.is_alphabetic() || c.is_whitespace() {
                c
            } else {
                ' '
            }
        })
        .collect();

    let stop_words = [
        "and", "or", "is", "the", "a", "an", "of", "to", "in", "for", "with", "on", "at", "by",
    ];

    cleaned_chars
        .split_whitespace()
        .filter(|word| !stop_words.contains(word) || word.len() == 1)
        .map(|e| e.to_owned())
        .collect::<Vec<_>>()
}
