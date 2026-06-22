use yew::prelude::*;

use crate::app::App;
use crate::types::Msg;

impl App {
    pub fn render_pin_entry(&self, ctx: &Context<Self>) -> Html {
        let site_title = self.config.as_ref().map(|c| c.site_title.as_str()).unwrap_or("RustDrop");
        let pin_len = self.config.as_ref().map(|c| c.pin_length).unwrap_or(4);
        
        let theme_toggle_icon = if self.theme == "dark" {
            html! {
                <svg id="sun-icon" class="sun" width="24" height="24" fill="none" stroke="currentColor" stroke-width="2">
                    <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
                    <path d="M14.828 14.828a4 4 0 1 0 -5.656 -5.656a4 4 0 0 0 5.656 5.656z" />
                    <path d="M6.343 17.657l-1.414 1.414" />
                    <path d="M6.343 6.343l-1.414 -1.414" />
                    <path d="M17.657 6.343l1.414 -1.414" />
                    <path d="M17.657 17.657l1.414 1.414" />
                    <path d="M4 12h-2" />
                    <path d="M12 4v-2" />
                    <path d="M20 12h2" />
                    <path d="M12 20v2" />
                </svg>
            }
        } else {
            html! {
                <svg id="moon-icon" class="moon" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
                    <path d="M12 3c.132 0 .263 0 .393 0a7.5 7.5 0 0 0 7.92 12.446a9 9 0 1 1 -8.313 -12.454z" />
                </svg>
            }
        };

        html! {
            <div class="login-container">
                <button id="theme-toggle" class="theme-toggle" onclick={ctx.link().callback(|_| Msg::ToggleTheme)} aria-label="Toggle dark mode">
                    {theme_toggle_icon}
                </button>
                <div id="login-content">
                    <div class="pin-header">
                        <h1 id="site-title">{site_title}</h1>
                        <h2>
                            {if self.is_lockout { "Locked Out" } else { "Enter PIN" }}
                        </h2>
                    </div>
                    <form id="pin-form" onsubmit={ctx.link().callback(|e: SubmitEvent| { e.prevent_default(); Msg::VerifyPin })}>
                        <input
                            ref={self.pin_ref.clone()}
                            type="password"
                            class="pin-input-field"
                            disabled={self.is_lockout}
                            value={self.pin_input.clone()}
                            oninput={ctx.link().callback(|e: InputEvent| {
                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                Msg::PinInputChanged(input.value())
                            })}
                            placeholder={"• ".repeat(pin_len).trim().to_string()}
                            maxlength={pin_len.to_string()}
                            autofocus=true
                        />
                    </form>
                    {if let Some(ref err) = self.error_message {
                        html! { <p id="pin-error" class="error-message">{err}</p> }
                    } else {
                        html! {}
                    }}
                </div>
            </div>
        }
    }
}
