use tokio::{
    sync::oneshot,
    time::{sleep, Duration},
};

pub struct Delayed {
    cancel_tx: Option<oneshot::Sender<()>>,
    handle: Option<tokio::task::JoinHandle<()>>,
}

impl Delayed {
    pub fn new<F, Fut, R>(wait: Duration, f: F) -> Self
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = R> + Send + 'static,
    {
        let (cancel_tx, cancel_rx) = oneshot::channel::<()>();
        let handle = tokio::spawn(async move {
            tokio::select! {
               _ = sleep(wait) => {
                    println!("Executing delayed task.");
                   f().await;
               }
               _ = cancel_rx => {
                   println!("Task was cancelled before execution.");
               }
            }
        });
        Self {
            cancel_tx: Some(cancel_tx),
            handle: Some(handle),
        }
    }

    pub fn cancel(&mut self) {
        if let Some(canceller) = self.cancel_tx.take() {
            if canceller.is_closed() {
                return;
            }
            println!("Cancelling the task.");
            canceller.send(()).expect("error cancelling the task");
        }
    }

    // fixme: remove it
    #[allow(dead_code)]
    async fn wait(&mut self) {
        let handle = self.handle.take();
        if let Some(handle) = handle {
            handle.await.expect("error running the task");
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use tokio::time::Duration;

    use crate::delayed::Delayed;

    #[tokio::test]
    async fn should_not_be_cancelled() {
        let boolean = Arc::new(Mutex::new(false));
        let bclone = boolean.clone();
        let mut delayed = Delayed::new(Duration::from_secs(1), move || async move {
            println!("Executing the task.");
            let mut boolean = bclone.lock().await;
            *boolean = true;
        });

        delayed.wait().await;

        let final_value = *boolean.lock().await;
        assert!(final_value, "should not be cancelled.");
    }

    #[tokio::test]
    async fn should_be_cancelled() {
        let boolean = Arc::new(Mutex::new(false));
        let bclone = boolean.clone();
        let mut delayed = Delayed::new(Duration::from_secs(1), move || async move {
            let mut boolean = bclone.lock().await;
            *boolean = true;
        });

        delayed.cancel();
        delayed.wait().await;

        let final_value = *boolean.lock().await;
        assert!(!final_value, "should be cancelled");
    }

    #[tokio::test]
    async fn runs_until_handle_isnt_dropped() {
        let boolean: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
        let bclone = boolean.clone();
        let mut delayed = Delayed::new(Duration::from_secs(1), move || async move {
            let mut boolean = bclone.lock().await;
            *boolean = true;
        });

        tokio::spawn(async move {
            delayed.wait().await;
        });
        // wait for it in the background. The value should have been modified already
        tokio::time::sleep(Duration::from_secs(2)).await;

        let final_value = *boolean.lock().await;
        assert!(final_value, "should be executed");
    }
}
