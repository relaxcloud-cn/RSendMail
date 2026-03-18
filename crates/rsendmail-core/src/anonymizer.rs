use log::debug;
use rand::{distributions::Alphanumeric, Rng};
use regex::Regex;
use std::collections::HashMap;

pub struct EmailAnonymizer {
    email_regex: Regex,
    map: HashMap<String, String>,
    target_domain: String,
}

impl EmailAnonymizer {
    pub fn new(target_domain: &str) -> Self {
        Self {
            // 匹配大多数标准格式的邮箱
            email_regex: Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap(),
            map: HashMap::new(),
            target_domain: target_domain.to_string(),
        }
    }

    // 对文本内容进行匿名化处理
    // 使用 replace_all 一次性替换，避免子串匹配导致的双重替换问题
    pub fn anonymize_text(&mut self, text: &str) -> String {
        let anonymizer = &mut self.map;
        let domain = &self.target_domain;

        self.email_regex
            .replace_all(text, |caps: &regex::Captures| {
                let email = caps[0].to_string();
                anonymizer
                    .entry(email.clone())
                    .or_insert_with(|| {
                        let random_string: String = rand::thread_rng()
                            .sample_iter(&Alphanumeric)
                            .take(8)
                            .map(|c| c as char)
                            .collect();
                        let anonymized = format!("{}@{}", random_string, domain);
                        debug!("匿名化邮箱: {} -> {}", email, anonymized);
                        anonymized
                    })
                    .clone()
            })
            .to_string()
    }

    // 对二进制内容（如邮件文件）进行匿名化处理
    pub fn anonymize_binary(&mut self, content: &[u8]) -> Vec<u8> {
        let text = match std::str::from_utf8(content) {
            Ok(s) => s,
            Err(_) => return content.to_vec(), // 如果无法解析为UTF-8，则返回原内容
        };

        let anonymized = self.anonymize_text(text);
        anonymized.into_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anonymize_text() {
        let mut anonymizer = EmailAnonymizer::new("example.com");

        let text = "联系我: user@domain.com 或者 another.user@example.org";
        let anonymized = anonymizer.anonymize_text(text);

        // 验证原邮箱已被替换
        assert!(!anonymized.contains("user@domain.com"));
        assert!(!anonymized.contains("another.user@example.org"));

        // 验证替换后的邮箱格式正确
        let regex = Regex::new(r"[a-zA-Z0-9]+@example\.com").unwrap();
        assert!(regex.find_iter(&anonymized).count() == 2);

        // 验证对同一邮箱的多次替换保持一致
        let text2 = "再次联系: user@domain.com";
        let anonymized2 = anonymizer.anonymize_text(text2);

        let first_replacement = anonymized
            .split_whitespace()
            .find(|s| s.contains("@example.com"))
            .unwrap();

        assert!(anonymized2.contains(first_replacement));
    }
}
