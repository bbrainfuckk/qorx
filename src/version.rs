use std::env;

pub const QORX_OWNER_PRODUCT: &str = "Qorx Ayie";
pub const QORX_PRODUCT: &str = "Qorx Void";
pub const QORX_DEMO_PRODUCT: &str = "Qorx Void Demo";
pub const QORX_VERSION: &str = "0.0.1-ylem";

pub fn product_name() -> &'static str {
    let edition = env::var("QORX_EDITION").ok();
    let owner = env::var("QORX_OWNER").ok();
    product_name_for_mode(
        crate::demo::is_demo_mode(),
        edition.as_deref(),
        owner.as_deref(),
    )
}

pub fn runtime_edition() -> &'static str {
    let edition = env::var("QORX_EDITION").ok();
    let owner = env::var("QORX_OWNER").ok();
    runtime_edition_for_mode(
        crate::demo::is_demo_mode(),
        edition.as_deref(),
        owner.as_deref(),
    )
}

fn product_name_for_mode(
    demo_mode: bool,
    edition: Option<&str>,
    owner: Option<&str>,
) -> &'static str {
    if demo_mode {
        QORX_DEMO_PRODUCT
    } else if is_owner_ayie_mode(edition, owner) {
        QORX_OWNER_PRODUCT
    } else {
        QORX_PRODUCT
    }
}

fn runtime_edition_for_mode(
    demo_mode: bool,
    edition: Option<&str>,
    owner: Option<&str>,
) -> &'static str {
    if demo_mode {
        "demo"
    } else if is_owner_ayie_mode(edition, owner) {
        "ayie"
    } else {
        "void"
    }
}

fn is_owner_ayie_mode(edition: Option<&str>, owner: Option<&str>) -> bool {
    owner
        .map(|value| matches_normalized(value, &["1", "true", "yes", "on", "ayie", "owner"]))
        .unwrap_or(false)
        || edition
            .map(|value| matches_normalized(value, &["ayie", "owner", "private", "internal"]))
            .unwrap_or(false)
}

fn matches_normalized(value: &str, choices: &[&str]) -> bool {
    let normalized = value.trim().to_ascii_lowercase();
    choices.iter().any(|choice| normalized == *choice)
}

#[cfg(test)]
pub(crate) fn product_name_for_test(
    demo_mode: bool,
    edition: Option<&str>,
    owner: Option<&str>,
) -> &'static str {
    product_name_for_mode(demo_mode, edition, owner)
}

#[cfg(test)]
pub(crate) fn runtime_edition_for_test(
    demo_mode: bool,
    edition: Option<&str>,
    owner: Option<&str>,
) -> &'static str {
    runtime_edition_for_mode(demo_mode, edition, owner)
}

#[cfg(test)]
mod tests {
    #[test]
    fn package_version_tracks_void_release() {
        assert_eq!(env!("CARGO_PKG_VERSION"), "0.0.1-ylem");
        assert_eq!(super::QORX_VERSION, "0.0.1-ylem");
    }

    #[test]
    fn runtime_defaults_to_public_qorx_void() {
        assert_eq!(
            super::product_name_for_test(false, None, None),
            super::QORX_PRODUCT
        );
        assert_eq!(super::runtime_edition_for_test(false, None, None), "void");
    }

    #[test]
    fn ayie_void_and_demo_keep_their_names() {
        assert_eq!(
            super::product_name_for_test(false, Some("ayie"), None),
            super::QORX_OWNER_PRODUCT
        );
        assert_eq!(
            super::product_name_for_test(false, None, Some("1")),
            super::QORX_OWNER_PRODUCT
        );
        assert_eq!(
            super::product_name_for_test(false, Some("public"), None),
            super::QORX_PRODUCT
        );
        assert_eq!(
            super::product_name_for_test(false, None, Some("0")),
            super::QORX_PRODUCT
        );
        assert_eq!(
            super::product_name_for_test(true, Some("public"), Some("0")),
            super::QORX_DEMO_PRODUCT
        );
    }
}
