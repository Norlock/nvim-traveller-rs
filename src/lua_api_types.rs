use nvim_oxi::mlua::{self, FromLua, LuaSerdeExt, Table, Value};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtTextPos {
    Eol,
    Overlay,
    RightAlign,
    Inline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HlMode {
    Replace,
    Combine,
    Blend,
}

#[derive(Debug, Default)]
pub struct ExtmarkOpts<'lua> {
    pub id: Option<u32>,
    pub end_row: Option<i32>,
    pub end_col: Option<i32>,
    pub hl_group: Option<String>,
    pub hl_eol: Option<bool>,
    pub virt_text: Option<Vec<mlua::Table<'lua>>>,
    pub virt_text_pos: Option<VirtTextPos>,
    pub virt_text_win_col: Option<u32>,
    pub hl_mode: Option<HlMode>,
    pub virt_lines_above: Option<bool>,
    // TODO more
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UI {
    pub chan: u32,
    pub ext_cmdline: bool,
    pub ext_hlstate: bool,
    pub ext_linegrid: bool,
    pub ext_messages: bool,
    pub ext_multigrid: bool,
    pub ext_popupmenu: bool,
    pub ext_tabline: bool,
    pub ext_termcolors: bool,
    pub ext_wildmenu: bool,
    pub height: u32,
    pub r#override: bool,
    pub rgb: bool,
    pub stdin_tty: bool,
    pub stdout_tty: bool,
    pub term_background: String,
    pub term_colors: u32,
    pub term_name: String,
    pub width: u32,
}

impl<'lua> FromLua<'lua> for UI {
    fn from_lua(value: LuaValue<'lua>, lua: &'lua mlua::prelude::Lua) -> mlua::prelude::LuaResult<Self> {
        let ui: UI = lua.

    }
}

