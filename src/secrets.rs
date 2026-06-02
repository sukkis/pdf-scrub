use zeroize::Zeroizing;

const DEFAULT_MODEL: &str = "mistral-small3.2:24b";

pub struct Secrets {
    pub source_dir: Zeroizing<String>,
    pub dest_dir: Zeroizing<String>,
    pub owner_name: Zeroizing<String>,
    pub model: String,
}

pub fn load() -> Result<Secrets, String> {
    assemble(
        getfrompass::try_get_from_pass("machine/pdf-scrub/source-dir"),
        getfrompass::try_get_from_pass("machine/pdf-scrub/dest-dir"),
        getfrompass::try_get_from_pass("machine/pdf-scrub/owner-name"),
        getfrompass::try_get_from_pass("machine/pdf-scrub/model"),
    )
}

fn assemble(
    source_dir: Option<Zeroizing<String>>,
    dest_dir: Option<Zeroizing<String>>,
    owner_name: Option<Zeroizing<String>>,
    model: Option<Zeroizing<String>>,
) -> Result<Secrets, String> {
    Ok(Secrets {
        source_dir: source_dir.ok_or("configuration incomplete")?,
        dest_dir: dest_dir.ok_or("configuration incomplete")?,
        owner_name: owner_name.ok_or("configuration incomplete")?,
        model: model
            .map(|z| z.to_string())
            .unwrap_or_else(|| DEFAULT_MODEL.to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn z(s: &str) -> Option<Zeroizing<String>> {
        Some(Zeroizing::new(s.to_string()))
    }

    #[test]
    fn all_required_entries_present_returns_ok() {
        let result = assemble(
            z("/src"),
            z("/dst"),
            z("Matti Meikäläinen"),
            z("custom-model"),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn missing_source_dir_returns_err() {
        let result = assemble(None, z("/dst"), z("Matti Meikäläinen"), None);
        assert!(result.is_err());
    }

    #[test]
    fn missing_dest_dir_returns_err() {
        let result = assemble(z("/src"), None, z("Matti Meikäläinen"), None);
        assert!(result.is_err());
    }

    #[test]
    fn missing_owner_name_returns_err() {
        let result = assemble(z("/src"), z("/dst"), None, None);
        assert!(result.is_err());
    }

    #[test]
    fn missing_model_falls_back_to_default() {
        let result = assemble(z("/src"), z("/dst"), z("Matti Meikäläinen"), None).unwrap();
        assert_eq!(result.model, DEFAULT_MODEL);
    }

    #[test]
    fn provided_model_is_used() {
        let result = assemble(
            z("/src"),
            z("/dst"),
            z("Matti Meikäläinen"),
            z("custom-model"),
        )
        .unwrap();
        assert_eq!(result.model, "custom-model");
    }

    #[cfg(feature = "local")]
    #[test]
    fn load_returns_ok_with_pass_entries_present() {
        assert!(load().is_ok());
    }
}
