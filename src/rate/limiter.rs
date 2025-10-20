use std::time::{Duration, Instant};
use tokio::time::sleep;

#[derive(Clone)]
pub struct RateLimiter {
    bytes_per_second: u64,
    last_check: Instant,
    bytes_consumed: u64,
}

impl RateLimiter {
    pub fn new(rate_str: &str) -> Result<Self, String> {
        let bytes_per_second = Self::parse_rate(rate_str)?;
        Ok(Self {
            bytes_per_second,
            last_check: Instant::now(),
            bytes_consumed: 0,
        })
    }

    pub async fn consume(&mut self, bytes: u64) {
        if self.bytes_per_second == 0 {
            return;
        }

        self.bytes_consumed += bytes;
        let elapsed = self.last_check.elapsed();
        let allowed_bytes = (elapsed.as_secs_f64() * self.bytes_per_second as f64) as u64;

        if self.bytes_consumed > allowed_bytes {
            let excess = self.bytes_consumed - allowed_bytes;
            let delay_secs = excess as f64 / self.bytes_per_second as f64;
            sleep(Duration::from_secs_f64(delay_secs)).await;
        }

        // Reset counters periodically
        if elapsed >= Duration::from_secs(1) {
            self.last_check = Instant::now();
            self.bytes_consumed = 0;
        }
    }

    fn parse_rate(rate_str: &str) -> Result<u64, String> {
        if rate_str.ends_with('k') || rate_str.ends_with('K') {
            let num_str = &rate_str[..rate_str.len() - 1];
            let num: u64 = num_str.parse().map_err(|_| "Invalid rate format".to_string())?;
            Ok(num * 1024)
        } else if rate_str.ends_with('M') || rate_str.ends_with('m') {
            let num_str = &rate_str[..rate_str.len() - 1];
            let num: u64 = num_str.parse().map_err(|_| "Invalid rate format".to_string())?;
            Ok(num * 1024 * 1024)
        } else {
            rate_str.parse().map_err(|_| "Invalid rate format".to_string())
        }
    }
}