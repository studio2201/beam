use yew::prelude::*;

use crate::app::App;
use crate::types::Msg;
use crate::api::{verify_pin_api, logout_api};

impl App {
    pub fn update_pin(&mut self, ctx: &Context<Self>, msg: Msg) -> bool {
        match msg {
            Msg::PinInputChanged(val) => {
                if self.is_lockout {
                    return false;
                }
                
                let filtered: String = val.chars().filter(|c| c.is_ascii_digit()).collect();
                let max_len = self.config.as_ref().map(|c| c.pin_length).unwrap_or(4);
                
                if filtered.len() <= max_len {
                    self.pin_input = filtered.clone();
                    if let Some(input) = self.pin_ref.cast::<web_sys::HtmlInputElement>() {
                        input.set_value(&self.pin_input);
                    }
                    
                    if self.pin_input.len() == max_len {
                        ctx.link().send_message(Msg::VerifyPin);
                    }
                }
                true
            }
            
            Msg::VerifyPin => {
                let pin = self.pin_input.clone();
                let expected_len = self.config.as_ref().map(|c| c.pin_length).unwrap_or(4);
                if pin.len() < expected_len {
                    return false;
                }
                
                let link = ctx.link().clone();
                wasm_bindgen_futures::spawn_local(async move {
                    match verify_pin_api(&pin).await {
                        Ok(success) => {
                            if success {
                                link.send_message(Msg::PinVerificationResult(Ok(true)));
                            } else {
                                link.send_message(Msg::PinVerificationResult(Err("Invalid PIN.".to_string())));
                            }
                        }
                        Err(e) => {
                            link.send_message(Msg::PinVerificationResult(Err(e)));
                        }
                    }
                });
                false
            }
            
            Msg::PinVerificationResult(res) => {
                match res {
                    Ok(true) => {
                        self.is_authenticated = true;
                        self.error_message = None;
                        self.is_lockout = false;
                        self.show_toast(ctx, "Authentication successful", "success");
                        ctx.link().send_message(Msg::RefreshFiles);
                    }
                    Ok(false) => {
                        self.error_message = Some("Invalid PIN".to_string());
                        self.reset_pin_inputs();
                    }
                    Err(e) => {
                        if !e.is_empty() {
                            self.error_message = Some(e.clone());
                            if e.contains("Too many") || e.contains("locked") {
                                self.is_lockout = true;
                            } else {
                                self.reset_pin_inputs();
                            }
                        }
                    }
                }
                true
            }
            
            Msg::Logout => {
                let link = ctx.link().clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let _ = logout_api().await;
                    link.send_message(Msg::RefreshFiles);
                });
                self.is_authenticated = false;
                self.reset_pin_inputs();
                true
            }
            _ => false,
        }
    }
}
