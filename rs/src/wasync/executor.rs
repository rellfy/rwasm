use {
    std::{
        future::Future,
        pin::Pin,
        sync::{
            Arc,
            Mutex,
        },
        task::{
            Context,
            Poll,
        },
    },
    once_cell::sync::OnceCell,
    super::waker::{
        waker_ref,
        Waker,
    },
};

fn get_executor() -> &'static Mutex<Executor> {
    static INSTANCE: OnceCell<Mutex<Executor>> = OnceCell::new();
    INSTANCE.get_or_init(|| Mutex::new(Executor { task: None }))
}

/// Executor storing one task.
pub struct Executor {
    task: Option<Arc<Task>>,
}

/// Task storing one Future to be polled by an Executor.
struct Task {
    pub future: Mutex<Option<Pin<Box<dyn Future<Output = ()> + Send + 'static>>>>,
}

impl Waker for Task {
    /// Updates executor on awake.
    fn wake_by_ref(_: &Arc<Self>) {
        Executor::run()
    }
}

impl Executor {
    pub fn spawn(future: impl Future<Output = ()> + 'static + Send) {
        // Store task in global state.
        let task = Arc::new(Task {
            future: Mutex::new(Some(Box::pin(future))),
        });
        let mut executor = get_executor().lock().unwrap();
        executor.task = Some(task);

        // Drop reference to unlock mutex.
        std::mem::drop(executor);
        Executor::run();
    }

    fn run() {
        let executor = get_executor().lock().unwrap();

        if executor.task.is_none() {
            return;
        }

        let task = executor.task.as_ref().unwrap();

        let mut future_slot = task.future.lock().unwrap();
        if let Some(mut future) = future_slot.take() {
            // Get waker from task.
            let waker = waker_ref(&task);
            // Poll the future.
            let context = &mut Context::from_waker(&*waker);
            if let Poll::Pending = future.as_mut().poll(context) {
                *future_slot = Some(future);
            }
        }
    }
}