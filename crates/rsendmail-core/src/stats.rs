use rsendmail_i18n::{tr, tr_with_args};
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
            .or_default()
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
        writeln!(f, "{}", tr("core.stats.report_title"))?;
        writeln!(f, "{}", tr("core.stats.separator"))?;
        writeln!(f, "{}", tr("core.stats.basic_stats"))?;
        writeln!(
            f,
            "{}",
            tr_with_args("core.stats.total_processed", &[("count", &self.email_count.to_string())])
        )?;
        writeln!(
            f,
            "{}",
            tr_with_args(
                "core.stats.success_sent",
                &[("count", &(self.email_count - self.send_errors - self.parse_errors).to_string())]
            )
        )?;
        writeln!(
            f,
            "{}",
            tr_with_args(
                "core.stats.total_failed",
                &[("count", &(self.send_errors + self.parse_errors).to_string())]
            )
        )?;

        if !self.error_details.is_empty() {
            writeln!(f, "\n{}", tr("core.stats.error_classification"))?;
            let mut sorted_errors: Vec<_> = self.error_details.iter().collect();
            sorted_errors.sort_by(|a, b| b.1.cmp(a.1));

            for (error_type, count) in sorted_errors {
                let percent = if self.email_count > 0 {
                    (*count as f64 / self.email_count as f64) * 100.0
                } else {
                    0.0
                };
                writeln!(
                    f,
                    "{}",
                    tr_with_args(
                        "core.stats.error_type_count",
                        &[
                            ("type", error_type),
                            ("count", &count.to_string()),
                            ("percent", &format!("{:.1}", percent))
                        ]
                    )
                )?;
                if let Some(files) = self.failed_files.get(error_type) {
                    writeln!(f, "{}", tr("core.stats.failed_files_list"))?;
                    for file in files {
                        writeln!(
                            f,
                            "{}",
                            tr_with_args("core.stats.failed_file_item", &[("file", file.as_str())])
                        )?;
                    }
                }
            }
        }

        // Calculate total parse and send duration
        let total_parse_duration: Duration = self.parse_durations.iter().sum();
        let total_send_duration: Duration = self.send_durations.iter().sum();

        // Calculate parse QPS
        let parse_qps = self.calculate_qps(self.email_count, total_parse_duration);
        writeln!(
            f,
            "{}",
            tr_with_args(
                "core.stats.parse_duration",
                &[
                    ("seconds", &format!("{:.2}", total_parse_duration.as_secs_f64())),
                    ("qps", &format!("{:.2}", parse_qps))
                ]
            )
        )?;

        // Calculate send QPS
        let send_qps = self.calculate_qps(self.email_count, total_send_duration);
        writeln!(
            f,
            "{}",
            tr_with_args(
                "core.stats.send_duration",
                &[
                    ("seconds", &format!("{:.2}", total_send_duration.as_secs_f64())),
                    ("qps", &format!("{:.2}", send_qps))
                ]
            )
        )?;

        // Calculate actual total time
        let total_secs = self.total_duration.as_secs_f64();
        let actual_qps = self.calculate_qps(self.email_count, self.total_duration);
        writeln!(
            f,
            "{}",
            tr_with_args(
                "core.stats.actual_duration",
                &[
                    ("seconds", &format!("{:.2}", total_secs)),
                    ("qps", &format!("{:.2}", actual_qps))
                ]
            )
        )?;

        Ok(())
    }
}
