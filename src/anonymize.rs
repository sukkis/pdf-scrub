pub fn anonymize(text: &str, owner_firstname: &str, owner_lastname: &str) -> String {
    let full_fn_ln = format!("{owner_firstname} {owner_lastname}");
    let full_ln_fn = format!("{owner_lastname} {owner_firstname}");
    let text = replace_ci(text, &full_fn_ln, "Omistaja");
    let text = replace_ci(&text, &full_ln_fn, "Omistaja");
    let text = replace_ci(&text, owner_firstname, "Omistaja?");
    replace_ci(&text, owner_lastname, "Omistaja?")
}

// Case-insensitive substring replacement. Assumes that for all characters in
// `pattern`, uppercase and lowercase forms have identical UTF-8 byte lengths
// (true for ASCII and Finnish letters ä, ö, å).
fn replace_ci(text: &str, pattern: &str, replacement: &str) -> String {
    if pattern.is_empty() {
        return text.to_string();
    }
    let pattern_lower = pattern.to_lowercase();
    let text_lower = text.to_lowercase();
    let pat_len = pattern_lower.len();
    let mut result = String::new();
    let mut pos = 0;

    while pos + pat_len <= text_lower.len() {
        if text_lower[pos..].starts_with(&pattern_lower) {
            result.push_str(replacement);
            pos += pat_len;
        } else {
            let c = text[pos..].chars().next().unwrap();
            result.push(c);
            pos += c.len_utf8();
        }
    }
    result.push_str(&text[pos..]);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_owner_name_in_text_returns_unchanged() {
        assert_eq!(
            anonymize("No names here.", "Matti", "Meikäläinen"),
            "No names here."
        );
    }

    #[test]
    fn firstname_lastname_replaced_with_omistaja() {
        assert_eq!(
            anonymize("Matti Meikäläinen signed.", "Matti", "Meikäläinen"),
            "Omistaja signed."
        );
    }

    #[test]
    fn lastname_firstname_replaced_with_omistaja() {
        assert_eq!(
            anonymize("Meikäläinen Matti signed.", "Matti", "Meikäläinen"),
            "Omistaja signed."
        );
    }

    #[test]
    fn full_name_match_is_case_insensitive() {
        assert_eq!(
            anonymize("MATTI MEIKÄLÄINEN signed.", "Matti", "Meikäläinen"),
            "Omistaja signed."
        );
    }

    #[test]
    fn firstname_alone_replaced_with_omistaja_question() {
        assert_eq!(
            anonymize("Signed by Matti.", "Matti", "Meikäläinen"),
            "Signed by Omistaja?."
        );
    }

    #[test]
    fn lastname_alone_replaced_with_omistaja_question() {
        assert_eq!(
            anonymize("Signed by Meikäläinen.", "Matti", "Meikäläinen"),
            "Signed by Omistaja?."
        );
    }

    #[test]
    fn partial_match_is_case_insensitive() {
        assert_eq!(
            anonymize("Signed by MATTI.", "Matti", "Meikäläinen"),
            "Signed by Omistaja?."
        );
    }

    #[test]
    fn multiple_occurrences_all_replaced() {
        assert_eq!(
            anonymize("Matti met Matti again.", "Matti", "Meikäläinen"),
            "Omistaja? met Omistaja? again."
        );
    }

    #[test]
    fn full_name_replaced_before_partials() {
        assert_eq!(
            anonymize(
                "Matti Meikäläinen said hello to Matti.",
                "Matti",
                "Meikäläinen"
            ),
            "Omistaja said hello to Omistaja?."
        );
    }

    #[test]
    fn newlines_preserved() {
        assert_eq!(
            anonymize(
                "Dear Matti Meikäläinen,\nPlease sign.\nRegards",
                "Matti",
                "Meikäläinen"
            ),
            "Dear Omistaja,\nPlease sign.\nRegards"
        );
    }
}
