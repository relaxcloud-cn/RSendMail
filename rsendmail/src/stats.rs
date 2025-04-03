use std::collections::HashMap;
use std::fmt;
use std::time::Duration;

#[derive(Default)]
pub struct Stats {
    pub email_count: usize,
    pub parse_durations: Vec<Duration>,
    pub send_durations: Vec<Duration>,
    pub total_duration: Duration,
    pub parse_errors: usize,
    pub send_errors: usize,
    pub error_details: HashMap<String, usize>,
    pub failed_files: HashMap<String, Vec<String>>,
}

impl Stats {
    pub fn new() -> Self {
        Stats {
            email_count: 0,
            parse_durations: Vec::new(),
            send_durations: Vec::new(),
            total_duration: Duration::from_secs(0),
            parse_errors: 0,
            send_errors: 0,
            error_details: HashMap::new(),
            failed_files: HashMap::new(),
        }
    }

    pub fn increment_error(&mut self, error_type: &str, file_path: &str) {
        *self
            .error_details
            .entry(error_type.to_string())
            .or_insert(0) += 1;
        self.failed_files
            .entry(error_type.to_string())
            .or_insert_with(Vec::new)
            .push(file_path.to_string());
        self.send_errors += 1;
    }

    fn calculate_qps(&self, count: usize, duration: Duration) -> f64 {
        if duration.as_secs_f64() > 0.0 {
            count as f64 / duration.as_secs_f64()
        } else {
            0.0
        }
    }
}

impl fmt::Display for Stats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "邮件发送统计报告")?;
        writeln!(f, "===================")?;
        writeln!(f, "1. 基本统计")?;
        writeln!(f, "    总计处理: {} 封邮件", self.email_count)?;
        writeln!(
            f,
            "    成功发送: {} 封",
            self.email_count - self.send_errors - self.parse_errors
        )?;
        writeln!(
            f,
            "    总计失败: {} 封",
            self.send_errors + self.parse_errors
        )?;

        if !self.error_details.is_empty() {
            writeln!(f, "\n2. 错误分类统计")?;
            let mut sorted_errors: Vec<_> = self.error_details.iter().collect();
            sorted_errors.sort_by(|a, b| b.1.cmp(a.1));

            for (error_type, count) in sorted_errors {
                writeln!(
                    f,
                    "    {} - {} 封 ({:.1}%)",
                    error_type,
                    count,
                    (*count as f64 / self.email_count as f64) * 100.0
                )?;
                if let Some(files) = self.failed_files.get(error_type) {
                    writeln!(f, "    失败文件列表:")?;
                    for file in files {
                        writeln!(f, "        - {}", file)?;
                    }
                }
            }
        }

        // 计算总的解析和发送时间
        let total_parse_duration: Duration = self.parse_durations.iter().sum();
        let total_send_duration: Duration = self.send_durations.iter().sum();

        // 计算解析QPS
        let parse_qps = self.calculate_qps(self.email_count, total_parse_duration);
        writeln!(
            f,
            "    邮件解析总耗时: {:.2}秒（所有进程总和），QPS: {:.2}封/秒",
            total_parse_duration.as_secs_f64(),
            parse_qps
        )?;

        // 计算发送QPS
        let send_qps = self.calculate_qps(self.email_count, total_send_duration);
        writeln!(
            f,
            "    邮件发送总耗时: {:.2}秒（所有进程总和），QPS: {:.2}封/秒",
            total_send_duration.as_secs_f64(),
            send_qps
        )?;

        // 计算实际总用时
        let total_secs = self.total_duration.as_secs_f64();
        let actual_qps = self.calculate_qps(self.email_count, self.total_duration);
        writeln!(
            f,
            "    实际总用时: {:.2}秒, QPS: {:.2}封/秒",
            total_secs, actual_qps
        )?;

        Ok(())
    }
}
