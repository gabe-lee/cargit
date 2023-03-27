use gmec::{patterns::PatternMatcher, types::error_chain::{ErrorChain, ErrorPropogation}};

pub(crate) const CARGO_MANIFEST: &str = "Cargo.toml";
pub(crate) const VERSION_PREFIX: &str = "version = \"";

pub(crate) fn split_version_from_cargo_toml<'a>(cargo_toml_str: &'a str) -> Result<(&'a str, &'a str, &'a str), ErrorChain> {
    let version_prefix_location = cargo_toml_str.find_first(&VERSION_PREFIX).on_error("could not locate the string 'version = \"' in the Cargo.toml file")?;
    let version_end_location = cargo_toml_str.find_first_from(&"\"", version_prefix_location.end()).on_error("could not locate a '\"' after the version numbers in the Cargo.toml file")?;
    let before_version = &cargo_toml_str[..version_prefix_location.end()];
    let version_string = &cargo_toml_str[version_prefix_location.end()..version_end_location.start()];
    let after_version = &cargo_toml_str[version_end_location.start()..];
    return Ok((before_version, version_string, after_version))
}

pub(crate) fn split_parts_from_version<'a>(version_string: &'a str) -> Result<(&'a str, &'a str, &'a str), ErrorChain> {
    let first_version_period = version_string.find_first(&".").on_error("could not locate a '.' after the major version")?;
    let second_version_period = version_string.find_first_from(&".", first_version_period.end()).on_error("could not locate a '.' after the minor version")?;
    let major = &version_string[..first_version_period.start()];
    let minor = &version_string[first_version_period.end()..second_version_period.start()];
    let patch = &version_string[second_version_period.end()..];
    return Ok((major, minor, patch));
}