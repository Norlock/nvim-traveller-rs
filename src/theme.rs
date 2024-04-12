use std::{error::Error, io};

use crate::{
    lua_api::LuaApi,
    lua_api_types::{ExtmarkOpts, Ui},
    state::AppState,
};
use mlua::prelude::{LuaError, LuaExternalError, LuaResult};

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
            self.navigation_ns = LuaApi::create_namespace(lua, "TravellerNavigation")?;
            self.popup_ns = LuaApi::create_namespace(lua, "TravellerInfo")?;
            self.help_ns = LuaApi::create_namespace(lua, "TravellerHelp")?;
            self.status_ns = LuaApi::create_namespace(lua, "TravellerStatus")?;
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
        LuaApi::buf_clear_namespace(lua, self.buf.id(), theme.navigation_ns, 0, -1)?;
        //self.buf.clear_namespace(self.theme.navigation_ns, 0i32..-1i32);

        if self.buf_content.is_empty() {
            // TODO cursorline false
            let ui = &LuaApi::list_uis(lua)?[0];

            let text = "Traveller - (Empty directory)".to_string();
            let width = text.len() as u32;
            let center = ((ui.width - width) as f32 * 0.5).round() as u32 - 2;

            let virt_text_item = lua.create_table()?;
            virt_text_item.push(text)?;
            virt_text_item.push("Comment")?;

            let opts = LuaApi::buf_extmark_opts(
                lua,
                ExtmarkOpts {
                    id: Some(1),
                    end_row: Some(0),
                    virt_text: Some(vec![virt_text_item]),
                    virt_text_win_col: Some(center),
                    ..Default::default()
                },
            )?;

            LuaApi::buf_set_extmark(lua, self.buf.id(), theme.navigation_ns, 0, 0, opts)?;
        }

        Ok(())
    }
}
