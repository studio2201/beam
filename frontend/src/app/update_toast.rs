use yew::prelude::*;

use crate::app::App;
use crate::types::{Msg, Toast};

impl App {
    pub fn update_toast(&mut self, ctx: &Context<Self>, msg: Msg) -> bool {
        match msg {
            Msg::AddToast(message, toast_type) => {
                let id = self.next_toast_id;
                self.next_toast_id += 1;
                
                self.toasts.push(Toast {
                    id,
                    message,
                    toast_type,
                });
                
                let link = ctx.link().clone();
                let timeout = gloo_timers::callback::Timeout::new(3000, move || {
                    link.send_message(Msg::RemoveToast(id));
                });
                
                self.toast_timeouts.insert(id, timeout);
                true
            }
            
            Msg::RemoveToast(id) => {
                self.toasts.retain(|t| t.id != id);
                self.toast_timeouts.remove(&id);
                true
            }
            _ => false,
        }
    }
}
