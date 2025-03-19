use itertools::Itertools;
/// Set the correct language for the loader.
///
/// # Examples
///
/// Example taken from Taidan.
///
/// ```rs,ignore
/// #[derive(rust_embed::RustEmbed)]
/// #[folder = "po/"]
/// struct Localizations;
///
/// let loader = handle_l10n(&Localizations, fluent_language_loader!())
/// ```
///
/// Note that the usage of [`i18n_embed::fluent::fluent_language_loader!`]
/// requires the `i18n.toml` file.
pub fn handle_l10n(
    i18n_assets: &dyn i18n_embed::I18nAssets,
    loader: i18n_embed::fluent::FluentLanguageLoader,
) -> i18n_embed::fluent::FluentLanguageLoader {
    use i18n_embed::{
        unic_langid::{subtags::Script, LanguageIdentifier},
        LanguageLoader,
    };
    use std::str::FromStr;
    let available_langs = loader.available_languages(i18n_assets).unwrap();
    let mut langs = ["LC_ALL", "LC_MESSAGES", "LANG", "LANGUAGE", "LANGUAGES"]
        .into_iter()
        .flat_map(|env| {
            std::env::var(env).ok().into_iter().flat_map(|locales| {
                locales
                    .split(':')
                    .filter_map(|locale| LanguageIdentifier::from_str(locale).ok())
                    .collect::<Vec<_>>()
            })
        })
        .update(|li| {
            if li.language == "zh" {
                if available_langs.iter().contains(li) {
                } else if li.script.is_some() {
                    li.clear_variants();
                } else if li
                    .region
                    .is_some_and(|region| ["HK", "TW", "MO"].contains(&region.as_str()))
                {
                    li.script = Some(Script::from_bytes(b"Hant").unwrap());
                } else {
                    li.script = Some(Script::from_bytes(b"Hans").unwrap());
                }
            }
        })
        .collect::<Vec<_>>();
    if langs.is_empty() {
        langs = vec![loader.fallback_language().clone()];
    }
    loader.load_languages(i18n_assets, &langs).unwrap();
    loader
}
