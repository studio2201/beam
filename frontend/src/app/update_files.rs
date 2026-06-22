use yew::prelude::*;

use crate::app::App;
use crate::types::{Msg, RenameData};
use crate::api::{fetch_files, delete_file_api, rename_file_api};

impl App {
    pub fn update_files(&mut self, ctx: &Context<Self>, msg: Msg) -> bool {
        match msg {
            Msg::LoadFileList(res) => {
                match res {
                    Ok(data) => {
                        self.uploaded_files = Some(data);
                    }
                    Err(e) => {
                        self.show_toast(ctx, &format!("Failed to load files: {}", e), "error");
                    }
                }
                true
            }
            
            Msg::RefreshFiles => {
                if !self.is_authenticated {
                    return false;
                }
                
                let link = ctx.link().clone();
                wasm_bindgen_futures::spawn_local(async move {
                    match fetch_files().await {
                        Ok(data) => link.send_message(Msg::LoadFileList(Ok(data))),
                        Err(e) => link.send_message(Msg::LoadFileList(Err(e))),
                    }
                });
                false
            }
            
            Msg::DeleteFile(path) => {
                let name = path.split('/').last().unwrap_or(&path).to_string();
                let window = web_sys::window().unwrap();
                let confirm_msg = format!("Are you sure you want to delete \"{}\"?", name);
                
                if window.confirm_with_message(&confirm_msg).unwrap_or(false) {
                    let link = ctx.link().clone();
                    let path_c = path.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        match delete_file_api(&path_c).await {
                            Ok(_) => link.send_message(Msg::DeleteResult(Ok(path_c))),
                            Err(e) => link.send_message(Msg::DeleteResult(Err(e))),
                        }
                    });
                }
                false
            }
            
            Msg::DeleteResult(res) => {
                match res {
                    Ok(path) => {
                        let name = path.split('/').last().unwrap_or(&path).to_string();
                        self.show_toast(ctx, &format!("Deleted: {}", name), "success");
                        ctx.link().send_message(Msg::RefreshFiles);
                    }
                    Err(e) => {
                        self.show_toast(ctx, &format!("Delete failed: {}", e), "error");
                    }
                }
                true
            }
            
            Msg::StartRename(path, current_name) => {
                self.rename_target = Some(RenameData {
                    item_path: path,
                    current_name: current_name.clone(),
                });
                self.rename_input_val = current_name;
                true
            }
            
            Msg::CancelRename => {
                self.rename_target = None;
                self.rename_input_val.clear();
                true
            }
            
            Msg::RenameInputChanged(val) => {
                self.rename_input_val = val;
                true
            }
            
            Msg::ConfirmRename => {
                if self.rename_input_val.trim().is_empty() {
                    return false;
                }
                
                if let Some(target) = self.rename_target.take() {
                    let new_name = self.rename_input_val.trim().to_string();
                    let link = ctx.link().clone();
                    
                    wasm_bindgen_futures::spawn_local(async move {
                        match rename_file_api(&target.item_path, &new_name).await {
                            Ok(_) => link.send_message(Msg::RenameResult(Ok(new_name))),
                            Err(e) => link.send_message(Msg::RenameResult(Err(e))),
                        }
                    });
                }
                false
            }
            
            Msg::RenameResult(res) => {
                self.rename_target = None;
                self.rename_input_val.clear();
                
                match res {
                    Ok(new_name) => {
                        self.show_toast(ctx, &format!("Renamed to: {}", new_name), "success");
                        ctx.link().send_message(Msg::RefreshFiles);
                    }
                    Err(e) => {
                        self.show_toast(ctx, &format!("Rename failed: {}", e), "error");
                    }
                }
                true
            }
            _ => false,
        }
    }
}
