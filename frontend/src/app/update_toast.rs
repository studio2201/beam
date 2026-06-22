use yew::prelude::*;

use crate::app::App;
use crate::types::Msg;

impl App {
    pub fn update_toast(&mut self, ctx: &Context<Self>, msg: Msg) -> bool {
        match msg {
            Msg::AddToast(message, toast_type) => {
                if let Some(t) = self.active_timeout.take() {
                    t.cancel();
                }

                self.active_notification = Some((message, toast_type));

                let link = ctx.link().clone();
                let timeout = gloo_timers::callback::Timeout::new(3000, move || {
                    link.send_message(Msg::RemoveToast(0));
                });

                self.active_timeout = Some(timeout);
                true
            }

            Msg::RemoveToast(_) => {
                self.active_notification = None;
                self.active_timeout = None;
                true
            }
            _ => false,
        }
    }
}
