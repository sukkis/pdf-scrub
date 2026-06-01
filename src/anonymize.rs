pub fn anonymize(text: &str, owner_name: &str) -> String {
    let owner_lower = owner_name.to_lowercase();
    let owner_tokens: Vec<String> = owner_name
        .split_whitespace()
        .map(|t| t.to_lowercase())
        .collect();

    let mut result = String::new();
    let mut unknown_names: Vec<String> = Vec::new();
    let mut remaining = text;

    while let Some(tag_start) = remaining.find("[NAME:") {
        result.push_str(&remaining[..tag_start]);
        remaining = &remaining[tag_start..];

        match remaining.find(']') {
            Some(tag_end) => {
                let inner = remaining[6..tag_end].trim();
                let replacement =
                    replacement_for(inner, &owner_lower, &owner_tokens, &mut unknown_names);
                result.push_str(&replacement);
                remaining = &remaining[tag_end + 1..];
            }
            None => {
                result.push_str(remaining);
                remaining = "";
            }
        }
    }

    result.push_str(remaining);
    result
}

fn replacement_for(
    name: &str,
    owner_lower: &str,
    owner_tokens: &[String],
    unknown_names: &mut Vec<String>,
) -> String {
    let name_lower = name.to_lowercase();

    if name_lower == owner_lower {
        return "Omistaja".to_string();
    }

    let name_tokens: Vec<String> = name.split_whitespace().map(|t| t.to_lowercase()).collect();

    if owner_tokens.iter().any(|t| name_tokens.contains(t)) {
        return "Omistaja?".to_string();
    }

    if let Some(pos) = unknown_names
        .iter()
        .position(|n| n.to_lowercase() == name_lower)
    {
        format!("henkilö {}", pos + 1)
    } else {
        unknown_names.push(name.to_string());
        format!("henkilö {}", unknown_names.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_tags_returns_text_unchanged() {
        assert_eq!(
            anonymize("Hello world.", "Matti Meikäläinen"),
            "Hello world."
        );
    }

    #[test]
    fn exact_owner_name_replaced_with_omistaja() {
        assert_eq!(
            anonymize("[NAME: Matti Meikäläinen] signed.", "Matti Meikäläinen"),
            "Omistaja signed."
        );
    }

    #[test]
    fn owner_name_match_is_case_insensitive() {
        assert_eq!(
            anonymize("[NAME: matti meikäläinen] signed.", "Matti Meikäläinen"),
            "Omistaja signed."
        );
    }

    #[test]
    fn owner_first_name_alone_replaced_with_omistaja_question() {
        assert_eq!(
            anonymize("Signed by [NAME: Matti].", "Matti Meikäläinen"),
            "Signed by Omistaja?."
        );
    }

    #[test]
    fn owner_last_name_alone_replaced_with_omistaja_question() {
        assert_eq!(
            anonymize("Signed by [NAME: Meikäläinen].", "Matti Meikäläinen"),
            "Signed by Omistaja?."
        );
    }

    #[test]
    fn owner_token_match_is_case_insensitive() {
        assert_eq!(
            anonymize("Signed by [NAME: MATTI].", "Matti Meikäläinen"),
            "Signed by Omistaja?."
        );
    }

    #[test]
    fn unknown_name_replaced_with_henkilo_1() {
        assert_eq!(
            anonymize("Contact [NAME: John Doe].", "Matti Meikäläinen"),
            "Contact henkilö 1."
        );
    }

    #[test]
    fn two_unknown_names_get_sequential_numbers() {
        assert_eq!(
            anonymize(
                "[NAME: John Doe] and [NAME: Jane Smith].",
                "Matti Meikäläinen"
            ),
            "henkilö 1 and henkilö 2."
        );
    }

    #[test]
    fn same_unknown_name_gets_same_number_throughout() {
        assert_eq!(
            anonymize(
                "[NAME: John Doe] met [NAME: John Doe] again.",
                "Matti Meikäläinen"
            ),
            "henkilö 1 met henkilö 1 again."
        );
    }

    #[test]
    fn mixed_owner_and_unknown_names() {
        assert_eq!(
            anonymize(
                "[NAME: Matti Meikäläinen] called [NAME: John Doe].",
                "Matti Meikäläinen"
            ),
            "Omistaja called henkilö 1."
        );
    }

    #[test]
    fn unknown_name_numbering_is_stable_across_multiple_occurrences() {
        assert_eq!(
            anonymize(
                "[NAME: John Doe], [NAME: Jane Smith], [NAME: John Doe].",
                "Matti Meikäläinen"
            ),
            "henkilö 1, henkilö 2, henkilö 1."
        );
    }

    #[test]
    fn surrounding_text_is_preserved_exactly() {
        assert_eq!(
            anonymize(
                "Dear [NAME: Matti Meikäläinen],\nPlease sign.\nRegards",
                "Matti Meikäläinen"
            ),
            "Dear Omistaja,\nPlease sign.\nRegards"
        );
    }
}
