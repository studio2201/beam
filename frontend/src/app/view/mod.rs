pub mod pin_entry;
pub mod uploader;
pub mod explorer;

use yew::prelude::*;

use crate::app::App;
use crate::types::Msg;

impl App {
    pub fn render_view(&self, ctx: &Context<Self>) -> Html {
        let site_title = self.config.as_ref().map(|c| c.site_title.as_str()).unwrap_or("RustDrop");

        html! {
            <div class="container">
                // Theme Toggle
                <button class="theme-toggle" onclick={ctx.link().callback(|_| Msg::ToggleTheme)} aria-label="Toggle theme">
                    <svg
                      xmlns="http://www.w3.org/2000/svg"
                      class="theme-toggle-icon"
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      stroke-width="2"
                      stroke-linecap="round"
                      stroke-linejoin="round"
                    >
                      {if self.theme == "light" {
                          html! {
                              <path class="moon" d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
                          }
                      } else {
                          html! {
                              <>
                                  <circle class="sun" cx="12" cy="12" r="5" />
                                  <line class="sun" x1="12" y1="1" x2="12" y2="3" />
                                  <line class="sun" x1="12" y1="21" x2="12" y2="23" />
                                  <line class="sun" x1="4.22" y1="4.22" x2="5.64" y2="5.64" />
                                  <line class="sun" x1="18.36" y1="18.36" x2="19.78" y2="19.78" />
                                  <line class="sun" x1="1" y1="12" x2="3" y2="12" />
                                  <line class="sun" x1="21" y1="12" x2="23" y2="12" />
                                  <line class="sun" x1="4.22" y1="19.78" x2="5.64" y2="18.36" />
                                  <line class="sun" x1="18.36" y1="5.64" x2="19.78" y2="4.22" />
                              </>
                          }
                      }}
                    </svg>
                </button>
                
                <h1>{site_title}</h1>
                
                {if !self.is_authenticated {
                    self.render_pin_entry(ctx)
                } else {
                    html! {
                        <>
                            {self.render_uploader(ctx)}
                            
                            {if self.config.as_ref().map(|c| c.show_file_list).unwrap_or(false) {
                                self.render_explorer(ctx)
                            } else {
                                // Logout button when file list is hidden but PIN is required
                                if self.config.as_ref().map(|c| c.pin_required).unwrap_or(false) {
                                    html! {
                                        <div class="uploaded-files-stats" style="justify-content: center; margin-top: 20px;">
                                            <button class="refresh-btn" style="background-color: var(--danger-color); padding: 8px 16px;" onclick={ctx.link().callback(|_| Msg::Logout)}>
                                                {"Logout"}
                                            </button>
                                        </div>
                                    }
                                } else {
                                    html! {}
                                }
                            }}
                        </>
                    }
                }}

                // Rename Modal Dialog
                {if self.rename_target.is_some() {
                    html! {
                        <div id="renameModal" class="rename-modal show" onclick={ctx.link().callback(|e: MouseEvent| {
                            let target_el: web_sys::HtmlElement = e.target_unchecked_into();
                            if target_el.id() == "renameModal" {
                                Msg::CancelRename
                            } else {
                                Msg::Nothing
                            }
                        })}>
                            <div class="rename-modal-content">
                                <h3>{"Rename Item"}</h3>
                                <input 
                                    type="text" 
                                    id="renameInput" 
                                    class="rename-input" 
                                    value={self.rename_input_val.clone()} 
                                    oninput={ctx.link().callback(|e: InputEvent| {
                                        let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                        Msg::RenameInputChanged(input.value())
                                    })}
                                    onkeydown={ctx.link().callback(|e: KeyboardEvent| {
                                        if e.key() == "Enter" {
                                            Msg::ConfirmRename
                                        } else if e.key() == "Escape" {
                                            Msg::CancelRename
                                        } else {
                                            Msg::Nothing
                                        }
                                    })}
                                />
                                <div class="rename-actions">
                                    <button class="modal-btn modal-btn-cancel" onclick={ctx.link().callback(|_| Msg::CancelRename)}>
                                        {"Cancel"}
                                    </button>
                                    <button class="modal-btn modal-btn-confirm" onclick={ctx.link().callback(|_| Msg::ConfirmRename)}>
                                        {"Rename"}
                                    </button>
                                </div>
                            </div>
                        </div>
                    }
                } else {
                    html! {}
                }}

                // Toast Notification Overlay
                <div class="toast-container">
                    {for self.toasts.iter().map(|toast| {
                        html! {
                            <div key={toast.id} class={classes!("toast", format!("toast-{}", toast.toast_type))}>
                                {&toast.message}
                            </div>
                        }
                    })}
                </div>
            </div>
        }
    }
}
