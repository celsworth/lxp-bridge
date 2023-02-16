use crate::prelude::*;

#[derive(Default, Clone)]
pub struct Notify {
    /* wrap everything in a single Rc, avoids having an Rc for each subsystem */
    inner: Rc<Inner>,
}

#[derive(Default)]
struct Inner {
    mqtt_sender: Subsystem,
}

#[derive(Default)]
struct Subsystem {
    ready: RefCell<bool>,
    notify: tokio::sync::Notify,
}

impl Notify {
    pub fn new() -> Self {
        Self {
            inner: Rc::new(Inner::default()),
        }
    }

    pub fn mqtt_sender_is_ready(&self) {
        Self::is_ready(&self.inner.mqtt_sender);
    }

    pub async fn wait_for_mqtt_sender(&self) {
        Self::wait(&self.inner.mqtt_sender).await;
    }

    fn is_ready(subsystem: &Subsystem) {
        *subsystem.ready.borrow_mut() = true;
        subsystem.notify.notify_waiters();
    }

    async fn wait(subsystem: &Subsystem) {
        if *subsystem.ready.borrow() {
            return;
        }

        subsystem.notify.notified().await;
    }
}
