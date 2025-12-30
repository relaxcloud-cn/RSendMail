//! 国际化 (i18n) 模块
//!
//! 简化的翻译实现，使用 HashMap 存储翻译字符串

use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

/// 支持的语言
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    English,
    SimplifiedChinese,
    TraditionalChinese,
    Japanese,
}

impl Language {
    /// 语言名称（本地化）
    pub fn name(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::SimplifiedChinese => "简体中文",
            Language::TraditionalChinese => "繁體中文",
            Language::Japanese => "日本語",
        }
    }

    /// 从系统语言检测
    pub fn from_system() -> Self {
        // 检测环境变量
        if let Ok(lang) = std::env::var("LANG") {
            return Self::from_locale_string(&lang);
        }
        if let Ok(lang) = std::env::var("LC_ALL") {
            return Self::from_locale_string(&lang);
        }

        // macOS 特殊处理
        #[cfg(target_os = "macos")]
        {
            if let Ok(output) = std::process::Command::new("defaults")
                .args(["read", "-g", "AppleLocale"])
                .output()
            {
                let locale = String::from_utf8_lossy(&output.stdout);
                return Self::from_locale_string(&locale);
            }
        }

        Language::English
    }

    fn from_locale_string(s: &str) -> Self {
        let s = s.to_lowercase();
        if s.starts_with("zh_cn") || s.starts_with("zh-cn") || s.starts_with("zh-hans") {
            Language::SimplifiedChinese
        } else if s.starts_with("zh_tw")
            || s.starts_with("zh-tw")
            || s.starts_with("zh_hk")
            || s.starts_with("zh-hk")
            || s.starts_with("zh-hant")
        {
            Language::TraditionalChinese
        } else if s.starts_with("ja") {
            Language::Japanese
        } else {
            Language::English
        }
    }

    /// 从索引获取语言
    pub fn from_index(index: usize) -> Self {
        match index {
            0 => Language::English,
            1 => Language::SimplifiedChinese,
            2 => Language::TraditionalChinese,
            3 => Language::Japanese,
            _ => Language::English,
        }
    }

    /// 获取索引
    pub fn index(self) -> usize {
        match self {
            Language::English => 0,
            Language::SimplifiedChinese => 1,
            Language::TraditionalChinese => 2,
            Language::Japanese => 3,
        }
    }
}

/// 当前语言
static CURRENT_LANGUAGE: LazyLock<RwLock<Language>> =
    LazyLock::new(|| RwLock::new(Language::from_system()));

/// 翻译表
static TRANSLATIONS: LazyLock<HashMap<Language, HashMap<&'static str, &'static str>>> =
    LazyLock::new(|| {
        let mut map = HashMap::new();

        // English translations
        let mut en = HashMap::new();
        en.insert("app-title", "RSendMail");
        en.insert("smtp-server", "SMTP Server");
        en.insert("server-address", "Server Address");
        en.insert("port", "Port");
        en.insert("use-tls", "Use TLS");
        en.insert("accept-invalid-certs", "Accept Invalid Certificates");
        en.insert("auth-required", "Authentication Required");
        en.insert("username", "Username");
        en.insert("password", "Password");
        en.insert("sender", "Sender");
        en.insert("recipient", "Recipient");
        en.insert("recipient-hint", "(comma separated for multiple)");
        en.insert("send-mode", "Send Mode");
        en.insert("eml-batch", "EML Batch");
        en.insert("single-attachment", "Single Attachment");
        en.insert("dir-attachment", "Directory Attachment");
        en.insert("eml-directory", "EML Directory");
        en.insert("attachment-file", "Attachment File");
        en.insert("attachment-directory", "Attachment Directory");
        en.insert("extension", "Extension");
        en.insert("browse", "Browse...");
        en.insert("email-subject", "Email Subject");
        en.insert("email-body", "Email Body");
        en.insert(
            "filename-hint",
            "Use {filename} for auto filename insertion",
        );
        en.insert("advanced-options", "Advanced Options");
        en.insert("performance", "Performance");
        en.insert("processes", "Processes");
        en.insert("batch-size", "Batch Size");
        en.insert("send-interval", "Send Interval (ms)");
        en.insert("timeout", "Timeout (sec)");
        en.insert("loop-settings", "Loop Settings");
        en.insert("infinite-loop", "Infinite Loop");
        en.insert("repeat-count", "Repeat Count");
        en.insert("loop-interval", "Loop Interval (sec)");
        en.insert("retry-interval", "Retry Interval (sec)");
        en.insert("email-processing", "Email Processing");
        en.insert("keep-headers", "Keep Original Headers");
        en.insert("modify-headers", "Modify Headers");
        en.insert("anonymize-emails", "Anonymize Emails");
        en.insert("domain", "Domain");
        en.insert("logging", "Logging & Error Handling");
        en.insert("log-level", "Log Level");
        en.insert("log-file", "Log File");
        en.insert("failed-emails-dir", "Failed Emails Directory");
        en.insert("optional", "(optional)");
        en.insert("statistics", "Statistics");
        en.insert("total", "Total");
        en.insert("success", "Success");
        en.insert("failed", "Failed");
        en.insert("current-round", "Current Round");
        en.insert("elapsed-time", "Elapsed Time");
        en.insert("send-log", "Send Log");
        en.insert("clear", "Clear");
        en.insert("export-log", "Export Log");
        en.insert("save-config", "Save Config");
        en.insert("load-config", "Load Config");
        en.insert("test-connection", "Test Connection");
        en.insert("start-send", "Start Send");
        en.insert("stop-send", "Stop Send");
        en.insert("language", "Language");
        en.insert("status-ready", "Ready");
        en.insert("status-preparing", "Preparing...");
        en.insert("status-sending", "Sending...");
        en.insert("status-stopped", "Stopped");
        en.insert("status-completed", "Completed");
        // Error messages
        en.insert("error-title", "Error");
        en.insert("error-no-smtp-server", "Please enter SMTP server address");
        en.insert("error-no-sender", "Please enter sender address");
        en.insert("error-no-recipient", "Please enter recipient address");
        en.insert("error-no-eml-dir", "Please select EML directory");
        en.insert("error-no-attachment", "Please select attachment file");
        en.insert("error-no-attachment-dir", "Please select attachment directory");
        en.insert("error-no-username", "Authentication requires username");
        en.insert("error-no-password", "Authentication requires password");
        map.insert(Language::English, en);

        // Simplified Chinese translations
        let mut zh_cn = HashMap::new();
        zh_cn.insert("app-title", "RSendMail");
        zh_cn.insert("smtp-server", "SMTP 服务器");
        zh_cn.insert("server-address", "服务器地址");
        zh_cn.insert("port", "端口");
        zh_cn.insert("use-tls", "使用 TLS");
        zh_cn.insert("accept-invalid-certs", "接受自签名证书");
        zh_cn.insert("auth-required", "需要认证");
        zh_cn.insert("username", "用户名");
        zh_cn.insert("password", "密码");
        zh_cn.insert("sender", "发件人");
        zh_cn.insert("recipient", "收件人");
        zh_cn.insert("recipient-hint", "(多个地址请用逗号分隔)");
        zh_cn.insert("send-mode", "发送模式");
        zh_cn.insert("eml-batch", "EML 批量");
        zh_cn.insert("single-attachment", "单个附件");
        zh_cn.insert("dir-attachment", "目录附件");
        zh_cn.insert("eml-directory", "EML 目录");
        zh_cn.insert("attachment-file", "附件文件");
        zh_cn.insert("attachment-directory", "附件目录");
        zh_cn.insert("extension", "扩展名");
        zh_cn.insert("browse", "浏览...");
        zh_cn.insert("email-subject", "邮件主题");
        zh_cn.insert("email-body", "邮件正文");
        zh_cn.insert("filename-hint", "使用 {filename} 自动插入文件名");
        zh_cn.insert("advanced-options", "高级选项");
        zh_cn.insert("performance", "性能配置");
        zh_cn.insert("processes", "进程数");
        zh_cn.insert("batch-size", "批量大小");
        zh_cn.insert("send-interval", "发送间隔(ms)");
        zh_cn.insert("timeout", "超时(秒)");
        zh_cn.insert("loop-settings", "循环发送");
        zh_cn.insert("infinite-loop", "无限循环");
        zh_cn.insert("repeat-count", "重复次数");
        zh_cn.insert("loop-interval", "循环间隔(秒)");
        zh_cn.insert("retry-interval", "重试间隔(秒)");
        zh_cn.insert("email-processing", "邮件处理");
        zh_cn.insert("keep-headers", "保留原始邮件头");
        zh_cn.insert("modify-headers", "修改邮件头");
        zh_cn.insert("anonymize-emails", "匿名化邮箱");
        zh_cn.insert("domain", "域名");
        zh_cn.insert("logging", "日志与错误处理");
        zh_cn.insert("log-level", "日志级别");
        zh_cn.insert("log-file", "日志文件");
        zh_cn.insert("failed-emails-dir", "失败邮件");
        zh_cn.insert("optional", "(可选)");
        zh_cn.insert("statistics", "发送统计");
        zh_cn.insert("total", "总计");
        zh_cn.insert("success", "成功");
        zh_cn.insert("failed", "失败");
        zh_cn.insert("current-round", "当前轮次");
        zh_cn.insert("elapsed-time", "已用时间");
        zh_cn.insert("send-log", "发送日志");
        zh_cn.insert("clear", "清空");
        zh_cn.insert("export-log", "导出日志");
        zh_cn.insert("save-config", "保存配置");
        zh_cn.insert("load-config", "加载配置");
        zh_cn.insert("test-connection", "测试连接");
        zh_cn.insert("start-send", "开始发送");
        zh_cn.insert("stop-send", "停止发送");
        zh_cn.insert("language", "语言");
        zh_cn.insert("status-ready", "就绪");
        zh_cn.insert("status-preparing", "准备中...");
        zh_cn.insert("status-sending", "发送中...");
        zh_cn.insert("status-stopped", "已停止");
        zh_cn.insert("status-completed", "完成");
        // Error messages
        zh_cn.insert("error-title", "错误");
        zh_cn.insert("error-no-smtp-server", "请输入 SMTP 服务器地址");
        zh_cn.insert("error-no-sender", "请输入发件人地址");
        zh_cn.insert("error-no-recipient", "请输入收件人地址");
        zh_cn.insert("error-no-eml-dir", "请选择 EML 文件目录");
        zh_cn.insert("error-no-attachment", "请选择附件文件");
        zh_cn.insert("error-no-attachment-dir", "请选择附件目录");
        zh_cn.insert("error-no-username", "认证模式需要输入用户名");
        zh_cn.insert("error-no-password", "认证模式需要输入密码");
        map.insert(Language::SimplifiedChinese, zh_cn);

        // Traditional Chinese translations
        let mut zh_tw = HashMap::new();
        zh_tw.insert("app-title", "RSendMail");
        zh_tw.insert("smtp-server", "SMTP 伺服器");
        zh_tw.insert("server-address", "伺服器地址");
        zh_tw.insert("port", "連接埠");
        zh_tw.insert("use-tls", "使用 TLS");
        zh_tw.insert("accept-invalid-certs", "接受自簽名憑證");
        zh_tw.insert("auth-required", "需要認證");
        zh_tw.insert("username", "使用者名稱");
        zh_tw.insert("password", "密碼");
        zh_tw.insert("sender", "寄件人");
        zh_tw.insert("recipient", "收件人");
        zh_tw.insert("recipient-hint", "(多個地址請用逗號分隔)");
        zh_tw.insert("send-mode", "發送模式");
        zh_tw.insert("eml-batch", "EML 批次");
        zh_tw.insert("single-attachment", "單一附件");
        zh_tw.insert("dir-attachment", "目錄附件");
        zh_tw.insert("eml-directory", "EML 目錄");
        zh_tw.insert("attachment-file", "附件檔案");
        zh_tw.insert("attachment-directory", "附件目錄");
        zh_tw.insert("extension", "副檔名");
        zh_tw.insert("browse", "瀏覽...");
        zh_tw.insert("email-subject", "郵件主旨");
        zh_tw.insert("email-body", "郵件內文");
        zh_tw.insert("filename-hint", "使用 {filename} 自動插入檔案名稱");
        zh_tw.insert("advanced-options", "進階選項");
        zh_tw.insert("performance", "效能設定");
        zh_tw.insert("processes", "處理程序數");
        zh_tw.insert("batch-size", "批次大小");
        zh_tw.insert("send-interval", "發送間隔(ms)");
        zh_tw.insert("timeout", "逾時(秒)");
        zh_tw.insert("loop-settings", "循環發送");
        zh_tw.insert("infinite-loop", "無限循環");
        zh_tw.insert("repeat-count", "重複次數");
        zh_tw.insert("loop-interval", "循環間隔(秒)");
        zh_tw.insert("retry-interval", "重試間隔(秒)");
        zh_tw.insert("email-processing", "郵件處理");
        zh_tw.insert("keep-headers", "保留原始郵件標頭");
        zh_tw.insert("modify-headers", "修改郵件標頭");
        zh_tw.insert("anonymize-emails", "匿名化郵箱");
        zh_tw.insert("domain", "網域");
        zh_tw.insert("logging", "日誌與錯誤處理");
        zh_tw.insert("log-level", "日誌等級");
        zh_tw.insert("log-file", "日誌檔案");
        zh_tw.insert("failed-emails-dir", "失敗郵件");
        zh_tw.insert("optional", "(選填)");
        zh_tw.insert("statistics", "發送統計");
        zh_tw.insert("total", "總計");
        zh_tw.insert("success", "成功");
        zh_tw.insert("failed", "失敗");
        zh_tw.insert("current-round", "目前輪次");
        zh_tw.insert("elapsed-time", "已用時間");
        zh_tw.insert("send-log", "發送日誌");
        zh_tw.insert("clear", "清空");
        zh_tw.insert("export-log", "匯出日誌");
        zh_tw.insert("save-config", "儲存設定");
        zh_tw.insert("load-config", "載入設定");
        zh_tw.insert("test-connection", "測試連線");
        zh_tw.insert("start-send", "開始發送");
        zh_tw.insert("stop-send", "停止發送");
        zh_tw.insert("language", "語言");
        zh_tw.insert("status-ready", "就緒");
        zh_tw.insert("status-preparing", "準備中...");
        zh_tw.insert("status-sending", "發送中...");
        zh_tw.insert("status-stopped", "已停止");
        zh_tw.insert("status-completed", "完成");
        // Error messages
        zh_tw.insert("error-title", "錯誤");
        zh_tw.insert("error-no-smtp-server", "請輸入 SMTP 伺服器地址");
        zh_tw.insert("error-no-sender", "請輸入寄件人地址");
        zh_tw.insert("error-no-recipient", "請輸入收件人地址");
        zh_tw.insert("error-no-eml-dir", "請選擇 EML 檔案目錄");
        zh_tw.insert("error-no-attachment", "請選擇附件檔案");
        zh_tw.insert("error-no-attachment-dir", "請選擇附件目錄");
        zh_tw.insert("error-no-username", "認證模式需要輸入使用者名稱");
        zh_tw.insert("error-no-password", "認證模式需要輸入密碼");
        map.insert(Language::TraditionalChinese, zh_tw);

        // Japanese translations
        let mut ja = HashMap::new();
        ja.insert("app-title", "RSendMail");
        ja.insert("smtp-server", "SMTP サーバー");
        ja.insert("server-address", "サーバーアドレス");
        ja.insert("port", "ポート");
        ja.insert("use-tls", "TLS を使用");
        ja.insert("accept-invalid-certs", "自己署名証明書を許可");
        ja.insert("auth-required", "認証が必要");
        ja.insert("username", "ユーザー名");
        ja.insert("password", "パスワード");
        ja.insert("sender", "送信者");
        ja.insert("recipient", "受信者");
        ja.insert("recipient-hint", "(複数はカンマ区切り)");
        ja.insert("send-mode", "送信モード");
        ja.insert("eml-batch", "EML 一括");
        ja.insert("single-attachment", "単一添付");
        ja.insert("dir-attachment", "フォルダ添付");
        ja.insert("eml-directory", "EML フォルダ");
        ja.insert("attachment-file", "添付ファイル");
        ja.insert("attachment-directory", "添付フォルダ");
        ja.insert("extension", "拡張子");
        ja.insert("browse", "参照...");
        ja.insert("email-subject", "メール件名");
        ja.insert("email-body", "メール本文");
        ja.insert("filename-hint", "{filename} でファイル名を自動挿入");
        ja.insert("advanced-options", "詳細オプション");
        ja.insert("performance", "パフォーマンス設定");
        ja.insert("processes", "プロセス数");
        ja.insert("batch-size", "バッチサイズ");
        ja.insert("send-interval", "送信間隔(ms)");
        ja.insert("timeout", "タイムアウト(秒)");
        ja.insert("loop-settings", "ループ送信");
        ja.insert("infinite-loop", "無限ループ");
        ja.insert("repeat-count", "繰り返し回数");
        ja.insert("loop-interval", "ループ間隔(秒)");
        ja.insert("retry-interval", "リトライ間隔(秒)");
        ja.insert("email-processing", "メール処理");
        ja.insert("keep-headers", "元のヘッダーを保持");
        ja.insert("modify-headers", "ヘッダーを変更");
        ja.insert("anonymize-emails", "メールを匿名化");
        ja.insert("domain", "ドメイン");
        ja.insert("logging", "ログとエラー処理");
        ja.insert("log-level", "ログレベル");
        ja.insert("log-file", "ログファイル");
        ja.insert("failed-emails-dir", "失敗メール");
        ja.insert("optional", "(任意)");
        ja.insert("statistics", "送信統計");
        ja.insert("total", "合計");
        ja.insert("success", "成功");
        ja.insert("failed", "失敗");
        ja.insert("current-round", "現在のラウンド");
        ja.insert("elapsed-time", "経過時間");
        ja.insert("send-log", "送信ログ");
        ja.insert("clear", "クリア");
        ja.insert("export-log", "ログをエクスポート");
        ja.insert("save-config", "設定を保存");
        ja.insert("load-config", "設定を読み込み");
        ja.insert("test-connection", "接続テスト");
        ja.insert("start-send", "送信開始");
        ja.insert("stop-send", "送信停止");
        ja.insert("language", "言語");
        ja.insert("status-ready", "準備完了");
        ja.insert("status-preparing", "準備中...");
        ja.insert("status-sending", "送信中...");
        ja.insert("status-stopped", "停止");
        ja.insert("status-completed", "完了");
        // Error messages
        ja.insert("error-title", "エラー");
        ja.insert("error-no-smtp-server", "SMTPサーバーアドレスを入力してください");
        ja.insert("error-no-sender", "送信者アドレスを入力してください");
        ja.insert("error-no-recipient", "受信者アドレスを入力してください");
        ja.insert("error-no-eml-dir", "EMLディレクトリを選択してください");
        ja.insert("error-no-attachment", "添付ファイルを選択してください");
        ja.insert("error-no-attachment-dir", "添付ディレクトリを選択してください");
        ja.insert("error-no-username", "認証にはユーザー名が必要です");
        ja.insert("error-no-password", "認証にはパスワードが必要です");
        map.insert(Language::Japanese, ja);

        map
    });

// ============ 公共 API ============

/// 获取翻译文本
pub fn t(key: &str) -> String {
    let lang = *CURRENT_LANGUAGE.read().unwrap();
    if let Some(translations) = TRANSLATIONS.get(&lang) {
        if let Some(text) = translations.get(key) {
            return (*text).to_string();
        }
    }
    // 回退到英文
    if let Some(translations) = TRANSLATIONS.get(&Language::English) {
        if let Some(text) = translations.get(key) {
            return (*text).to_string();
        }
    }
    key.to_string()
}

/// 设置当前语言
pub fn set_language(lang: Language) {
    *CURRENT_LANGUAGE.write().unwrap() = lang;
}

/// 获取当前语言
pub fn current_language() -> Language {
    *CURRENT_LANGUAGE.read().unwrap()
}

/// 获取支持的语言名称列表
pub fn language_names() -> Vec<String> {
    vec![
        Language::English.name().to_string(),
        Language::SimplifiedChinese.name().to_string(),
        Language::TraditionalChinese.name().to_string(),
        Language::Japanese.name().to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_detection() {
        let lang = Language::from_locale_string("zh_CN.UTF-8");
        assert_eq!(lang, Language::SimplifiedChinese);

        let lang = Language::from_locale_string("zh-TW");
        assert_eq!(lang, Language::TraditionalChinese);

        let lang = Language::from_locale_string("ja_JP");
        assert_eq!(lang, Language::Japanese);

        let lang = Language::from_locale_string("en_US");
        assert_eq!(lang, Language::English);
    }

    #[test]
    fn test_translation() {
        set_language(Language::English);
        assert_eq!(t("app-title"), "RSendMail");

        set_language(Language::SimplifiedChinese);
        assert_eq!(t("smtp-server"), "SMTP 服务器");
    }
}
