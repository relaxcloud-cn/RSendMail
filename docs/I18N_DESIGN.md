# RSendMail GUI 国际化 (i18n) 设计方案

## 概述

为 RSendMail GUI 添加多语言支持，初期支持以下语言：
- 简体中文 (zh-CN) - 默认
- English (en-US)
- 繁体中文 (zh-TW)
- 日本語 (ja-JP)

## 技术方案

### 方案对比

| 方案 | 优点 | 缺点 |
|------|------|------|
| Slint 内置 @tr() | 原生支持，性能好 | 需要 .po 文件，工具链复杂 |
| Rust fluent-rs | 功能强大，Mozilla 标准 | 依赖较多，学习曲线 |
| 自定义 JSON/TOML | 简单直观，易于维护 | 需要自己实现逻辑 |
| rust-i18n | 宏支持，编译时检查 | 灵活性较低 |

**推荐方案**: 使用 **Slint 内置的 @tr() 宏** + **.po 文件**

理由：
1. Slint 原生支持，无需额外运行时开销
2. 使用标准的 gettext .po 格式，有成熟的翻译工具支持
3. 支持热切换语言（无需重启）

## 实现架构

```
crates/rsendmail-gui/
├── Cargo.toml
├── build.rs              # 编译 .po 文件
├── src/
│   ├── main.rs
│   └── i18n.rs          # 语言切换逻辑
├── ui/
│   └── app.slint        # 使用 @tr() 宏
└── i18n/
    ├── en-US.po         # 英文翻译
    ├── zh-CN.po         # 简体中文
    ├── zh-TW.po         # 繁体中文
    └── ja-JP.po         # 日文翻译
```

## Slint @tr() 宏使用

### 基本用法

```slint
// 简单文本
Text { text: @tr("SMTP Server"); }

// 带参数的文本
Text { text: @tr("Found {} files" => file_count); }

// 复数形式
Text { text: @tr("1 email" | "{} emails" % count => count); }
```

### UI 示例

```slint
// app.slint
export component AppWindow inherits Window {
    title: @tr("RSendMail");

    GroupBox {
        title: @tr("SMTP Server");

        HorizontalBox {
            Text { text: @tr("Server Address"); }
            LineEdit { placeholder-text: @tr("smtp.example.com"); }
        }

        HorizontalBox {
            Text { text: @tr("Port"); }
            // ...
        }

        CheckBox { text: @tr("Use TLS"); }
        CheckBox { text: @tr("Accept Invalid Certificates"); }
    }
}
```

## 翻译文件格式

### en-US.po (基准语言)

```po
# RSendMail GUI - English
msgid ""
msgstr ""
"Language: en-US\n"
"Content-Type: text/plain; charset=UTF-8\n"

msgid "RSendMail"
msgstr "RSendMail"

msgid "SMTP Server"
msgstr "SMTP Server"

msgid "Server Address"
msgstr "Server Address"

msgid "Port"
msgstr "Port"

msgid "Use TLS"
msgstr "Use TLS"

msgid "Accept Invalid Certificates"
msgstr "Accept Invalid Certificates"

msgid "Authentication Required"
msgstr "Authentication Required"

msgid "Username"
msgstr "Username"

msgid "Password"
msgstr "Password"

msgid "Sender"
msgstr "Sender"

msgid "Recipient"
msgstr "Recipient"

msgid "Send Mode"
msgstr "Send Mode"

msgid "EML Batch"
msgstr "EML Batch"

msgid "Single Attachment"
msgstr "Single Attachment"

msgid "Directory Attachment"
msgstr "Directory Attachment"

msgid "EML Directory"
msgstr "EML Directory"

msgid "Extension"
msgstr "Extension"

msgid "Browse..."
msgstr "Browse..."

msgid "Found {} files"
msgstr "Found {} files"

msgid "Advanced Options"
msgstr "Advanced Options"

msgid "Performance"
msgstr "Performance"

msgid "Processes"
msgstr "Processes"

msgid "Batch Size"
msgstr "Batch Size"

msgid "Send Interval (ms)"
msgstr "Send Interval (ms)"

msgid "Timeout (sec)"
msgstr "Timeout (sec)"

msgid "Loop Settings"
msgstr "Loop Settings"

msgid "Infinite Loop"
msgstr "Infinite Loop"

msgid "Repeat Count"
msgstr "Repeat Count"

msgid "Loop Interval (sec)"
msgstr "Loop Interval (sec)"

msgid "Retry Interval (sec)"
msgstr "Retry Interval (sec)"

msgid "Email Processing"
msgstr "Email Processing"

msgid "Keep Original Headers"
msgstr "Keep Original Headers"

msgid "Modify Headers"
msgstr "Modify Headers"

msgid "Anonymize Emails"
msgstr "Anonymize Emails"

msgid "Domain"
msgstr "Domain"

msgid "Logging"
msgstr "Logging"

msgid "Log Level"
msgstr "Log Level"

msgid "Log File"
msgstr "Log File"

msgid "Failed Emails Directory"
msgstr "Failed Emails Directory"

msgid "Statistics"
msgstr "Statistics"

msgid "Total"
msgstr "Total"

msgid "Success"
msgstr "Success"

msgid "Failed"
msgstr "Failed"

msgid "Current Round"
msgstr "Current Round"

msgid "Elapsed Time"
msgstr "Elapsed Time"

msgid "Send Log"
msgstr "Send Log"

msgid "Clear"
msgstr "Clear"

msgid "Export Log"
msgstr "Export Log"

msgid "Ready"
msgstr "Ready"

msgid "Preparing..."
msgstr "Preparing..."

msgid "Sending..."
msgstr "Sending..."

msgid "Stopped"
msgstr "Stopped"

msgid "Completed"
msgstr "Completed"

msgid "Save Config"
msgstr "Save Config"

msgid "Load Config"
msgstr "Load Config"

msgid "Test Connection"
msgstr "Test Connection"

msgid "Start Send"
msgstr "Start Send"

msgid "Stop Send"
msgstr "Stop Send"

# Error messages
msgid "Please enter SMTP server address"
msgstr "Please enter SMTP server address"

msgid "Please enter sender address"
msgstr "Please enter sender address"

msgid "Please enter recipient address"
msgstr "Please enter recipient address"

msgid "Please select EML directory"
msgstr "Please select EML directory"

msgid "Please select attachment file"
msgstr "Please select attachment file"

msgid "Please select attachment directory"
msgstr "Please select attachment directory"

msgid "Authentication requires username"
msgstr "Authentication requires username"

msgid "Authentication requires password"
msgstr "Authentication requires password"

# Log messages
msgid "Testing connection..."
msgstr "Testing connection..."

msgid "Connecting to {}:{} (TLS: {})"
msgstr "Connecting to {}:{} (TLS: {})"

msgid "Connection test successful"
msgstr "Connection test successful"

msgid "Connection test failed: {}"
msgstr "Connection test failed: {}"

msgid "Starting round {}/{}"
msgstr "Starting round {}/{}"

msgid "Round {} completed"
msgstr "Round {} completed"

msgid "Send completed! Success: {}, Failed: {}"
msgstr "Send completed! Success: {}, Failed: {}"

msgid "Waiting {} seconds before next round..."
msgstr "Waiting {} seconds before next round..."

msgid "Stopping..."
msgstr "Stopping..."

msgid "Config saved to: {}"
msgstr "Config saved to: {}"

msgid "Config loaded from: {}"
msgstr "Config loaded from: {}"

msgid "Log exported to: {}"
msgstr "Log exported to: {}"
```

### zh-CN.po (简体中文)

```po
# RSendMail GUI - Simplified Chinese
msgid ""
msgstr ""
"Language: zh-CN\n"
"Content-Type: text/plain; charset=UTF-8\n"

msgid "RSendMail"
msgstr "RSendMail"

msgid "SMTP Server"
msgstr "SMTP 服务器"

msgid "Server Address"
msgstr "服务器地址"

msgid "Port"
msgstr "端口"

msgid "Use TLS"
msgstr "使用 TLS"

msgid "Accept Invalid Certificates"
msgstr "接受自签名证书"

msgid "Authentication Required"
msgstr "需要认证"

msgid "Username"
msgstr "用户名"

msgid "Password"
msgstr "密码"

msgid "Sender"
msgstr "发件人"

msgid "Recipient"
msgstr "收件人"

msgid "Send Mode"
msgstr "发送模式"

msgid "EML Batch"
msgstr "EML 批量"

msgid "Single Attachment"
msgstr "单个附件"

msgid "Directory Attachment"
msgstr "目录附件"

msgid "EML Directory"
msgstr "EML 目录"

msgid "Extension"
msgstr "扩展名"

msgid "Browse..."
msgstr "浏览..."

msgid "Found {} files"
msgstr "找到 {} 个文件"

msgid "Advanced Options"
msgstr "高级选项"

msgid "Performance"
msgstr "性能配置"

msgid "Processes"
msgstr "进程数"

msgid "Batch Size"
msgstr "批量大小"

msgid "Send Interval (ms)"
msgstr "发送间隔(ms)"

msgid "Timeout (sec)"
msgstr "超时(秒)"

msgid "Loop Settings"
msgstr "循环发送"

msgid "Infinite Loop"
msgstr "无限循环"

msgid "Repeat Count"
msgstr "重复次数"

msgid "Loop Interval (sec)"
msgstr "循环间隔(秒)"

msgid "Retry Interval (sec)"
msgstr "重试间隔(秒)"

msgid "Email Processing"
msgstr "邮件处理"

msgid "Keep Original Headers"
msgstr "保留原始邮件头"

msgid "Modify Headers"
msgstr "修改邮件头"

msgid "Anonymize Emails"
msgstr "匿名化邮箱"

msgid "Domain"
msgstr "域名"

msgid "Logging"
msgstr "日志与错误处理"

msgid "Log Level"
msgstr "日志级别"

msgid "Log File"
msgstr "日志文件"

msgid "Failed Emails Directory"
msgstr "失败邮件"

msgid "Statistics"
msgstr "发送统计"

msgid "Total"
msgstr "总计"

msgid "Success"
msgstr "成功"

msgid "Failed"
msgstr "失败"

msgid "Current Round"
msgstr "当前轮次"

msgid "Elapsed Time"
msgstr "已用时间"

msgid "Send Log"
msgstr "发送日志"

msgid "Clear"
msgstr "清空"

msgid "Export Log"
msgstr "导出日志"

msgid "Ready"
msgstr "就绪"

msgid "Preparing..."
msgstr "准备中..."

msgid "Sending..."
msgstr "发送中..."

msgid "Stopped"
msgstr "已停止"

msgid "Completed"
msgstr "完成"

msgid "Save Config"
msgstr "保存配置"

msgid "Load Config"
msgstr "加载配置"

msgid "Test Connection"
msgstr "测试连接"

msgid "Start Send"
msgstr "开始发送"

msgid "Stop Send"
msgstr "停止发送"

msgid "Please enter SMTP server address"
msgstr "请输入 SMTP 服务器地址"

msgid "Please enter sender address"
msgstr "请输入发件人地址"

msgid "Please enter recipient address"
msgstr "请输入收件人地址"

msgid "Please select EML directory"
msgstr "请选择 EML 文件目录"

msgid "Please select attachment file"
msgstr "请选择附件文件"

msgid "Please select attachment directory"
msgstr "请选择附件目录"

msgid "Authentication requires username"
msgstr "认证模式需要输入用户名"

msgid "Authentication requires password"
msgstr "认证模式需要输入密码"

msgid "Testing connection..."
msgstr "开始测试连接..."

msgid "Connecting to {}:{} (TLS: {})"
msgstr "连接到 {}:{} (TLS: {})"

msgid "Connection test successful"
msgstr "连接测试成功"

msgid "Connection test failed: {}"
msgstr "连接测试失败: {}"

msgid "Starting round {}/{}"
msgstr "开始第 {}/{} 轮发送"

msgid "Round {} completed"
msgstr "第 {} 轮发送完成"

msgid "Send completed! Success: {}, Failed: {}"
msgstr "发送完成！成功: {}, 失败: {}"

msgid "Waiting {} seconds before next round..."
msgstr "等待 {} 秒后开始下一轮..."

msgid "Stopping..."
msgstr "正在停止发送..."

msgid "Config saved to: {}"
msgstr "配置已保存到: {}"

msgid "Config loaded from: {}"
msgstr "配置已加载: {}"

msgid "Log exported to: {}"
msgstr "日志已导出到: {}"
```

## 实现步骤

### 1. 更新 build.rs

```rust
fn main() {
    // 编译 Slint UI
    let config = slint_build::CompilerConfiguration::new()
        .with_style("fluent".into());

    slint_build::compile_with_config("ui/app.slint", config).unwrap();
}
```

### 2. 创建语言管理模块 (i18n.rs)

```rust
use slint::SharedString;
use std::sync::atomic::{AtomicUsize, Ordering};

// 支持的语言
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    SimplifiedChinese,
    TraditionalChinese,
    Japanese,
}

impl Language {
    pub fn code(&self) -> &'static str {
        match self {
            Language::English => "en-US",
            Language::SimplifiedChinese => "zh-CN",
            Language::TraditionalChinese => "zh-TW",
            Language::Japanese => "ja-JP",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::SimplifiedChinese => "简体中文",
            Language::TraditionalChinese => "繁體中文",
            Language::Japanese => "日本語",
        }
    }

    pub fn from_system() -> Self {
        // 检测系统语言
        if let Ok(lang) = std::env::var("LANG") {
            if lang.starts_with("zh_CN") || lang.starts_with("zh-CN") {
                return Language::SimplifiedChinese;
            }
            if lang.starts_with("zh_TW") || lang.starts_with("zh-TW") {
                return Language::TraditionalChinese;
            }
            if lang.starts_with("ja") {
                return Language::Japanese;
            }
        }

        // macOS 特殊处理
        #[cfg(target_os = "macos")]
        {
            if let Ok(output) = std::process::Command::new("defaults")
                .args(["read", "-g", "AppleLocale"])
                .output()
            {
                let locale = String::from_utf8_lossy(&output.stdout);
                if locale.starts_with("zh_CN") || locale.starts_with("zh-Hans") {
                    return Language::SimplifiedChinese;
                }
                if locale.starts_with("zh_TW") || locale.starts_with("zh-Hant") {
                    return Language::TraditionalChinese;
                }
                if locale.starts_with("ja") {
                    return Language::Japanese;
                }
            }
        }

        Language::English
    }
}

/// 设置 Slint 语言
pub fn set_language(lang: Language) {
    slint::set_locale(lang.code());
}

/// 获取所有支持的语言列表（用于 UI ComboBox）
pub fn supported_languages() -> Vec<SharedString> {
    vec![
        "English".into(),
        "简体中文".into(),
        "繁體中文".into(),
        "日本語".into(),
    ]
}

/// 根据索引获取语言
pub fn language_from_index(index: usize) -> Language {
    match index {
        0 => Language::English,
        1 => Language::SimplifiedChinese,
        2 => Language::TraditionalChinese,
        3 => Language::Japanese,
        _ => Language::English,
    }
}
```

### 3. 更新 main.rs

```rust
mod i18n;

fn main() -> Result<()> {
    // 初始化日志...

    // 检测并设置系统语言
    let system_lang = i18n::Language::from_system();
    i18n::set_language(system_lang);

    // 创建应用
    let app = AppWindow::new()?;

    // 设置语言列表
    let languages = i18n::supported_languages();
    app.set_available_languages(ModelRc::new(VecModel::from(languages)));

    // 设置当前语言索引
    let current_lang_index = match system_lang {
        i18n::Language::English => 0,
        i18n::Language::SimplifiedChinese => 1,
        i18n::Language::TraditionalChinese => 2,
        i18n::Language::Japanese => 3,
    };
    app.set_current_language_index(current_lang_index as i32);

    // 语言切换回调
    let app_weak = app.as_weak();
    app.on_language_changed(move |index| {
        let lang = i18n::language_from_index(index as usize);
        i18n::set_language(lang);

        // 刷新 UI
        if let Some(app) = app_weak.upgrade() {
            app.window().request_redraw();
        }
    });

    app.run()?;
    Ok(())
}
```

### 4. 更新 app.slint

```slint
// 添加语言相关属性
in-out property <[string]> available-languages: [];
in-out property <int> current-language-index: 0;
callback language-changed(int);

// 在设置区域添加语言选择
GroupBox {
    title: @tr("Settings");

    HorizontalBox {
        Text { text: @tr("Language"); }
        ComboBox {
            model: available-languages;
            current-index <=> current-language-index;
            selected(index) => { language-changed(index); }
        }
    }
}

// 所有文本改用 @tr() 宏
GroupBox {
    title: @tr("SMTP Server");

    HorizontalBox {
        Text { text: @tr("Server Address"); }
        // ...
    }
}
```

## 字体处理

为确保各语言正确显示，需要配置跨平台字体回退：

```slint
export component AppWindow inherits Window {
    // 字体回退链：优先使用系统 UI 字体，然后是各平台中文字体
    default-font-family: "system-ui, -apple-system, PingFang SC, Microsoft YaHei, Hiragino Sans, Meiryo, sans-serif";
}
```

或者在 Rust 中动态设置：

```rust
fn get_default_font() -> &'static str {
    #[cfg(target_os = "macos")]
    { "PingFang SC" }

    #[cfg(target_os = "windows")]
    { "Microsoft YaHei" }

    #[cfg(target_os = "linux")]
    { "Noto Sans CJK SC" }
}
```

## 翻译工作流

### 开发者工作流

1. 在 .slint 文件中使用 `@tr("key")` 标记所有文本
2. 运行提取脚本生成 .pot 模板文件
3. 使用 .pot 更新各语言 .po 文件

### 翻译者工作流

1. 使用 Poedit 或其他 .po 编辑工具打开 .po 文件
2. 翻译所有 msgstr 为空的条目
3. 保存并提交

### 自动化脚本

```bash
#!/bin/bash
# scripts/extract-translations.sh

# 从 .slint 文件提取翻译字符串
slint-tr-extractor ui/app.slint > i18n/messages.pot

# 更新各语言文件
for lang in en-US zh-CN zh-TW ja-JP; do
    msgmerge -U i18n/$lang.po i18n/messages.pot
done
```

## 测试计划

1. **单元测试**: 验证语言切换逻辑
2. **UI 测试**: 验证各语言下 UI 布局不溢出
3. **集成测试**: 验证翻译完整性（无遗漏 msgstr）

```rust
#[test]
fn test_all_translations_complete() {
    for lang in ["en-US", "zh-CN", "zh-TW", "ja-JP"] {
        let po_content = std::fs::read_to_string(format!("i18n/{}.po", lang)).unwrap();
        assert!(!po_content.contains("msgstr \"\""),
            "Language {} has missing translations", lang);
    }
}
```

## 实施路线图

### Phase 1: 基础架构 (当前)
- [x] 设计文档
- [ ] 创建 i18n 模块
- [ ] 更新 build.rs

### Phase 2: 翻译文件
- [ ] 创建 en-US.po (基准)
- [ ] 创建 zh-CN.po
- [ ] 创建 zh-TW.po
- [ ] 创建 ja-JP.po

### Phase 3: UI 改造
- [ ] 替换所有硬编码文本为 @tr()
- [ ] 添加语言选择 UI
- [ ] 测试语言切换

### Phase 4: 完善
- [ ] 添加更多语言支持
- [ ] 优化字体显示
- [ ] 文档更新

## 参考资源

- [Slint Translations Documentation](https://slint.dev/releases/1.9/docs/slint/src/language/translation)
- [GNU gettext Manual](https://www.gnu.org/software/gettext/manual/)
- [Poedit Translation Editor](https://poedit.net/)
