use yew::prelude::*;
use wasm_bindgen::JsCast;

use crate::app::App;
use crate::types::Msg;

impl App {
    pub fn render_pin_entry(&self, ctx: &Context<Self>) -> Html {
        let pin_digits = self.pin_digits.clone();
        html! {
            <div class="login-container">
                <div class="pin-header">
                    <h2>{"Enter PIN"}</h2>
                </div>
                <form id="pin-form" onsubmit={ctx.link().callback(|e: SubmitEvent| { e.prevent_default(); Msg::VerifyPin })}>
                    {for self.pin_refs.iter().enumerate().map(|(idx, r)| {
                        let pin_digits = pin_digits.clone();
                        html! {
                            <input
                                ref={r.clone()}
                                type="password"
                                class={classes!(
                                    "pin-digit",
                                    if self.is_lockout { Some("locked") } else { None },
                                    if !pin_digits[idx].is_empty() { Some("filled") } else { None }
                                )}
                                maxlength="1"
                                pattern="[0-9]"
                                inputmode="numeric"
                                autocomplete="off"
                                required=true
                                disabled={self.is_lockout}
                                value={self.pin_digits[idx].clone()}
                                oninput={ctx.link().callback(move |e: InputEvent| {
                                    let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                    Msg::PinDigitInput(idx, input.value())
                                })}
                                onkeydown={ctx.link().callback(move |e: KeyboardEvent| {
                                    if e.key() == "Backspace" {
                                        Msg::PinBackspace(idx)
                                    } else {
                                        Msg::Nothing
                                    }
                                })}
                                onpaste={ctx.link().callback(move |e: Event| {
                                    let clipboard_event: web_sys::ClipboardEvent = e.unchecked_into();
                                    if let Some(dt) = clipboard_event.clipboard_data() {
                                        if let Ok(text) = dt.get_data("text") {
                                            Msg::PinPaste(text)
                                        } else {
                                            Msg::Nothing
                                        }
                                    } else {
                                        Msg::Nothing
                                    }
                                })}
                            />
                        }
                    })}
                </form>
                {if let Some(ref err) = self.error_message {
                    html! { <p id="pin-error" class="error-message">{err}</p> }
                } else {
                    html! {}
                }}
            </div>
        }
    }
}
