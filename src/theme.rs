use nvim_oxi::{api::{self, opts::SetMarkOpts, Buffer}, mlua::Lua};

use crate::{lua_api::LuaApi, state::AppState};

#[derive(Debug)]
pub struct Theme {
    pub navigation_ns: u32,
    pub popup_ns: u32,
    pub help_ns: u32,
    pub status_ns: u32,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            navigation_ns: api::create_namespace("TravellerNavigation"),
            popup_ns: api::create_namespace("TravellerInfo"),
            help_ns: api::create_namespace("TravellerHelp"),
            status_ns: api::create_namespace("TravellerStatus"),
        }
    }
}

impl AppState {

    pub fn theme_nav_buffer(&mut self, lua: &Lua) -> nvim_oxi::Result<()> {
        let theme = &self.theme;
        LuaApi::buf_clear_namespace(lua, self.buf.bufnr(), theme.navigation_ns, 0, -1)?;
        //self.buf.clear_namespace(self.theme.navigation_ns, 0i32..-1i32);

        if self.buf_content.is_empty() {
            // TODO cursorline false
            self.buf.set_extmark(theme.navigation_ns, 0, 0, &SetMarkOpts::builder());
        }

        Ok(())
    }
}
