use std::future::Future;

static TOKIO_HANDLE: once_cell::sync::OnceCell<tokio::runtime::Handle> =
    once_cell::sync::OnceCell::new();

pub fn start() -> std::io::Result<()> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    let handle = rt.handle().clone();
    let _thread = std::thread::spawn(move || {
        rt.block_on(std::future::pending::<()>());
    });

    TOKIO_HANDLE.get_or_init(|| handle);

    Ok(())
}

pub fn enter_context(f: impl FnOnce()) {
    let _g = TOKIO_HANDLE.get().expect("initialization").enter();
    f();
}

pub fn spawn<T>(fut: impl Future<Output = T> + Send + Sync + 'static) -> flume::Receiver<T>
where
    T: Send + Sync + 'static,
{
    let (tx, rx) = flume::bounded(1); // not-sync
    enter_context(|| {
        tokio::task::spawn(async move {
            let res = fut.await;
            let _ = tx.send(res);
        });
    });
    rx
}
