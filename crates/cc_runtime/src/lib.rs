use std::{collections::HashMap, future::Future};

use once_cell::sync::OnceCell;
pub use tokio;
use tokio::sync::oneshot::{self, Receiver};

static HANDLE: OnceCell<tokio::runtime::Handle> = OnceCell::new();

pub fn start() -> impl FnOnce() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("build tokio runtime");

    let handle = rt.handle().clone();
    HANDLE.set(handle).expect("single initialization");

    let (tx, rx) = oneshot::channel::<()>();
    let thread = std::thread::spawn(move || {
        rt.block_on({
            async move {
                tokio::select! {
                    _ = std::future::pending::<()>() => {}
                    _ = rx => {}
                }
            }
        });
    });

    move || {
        tracing::debug!("Shutdown runtime");
        drop(tx);
        let _ = thread.join();
    }
}

pub fn enter<T>(func: impl FnOnce() -> T) -> T {
    let _g = HANDLE.get().expect("runtime initialization").enter();
    func()
}

pub fn spawn<T>(fut: impl Future<Output = T> + Send + Sync + 'static) -> Receiver<T>
where
    T: Send + Sync + 'static,
{
    let (tx, rx) = oneshot::channel();
    enter(|| {
        tokio::task::spawn(async move {
            let res = fut.await;
            let _ = tx.send(res);
        });
    });
    rx
}

pub fn blocking<T>(func: impl FnOnce() -> T + Send + Sync + 'static) -> Receiver<T>
where
    T: Send + Sync + 'static,
{
    let (tx, rx) = oneshot::channel();
    enter(|| {
        tokio::task::spawn(async move {
            if let Ok(ok) = tokio::task::spawn_blocking(func).await {
                let _ = tx.send(ok);
            }
        });
    });
    rx
}

type ResolverItem = Box<dyn std::any::Any + Send + Sync>;
type PendingItem = Box<dyn FnMut() -> Option<(uuid::Uuid, ResolverItem)>>;

#[derive(Default)]
pub struct Resolver {
    pending: Vec<PendingItem>,
    resolved: HashMap<uuid::Uuid, ResolverItem>,
}

impl Resolver {
    pub fn spawn<T>(&mut self, fut: impl Future<Output = T> + Send + Sync + 'static) -> uuid::Uuid
    where
        T: Send + Sync + 'static,
    {
        let (tx, rx) = oneshot::channel();
        enter(|| {
            tokio::task::spawn(async move {
                let res = fut.await;
                let _ = tx.send(res);
            });
        });
        self.add(rx)
    }

    fn add<T>(&mut self, mut item: Receiver<T>) -> uuid::Uuid
    where
        T: Send + Sync + 'static + std::any::Any,
    {
        let uuid = uuid::Uuid::new_v4();

        self.pending.push(Box::new(move || {
            let out = item.try_recv().ok()?;
            Some((uuid, Box::new(out)))
        }));

        uuid
    }

    pub fn poll(&mut self) {
        let mut temp = vec![];
        for mut item in std::mem::take(&mut self.pending) {
            if let Some((uuid, item)) = item() {
                self.resolved.insert(uuid, item);
                continue;
            }
            temp.push(item);
        }
        self.pending = temp;
    }

    pub fn try_take<T>(&mut self, id: uuid::Uuid) -> Option<T>
    where
        T: Send + Sync + 'static + std::any::Any,
    {
        match self.resolved.remove(&id)?.downcast::<T>() {
            Ok(item) => Some(*item),
            Err(..) => panic!("invalid type"),
        }
    }
}
