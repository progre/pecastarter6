use std::sync::{Arc, Weak};

use crate::features::bbs::BbsListenerDelegate;

use super::{app::App, entities::contact_status::ContactStatus};

pub struct AppBbsListenerDelegate(Weak<App>);

impl AppBbsListenerDelegate {
    pub fn new(app: Weak<App>) -> Self {
        Self(app)
    }

    fn app(&self) -> Arc<App> {
        self.0.upgrade().unwrap()
    }
}

impl BbsListenerDelegate for AppBbsListenerDelegate {
    fn on_update_contact_status(&self, contact_status: &ContactStatus) {
        self.app()
            .ui
            .lock()
            .unwrap()
            .push_contact_status(contact_status);
    }

    fn on_update_contact_url(&self, url: String) {
        todo!()
    }
}
