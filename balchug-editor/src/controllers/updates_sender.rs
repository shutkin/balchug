use std::cell::Cell;
use std::pin::Pin;
use dioxus::prelude::*;
use gloo_timers::future::TimeoutFuture;
use std::rc::Rc;

#[derive(Clone)]
pub struct UpdatesSender<T> {
    handler: Rc<Box<dyn UpdatesHandler<T>>>,
}

impl<T: 'static> UpdatesSender<T> {
    pub fn new(handler: impl UpdatesHandler<T> + 'static) -> Self {
        Self {
            handler: Rc::new(Box::new(handler)),
        }
    }

    pub fn start(&self) -> Rc<Cell<bool>> {
        let update_signal = Rc::new(Cell::new(false));
        let update_signal_clone = update_signal.clone();
        let handler = self.handler.clone();
        use_future(move || {
            let handler = handler.clone();
            let update_signal = update_signal_clone.clone();
            async move {
                loop {
                    TimeoutFuture::new(3000).await;
                    if update_signal.get() {
                        if let Some(value) = handler.collect() {
                            handler.send(value).await;
                        }
                        update_signal.set(false);
                    }
                }
            }
        });
        update_signal
    }
}

pub type PinnedFuture<'a> = Pin<Box<dyn Future<Output = ()> + 'a>>;

pub trait UpdatesHandler<T> {
    fn collect(&self) -> Option<T>;
    fn send(&self, value: T) -> PinnedFuture<'_>;
}
