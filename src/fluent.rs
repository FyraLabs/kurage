pub fn handle_l10n() -> i18n_embed::fluent::FluentLanguageLoader {
    use i18n_embed::{
        fluent::fluent_language_loader,
        unic_langid::{subtags::Script, LanguageIdentifier},
        LanguageLoader,
    };
    use std::str::FromStr;
    let loader = fluent_language_loader!();
    let available_langs = loader.available_languages(&Localizations).unwrap();
    let mut langs = ["LC_ALL", "LC_MESSAGES", "LANG", "LANGUAGE", "LANGUAGES"]
        .into_iter()
        .flat_map(|env| {
            std::env::var(env).ok().into_iter().flat_map(|locales| {
                locales
                    .split(':')
                    .filter_map(|locale| LanguageIdentifier::from_str(locale).ok())
                    .collect_vec()
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
        .collect_vec();
    if langs.is_empty() {
        langs = vec![loader.fallback_language().clone()];
    }
    loader.load_languages(&Localizations, &langs).unwrap();
    loader
}
