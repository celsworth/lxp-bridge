use crate::prelude::*;

#[derive(Default, Clone)]
pub struct Notify {
    /* wrap everything in a single Rc, avoids having an Rc for each member */
    inner: Rc<Inner>,
}

#[derive(Default)]
struct Inner {
    mqtt_sender_ready: RefCell<bool>,
    mqtt_sender_notify: tokio::sync::Notify,
}

impl Notify {
    pub fn new() -> Self {
        Self {
            inner: Rc::new(Inner::default()),
        }
    }

    pub fn mqtt_sender_is_ready(&self) {
        *self.inner.mqtt_sender_ready.borrow_mut() = true;

        self.inner.mqtt_sender_notify.notify_waiters();
    }

    pub async fn wait_for_mqtt_sender(&self) {
        if *self.inner.mqtt_sender_ready.borrow() {
            return;
        }

        self.inner.mqtt_sender_notify.notified().await;
    }
}
