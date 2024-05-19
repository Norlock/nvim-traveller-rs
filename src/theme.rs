use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::state::{AppInstance, SelectionData};
use crate::CONTAINER;
use neo_api_rs::mlua::prelude::*;
use neo_api_rs::*;

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub navigation_ns: u32,
    pub popup_ns: u32,
    pub help_ns: u32,
    pub status_ns: u32,
}

impl Theme {
    pub fn init(&mut self, lua: &Lua) -> LuaResult<()> {
        self.navigation_ns = NeoTheme::create_namespace(lua, "TravellerNavigation")?;
        self.popup_ns = NeoTheme::create_namespace(lua, "TravellerInfo")?;
        self.help_ns = NeoTheme::create_namespace(lua, "TravellerHelp")?;
        self.status_ns = NeoTheme::create_namespace(lua, "TravellerStatus")?;

        Ok(())
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            navigation_ns: 0,
            popup_ns: 0,
            help_ns: 0,
            status_ns: 0,
        }
    }
}

impl AppInstance {
    pub async fn theme_nav_buffer(&mut self, lua: &Lua, selection: &SelectionData) -> LuaResult<()> {
        let theme = CONTAINER.theme.read().await;
        self.buf
            .clear_namespace(lua, theme.navigation_ns as i32, 0, -1)?;

        if self.buf_content.is_empty() {
            let ui = &NeoApi::list_uis(lua)?[0];
            self.win.set_option_value(lua, "cursorline", false)?;

            let text = "Traveller - (Empty directory)";
            let width = text.len() as u32;
            let center = ((ui.width - width) as f32 * 0.5).round() as u32 - 2;

            let hl_text = HLText::new(text, "Comment");

            let opts = ExtmarkOpts {
                id: Some(1),
                end_row: Some(0),
                virt_text: Some(vec![hl_text]),
                virt_text_win_col: Some(center),
                ..Default::default()
            };

            self.buf
                .set_extmarks(lua, theme.navigation_ns, 0, 0, opts)?;
        } else {
            self.win.set_option_value(lua, "cursorline", true)?;
        }

        for (i, item_name) in self.buf_content.iter().enumerate() {
            if item_name.ends_with("/") {
                self.buf
                    .add_highlight(lua, theme.navigation_ns as i32, "Directory", i, 0, -1)?;
            }

            if self.is_selected(&selection, item_name) {
                self.buf
                    .add_highlight(lua, theme.navigation_ns as i32, "Special", i, 0, -1)?;
            }
        }

        Ok(())
    }

    fn is_selected(&self, selection: &HashMap<PathBuf, HashSet<String>>,  item_name: &str) -> bool {
        if let Some(cwd) = selection.get(&self.cwd) {
            cwd.contains(item_name)
        } else {
            false
        }
    }
}
