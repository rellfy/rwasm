use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll, Waker},
    time::Duration,
    collections::HashMap,
    rc::Rc,
    cell::RefCell,
};

type Listeners = HashMap<u32, Box<dyn Fn() + 'static>>;

static mut LISTENERS: Option<Listeners> = None;

fn get_free_listener_key() -> u32 {
    let listeners = get_listeners();
    for i in 0..u32::max_value() {
        if listeners.contains_key(&i) {
            continue;
        }

        return i;
    }

    return 0;
}

fn get_listeners() -> &'static mut Listeners {
    unsafe {
        if LISTENERS.is_none() {
            LISTENERS = Some(HashMap::new());
        }

        LISTENERS.as_mut().unwrap()
    }
}

fn insert_listener(listener_id: u32, callback: Box<dyn Fn() + 'static>) {
    let listeners = get_listeners();
    listeners.insert(listener_id, callback);
}

#[no_mangle]
fn trigger_timeout(listener_id: u32) {
    let wake = get_listeners().get(&listener_id).unwrap();
    wake();
}

#[no_mangle]
extern "C" {
    pub fn request_timeout(listener_id: u32, millis: u32);
}

pub struct TimerFuture {
    state: Rc<RefCell<State>>,
}

struct State {
    completed: bool,
    waker: Option<Waker>,
}

impl Future for TimerFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.state.borrow_mut();

        if state.completed {
            Poll::Ready(())
        } else {
            state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

unsafe impl Send for TimerFuture { }

impl TimerFuture {
    pub fn new(duration: Duration) -> Self {
        let listener_id = get_free_listener_key();
        let timer = TimerFuture {
            state: Rc::new(RefCell::new(State {
                completed: false,
                waker: None
            })),
        };

        let listener_state = timer.state.clone();

        insert_listener(listener_id, Box::new(move || {
            let mut state = listener_state.borrow_mut();
            state.completed = true;

            if let Some(waker) = state.waker.take() {
                std::mem::drop(state);
                waker.wake();
            }
        }));

        unsafe {
            request_timeout(listener_id, duration.as_millis() as u32);
        }

        timer
    }
}
