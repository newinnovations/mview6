use std::time::SystemTime;

pub struct Performance {
    start: SystemTime,
}

impl Performance {
    pub fn start() -> Self {
        Performance {
            start: SystemTime::now(),
        }
    }

    pub fn elapsed_suffix(&self, msg: &str, suffix: &str) {
        if let Ok(d) = self.start.elapsed() {
            let elapsed = d.as_secs() as f64 * 1e3 + d.subsec_nanos() as f64 * 1e-6;
            println!("{:>20} {:6.1} ms {}", msg, elapsed, suffix);
        };
    }

    pub fn elapsed(&self, msg: &str) {
        self.elapsed_suffix(msg, "");
    }
}
