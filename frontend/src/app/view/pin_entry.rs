use yew::prelude::*;

use crate::app::App;
use crate::types::Msg;

impl App {
    pub fn render_pin_entry(&self, ctx: &Context<Self>) -> Html {
        let site_title = self.config.as_ref().map(|c| c.site_title.as_str()).unwrap_or("RustDrop");
        let pin_len = self.config.as_ref().map(|c| c.pin_length).unwrap_or(4);
        
        let theme_toggle_icon = match self.theme.as_str() {
            "dark" => html! {
                <svg id="moon-icon" class="moon" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 3c.132 0 .263 0 .393 0a7.5 7.5 0 0 0 7.92 12.446a9 9 0 1 1 -8.313 -12.454z" /></svg>
            },
            "nord" => html! {
                <svg id="droplet-icon" class="droplet" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 22a7 7 0 0 0 7-7c0-4.3-7-13-7-13S5 10.7 5 15a7 7 0 0 0 7 7z"/></svg>
            },
            "dracula" => html! {
                <svg id="sparkles-icon" class="sparkles" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m12 3-1.912 5.813a2 2 0 0 1-1.275 1.275L3 12l5.813 1.912a2 2 0 0 1 1.275 1.275L12 21l1.912-5.813a2 2 0 0 1 1.275-1.275L21 12l-5.813-1.912a2 2 0 0 1-1.275-1.275Z"/><path d="m5 3 1 2.5L8.5 6 6 7 5 9.5 4 7 1.5 6 4 5Z"/><path d="m19 17 1 2.5 2.5.5-2.5 1-1 2.5-1-2.5-2.5-1 2.5-1Z"/></svg>
            },
            "sepia" => html! {
                <svg id="coffee-icon" class="coffee" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17 8h1a4 4 0 1 1 0 8h-1"/><path d="M3 8h14v9a4 4 0 0 1-4 4H7a4 4 0 0 1-4-4Z"/><line x1="6" y1="2" x2="6" y2="4"/><line x1="10" y1="2" x2="10" y2="4"/><line x1="14" y1="2" x2="14" y2="4"/></svg>
            },
            _ => html! {
                <svg id="sun-icon" class="sun" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="4" /><path d="M12 2v2" /><path d="M12 20v2" /><path d="M4.93 4.93l1.41 1.41" /><path d="M17.66 17.66l1.41 1.41" /><path d="M2 12h2" /><path d="M20 12h2" /><path d="M6.34 17.66l-1.41 1.41" /><path d="M19.07 4.93l-1.41 1.41" /></svg>
            },
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
