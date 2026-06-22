use yew::prelude::*;

use crate::app::App;
use crate::types::Msg;
use crate::api::{verify_pin_api, logout_api};

impl App {
    pub fn update_pin(&mut self, ctx: &Context<Self>, msg: Msg) -> bool {
        match msg {
            Msg::PinDigitInput(idx, val) => {
                if self.is_lockout {
                    return false;
                }
                
                let filtered: String = val.chars().filter(|c| c.is_ascii_digit()).collect();
                
                if !filtered.is_empty() {
                    // Update input value
                    let single_char = filtered.chars().next().unwrap().to_string();
                    self.pin_digits[idx] = single_char.clone();
                    
                    if let Some(input) = self.pin_refs[idx].cast::<web_sys::HtmlInputElement>() {
                        input.set_value(&single_char);
                    }
                    
                    // Move focus
                    if idx < self.pin_digits.len() - 1 {
                        if let Some(next_input) = self.pin_refs[idx + 1].cast::<web_sys::HtmlInputElement>() {
                            let _ = next_input.focus();
                        }
                    } else {
                        // Submit on last digit filled
                        if self.pin_digits.iter().all(|d| !d.is_empty()) {
                            ctx.link().send_message(Msg::VerifyPin);
                        }
                    }
                } else {
                    self.pin_digits[idx] = "".to_string();
                    if let Some(input) = self.pin_refs[idx].cast::<web_sys::HtmlInputElement>() {
                        input.set_value("");
                    }
                }
                true
            }
            
            Msg::PinBackspace(idx) => {
                if self.is_lockout {
                    return false;
                }
                
                if self.pin_digits[idx].is_empty() && idx > 0 {
                    self.pin_digits[idx - 1] = "".to_string();
                    if let Some(prev_input) = self.pin_refs[idx - 1].cast::<web_sys::HtmlInputElement>() {
                        prev_input.set_value("");
                        let _ = prev_input.focus();
                    }
                    true
                } else {
                    false
                }
            }
            
            Msg::PinPaste(text) => {
                if self.is_lockout {
                    return false;
                }
                
                let digits: Vec<char> = text.chars().filter(|c| c.is_ascii_digit()).collect();
                if digits.is_empty() {
                    return false;
                }
                
                for (i, digit) in digits.into_iter().enumerate() {
                    if i < self.pin_digits.len() {
                        self.pin_digits[i] = digit.to_string();
                        if let Some(input) = self.pin_refs[i].cast::<web_sys::HtmlInputElement>() {
                            input.set_value(&digit.to_string());
                        }
                    }
                }
                
                if self.pin_digits.iter().all(|d| !d.is_empty()) {
                    ctx.link().send_message(Msg::VerifyPin);
                } else {
                    // Find first empty index and focus it
                    if let Some(first_empty) = self.pin_digits.iter().position(|d| d.is_empty()) {
                        if let Some(input) = self.pin_refs[first_empty].cast::<web_sys::HtmlInputElement>() {
                            let _ = input.focus();
                        }
                    }
                }
                true
            }
            
            Msg::VerifyPin => {
                let pin = self.pin_digits.join("");
                if pin.len() < self.pin_digits.len() {
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
