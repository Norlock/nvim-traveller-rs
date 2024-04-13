use crate::{neo_api::NeoApi, neo_api_types::ExtmarkOpts, state::AppState};
use mlua::prelude::LuaResult;

#[derive(Debug)]
pub struct Theme {
    pub navigation_ns: u32,
    pub popup_ns: u32,
    pub help_ns: u32,
    pub status_ns: u32,
    pub init: bool,
}

impl Theme {
    pub fn init(&mut self, lua: &mlua::Lua) -> LuaResult<()> {
        if !self.init {
            self.navigation_ns = NeoApi::create_namespace(lua, "TravellerNavigation")?;
            self.popup_ns = NeoApi::create_namespace(lua, "TravellerInfo")?;
            self.help_ns = NeoApi::create_namespace(lua, "TravellerHelp")?;
            self.status_ns = NeoApi::create_namespace(lua, "TravellerStatus")?;
            self.init = true;
        }

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
            init: false,
        }
    }
}

impl AppState {
    pub fn theme_nav_buffer(&mut self, lua: &mlua::Lua) -> LuaResult<()> {
        let theme = &self.theme;
        NeoApi::buf_clear_namespace(lua, self.buf.id(), theme.navigation_ns, 0, -1)?;

        if self.buf_content.is_empty() {
            // TODO cursorline false
            let ui = &NeoApi::list_uis(lua)?[0];
            self.win.set_option(lua, "cursorline", false)?;

            let text = "Traveller - (Empty directory)".to_string();
            let width = text.len() as u32;
            let center = ((ui.width - width) as f32 * 0.5).round() as u32 - 2;

            let virt_text_item = lua.create_table()?;
            virt_text_item.push(text)?;
            virt_text_item.push("Comment")?;

            let opts = ExtmarkOpts {
                id: Some(1),
                end_row: Some(0),
                virt_text: Some(vec![virt_text_item]),
                virt_text_win_col: Some(center),
                ..Default::default()
            };

            NeoApi::buf_set_extmark(lua, self.buf.id(), theme.navigation_ns, 0, 0, opts)?;
        } else {
            self.win.set_option(lua, "cursorline", true)?;
        }

        for (i, item_name) in self.buf_content.iter().enumerate() {
            if item_name.ends_with("/") {
                self.buf
                    .add_highlight(lua, theme.navigation_ns as i32, "Directory", i, 0, -1)?;
            }

            // TODO: selection Highlight!
        }

        Ok(())
    }
}
