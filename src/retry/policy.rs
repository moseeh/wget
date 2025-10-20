use std::time::Duration;
use tokio::time::sleep;

#[derive(Clone)]
pub struct RetryPolicy {
    max_tries: u32,
    wait_retry: Duration,
}

impl RetryPolicy {
    pub fn new(max_tries: u32, wait_retry_secs: u64) -> Self {
        Self {
            max_tries,
            wait_retry: Duration::from_secs(wait_retry_secs),
        }
    }

    pub fn default() -> Self {
        Self::new(3, 1)
    }

    pub async fn execute<F, T, E>(&self, mut operation: F) -> Result<T, E>
    where
        F: FnMut() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>,
    {
        let mut last_error = None;
        
        for attempt in 1..=self.max_tries {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < self.max_tries {
                        sleep(self.wait_retry * attempt).await;
                    }
                }
            }
        }
        
        Err(last_error.unwrap())
    }
}