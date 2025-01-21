use std::time::Duration;

#[derive(Default)]
pub struct Stats {
    pub email_count: usize,
    pub parse_durations: Vec<Duration>,
    pub send_durations: Vec<Duration>,
    pub total_duration: Duration,
    pub parse_errors: usize,
    pub send_errors: usize,
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
        }
    }

    pub fn add_parse_duration(&mut self, duration: Duration) {
        self.parse_durations.push(duration);
    }

    pub fn add_send_duration(&mut self, duration: Duration) {
        self.send_durations.push(duration);
    }

    pub fn set_total_duration(&mut self, duration: Duration) {
        self.total_duration = duration;
    }

    pub fn increment_count(&mut self) {
        self.email_count += 1;
    }

    pub fn increment_parse_error(&mut self) {
        self.parse_errors += 1;
    }

    pub fn increment_send_error(&mut self) {
        self.send_errors += 1;
    }

    pub fn average_parse_duration(&self) -> Duration {
        if self.parse_durations.is_empty() {
            Duration::default()
        } else {
            let total: Duration = self.parse_durations.iter().sum();
            total / self.parse_durations.len() as u32
        }
    }

    pub fn average_send_duration(&self) -> Duration {
        if self.send_durations.is_empty() {
            Duration::default()
        } else {
            let total: Duration = self.send_durations.iter().sum();
            total / self.send_durations.len() as u32
        }
    }

    pub fn total_parse_duration(&self) -> Duration {
        self.parse_durations.iter().sum()
    }

    pub fn total_send_duration(&self) -> Duration {
        self.send_durations.iter().sum()
    }
}
