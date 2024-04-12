use mlua::prelude::*;
use serde::{Deserialize, Serialize};

use crate::lua_api::LuaApi;

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

pub enum OptValueType {
    Window(Window),
    Buffer(Buffer),
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
pub struct Ui {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
    Visual,
    Select,
}

impl Mode {
    pub fn get_char(&self) -> char {
        match self {
            Mode::Insert => 'i',
            Mode::Normal => 'n',
            Mode::Visual => 'v',
            Mode::Select => 's',
            _ => 'n',
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Window(u32);

impl Window {
    pub const ZERO: Self = Self(0);

    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn id(&self) -> u32 {
        self.0
    }

    pub fn set_option<'a, V: IntoLua<'a>>(
        &self,
        lua: &'a Lua,
        key: &str,
        value: V,
    ) -> LuaResult<()> {
        LuaApi::set_option_value(lua, key, value, OptValueType::Window(*self))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Buffer(u32);

impl Buffer {
    pub const ZERO: Self = Self(0);

    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn id(&self) -> u32 {
        self.0
    }

    pub fn set_option<'a, V: IntoLua<'a>>(
        &self,
        lua: &'a Lua,
        key: &str,
        value: V,
    ) -> LuaResult<()> {
        LuaApi::set_option_value(lua, key, value, OptValueType::Buffer(*self))
    }
}

impl<'lua> FromLua<'lua> for Ui {
    fn from_lua(
        value: LuaValue<'lua>,
        lua: &'lua mlua::prelude::Lua,
    ) -> mlua::prelude::LuaResult<Self> {
        lua.from_value(value)
    }
}
