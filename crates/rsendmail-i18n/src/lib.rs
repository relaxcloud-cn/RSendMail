//! Internationalization (i18n) module for RSendMail
//!
//! This crate provides shared internationalization support for all RSendMail components
//! (CLI, Core, and GUI). It uses the `rust-i18n` crate for translation management.
//!
//! # Supported Languages
//! - English (en-US) - Default/Fallback
//! - Simplified Chinese (zh-CN)
//! - Traditional Chinese (zh-TW)
//! - Japanese (ja-JP)
//! - Korean (ko-KR)
//!
//! # Usage
//! ```rust,ignore
//! use rsendmail_i18n::{t, set_language, Language};
//!
//! // Set language
//! set_language(Language::SimplifiedChinese);
//!
//! // Use translations
//! println!("{}", t!("cli.smtp_server"));
//! println!("{}", t!("core.mailer.email_send_success", path = "test.eml"));
//! ```

rust_i18n::i18n!("locales", fallback = "en-US");

/// Supported languages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Language {
    #[default]
    English,
    SimplifiedChinese,
    TraditionalChinese,
    Japanese,
    Korean,
}

impl Language {
    /// Get the locale code for this language
    pub fn locale_code(&self) -> &'static str {
        match self {
            Language::English => "en-US",
            Language::SimplifiedChinese => "zh-CN",
            Language::TraditionalChinese => "zh-TW",
            Language::Japanese => "ja-JP",
            Language::Korean => "ko-KR",
        }
    }

    /// Get the display name for this language (in its native form)
    pub fn name(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::SimplifiedChinese => "简体中文",
            Language::TraditionalChinese => "繁體中文",
            Language::Japanese => "日本語",
            Language::Korean => "한국어",
        }
    }

    /// Get the short code for CLI argument
    pub fn short_code(&self) -> &'static str {
        match self {
            Language::English => "en",
            Language::SimplifiedChinese => "zh-CN",
            Language::TraditionalChinese => "zh-TW",
            Language::Japanese => "ja",
            Language::Korean => "ko",
        }
    }

    /// Parse language from string (supports various formats)
    pub fn parse(s: &str) -> Option<Self> {
        s.parse().ok()
    }

    /// Detect language from system environment
    pub fn from_system() -> Self {
        // Check RSENDMAIL_LANG first
        if let Ok(lang) = std::env::var("RSENDMAIL_LANG") {
            if let Some(l) = Self::parse(&lang) {
                return l;
            }
        }

        // Check standard environment variables
        if let Ok(lang) = std::env::var("LANG") {
            if let Some(l) = Self::from_locale_string(&lang) {
                return l;
            }
        }
        if let Ok(lang) = std::env::var("LC_ALL") {
            if let Some(l) = Self::from_locale_string(&lang) {
                return l;
            }
        }

        // macOS specific handling
        #[cfg(target_os = "macos")]
        {
            if let Ok(output) = std::process::Command::new("defaults")
                .args(["read", "-g", "AppleLocale"])
                .output()
            {
                let locale = String::from_utf8_lossy(&output.stdout);
                if let Some(l) = Self::from_locale_string(&locale) {
                    return l;
                }
            }
        }

        Self::default()
    }

    fn from_locale_string(s: &str) -> Option<Self> {
        let s = s.to_lowercase();
        if s.starts_with("zh_cn") || s.starts_with("zh-cn") || s.starts_with("zh-hans") {
            Some(Language::SimplifiedChinese)
        } else if s.starts_with("zh_tw")
            || s.starts_with("zh-tw")
            || s.starts_with("zh_hk")
            || s.starts_with("zh-hk")
            || s.starts_with("zh-hant")
        {
            Some(Language::TraditionalChinese)
        } else if s.starts_with("ja") {
            Some(Language::Japanese)
        } else if s.starts_with("ko") {
            Some(Language::Korean)
        } else if s.starts_with("en") {
            Some(Language::English)
        } else {
            None
        }
    }

    /// Get language from index (for GUI dropdown)
    pub fn from_index(index: usize) -> Self {
        match index {
            0 => Language::English,
            1 => Language::SimplifiedChinese,
            2 => Language::TraditionalChinese,
            3 => Language::Japanese,
            4 => Language::Korean,
            _ => Self::default(),
        }
    }

    /// Get index for this language (for GUI dropdown)
    pub fn index(&self) -> usize {
        match self {
            Language::English => 0,
            Language::SimplifiedChinese => 1,
            Language::TraditionalChinese => 2,
            Language::Japanese => 3,
            Language::Korean => 4,
        }
    }

    /// Get all supported languages
    pub fn all() -> &'static [Language] {
        &[
            Language::English,
            Language::SimplifiedChinese,
            Language::TraditionalChinese,
            Language::Japanese,
            Language::Korean,
        ]
    }

    /// Get all language names (for GUI dropdown)
    pub fn all_names() -> Vec<&'static str> {
        Self::all().iter().map(|l| l.name()).collect()
    }
}

impl std::str::FromStr for Language {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        match s.as_str() {
            "en" | "en-us" | "en_us" | "english" => Ok(Language::English),
            "zh-cn" | "zh_cn" | "zh-hans" | "zh" | "chinese" => {
                Ok(Language::SimplifiedChinese)
            }
            "zh-tw" | "zh_tw" | "zh-hant" | "zh-hk" | "zh_hk" => {
                Ok(Language::TraditionalChinese)
            }
            "ja" | "ja-jp" | "ja_jp" | "japanese" => Ok(Language::Japanese),
            "ko" | "ko-kr" | "ko_kr" | "korean" => Ok(Language::Korean),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Set the current language for all translations
pub fn set_language(lang: Language) {
    rust_i18n::set_locale(lang.locale_code());
}

/// Get the current language
pub fn current_language() -> Language {
    let locale = rust_i18n::locale();
    match &*locale {
        "zh-CN" => Language::SimplifiedChinese,
        "zh-TW" => Language::TraditionalChinese,
        "ja-JP" => Language::Japanese,
        "ko-KR" => Language::Korean,
        _ => Language::English,
    }
}

/// Initialize i18n with system language detection
pub fn init() {
    let lang = Language::from_system();
    set_language(lang);
}

/// Translate a key to the current language
/// This is a wrapper function that can be called from other crates
pub fn tr(key: &str) -> String {
    rust_i18n::t!(key).to_string()
}

/// Translate a key with arguments
/// Args should be in format: "key1=value1,key2=value2"
pub fn tr_with_args(key: &str, args: &[(&str, &str)]) -> String {
    // rust-i18n t! macro doesn't support dynamic args easily
    // We need to build the string manually
    let mut result = rust_i18n::t!(key).to_string();
    for (k, v) in args {
        result = result.replace(&format!("%{{{}}}", k), v);
    }
    result
}

// Re-export for crates that want to use the macro directly
// Note: Using t! from other crates requires i18n! to be called in that crate too
pub use rust_i18n::t;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_from_str() {
        assert_eq!(Language::parse("en"), Some(Language::English));
        assert_eq!(
            Language::parse("zh-CN"),
            Some(Language::SimplifiedChinese)
        );
        assert_eq!(
            Language::parse("zh-TW"),
            Some(Language::TraditionalChinese)
        );
        assert_eq!(Language::parse("ja"), Some(Language::Japanese));
        assert_eq!(Language::parse("unknown"), None);
    }

    #[test]
    fn test_language_locale_code() {
        assert_eq!(Language::English.locale_code(), "en-US");
        assert_eq!(Language::SimplifiedChinese.locale_code(), "zh-CN");
        assert_eq!(Language::TraditionalChinese.locale_code(), "zh-TW");
        assert_eq!(Language::Japanese.locale_code(), "ja-JP");
    }

    #[test]
    fn test_language_index() {
        assert_eq!(Language::English.index(), 0);
        assert_eq!(Language::SimplifiedChinese.index(), 1);
        assert_eq!(Language::from_index(2), Language::TraditionalChinese);
        assert_eq!(Language::from_index(3), Language::Japanese);
    }
}
