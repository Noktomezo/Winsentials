use std::future::Future;
use std::sync::LazyLock;
use tokio::runtime::Runtime;

pub static TOKIO: LazyLock<Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .thread_name("winsentials-tokio")
        .build()
        .expect("Failed to initialize background Tokio runtime")
});

pub fn spawn_tokio<F>(future: F) -> tokio::task::JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    TOKIO.spawn(future)
}
