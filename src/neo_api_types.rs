#![allow(unused)]
use std::fmt::{self, Display};

use mlua::prelude::*;
use serde::{Deserialize, Serialize};

use crate::neo_api::NeoApi;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VirtTextPos {
    Eol,
    Overlay,
    RightAlign,
    Inline,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HlMode {
    Replace,
    Combine,
    Blend,
}

pub enum OptValueType {
    Window(Window),
    Buffer(Buffer),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
    Off = 5,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StdpathType {
    /// Cache directory: arbitrary temporary storage for plugins, etc.
    Cache,
    /// User configuration directory. |init.vim| is stored here.
    Config,
    /// User data directory.
    Data,
    /// Logs directory (for use by plugins too).
    Log,
    /// Run directory: temporary, local storage for sockets, named pipes, etc.
    Run,
    /// Session state directory: storage for file drafts, swap, undo, |shada|.
    State,
}

impl std::fmt::Display for StdpathType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Self::Config => "config",
            Self::Cache => "cache",
            Self::Data => "data",
            Self::Log => "log",
            Self::Run => "run",
            Self::State => "state",
        };

        f.write_str(str)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OpenIn {
    Buffer,
    VSplit,
    HSplit,
    Tab,
}

impl Display for OpenIn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Buffer => f.write_str("edit"),
            Self::Tab => f.write_str("tabedit"),
            Self::VSplit => f.write_str("vsplit"),
            Self::HSplit => f.write_str("split"),
        }
    }
}

#[derive(Debug, Serialize, Default)]
/// Pleas help to add more and test
pub struct ExtmarkOpts<'lua> {
    pub id: Option<u32>,
    pub end_row: Option<i32>,
    pub end_col: Option<i32>,
    pub hl_group: Option<String>,
    pub hl_eol: Option<bool>,
    pub virt_text: Option<Vec<mlua::Table<'lua>>>,
    //pub virt_text_pos: Option<VirtTextPos>,
    pub virt_text_win_col: Option<u32>,
    //pub hl_mode: Option<HlMode>,
    pub virt_lines_above: Option<bool>,
}

impl<'a> IntoLua<'a> for ExtmarkOpts<'a> {
    fn into_lua(self, lua: &'a Lua) -> LuaResult<LuaValue<'a>> {
        let mut ser_opts = LuaSerializeOptions::new();
        ser_opts.serialize_none_to_null = false;
        ser_opts.serialize_unit_to_null = false;

        lua.to_value_with(&self, ser_opts)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct WinCursor {
    row: u32,
    pub column: u32,
}

impl WinCursor {
    // Starts from 0
    pub fn row_zero_indexed(&self) -> u32 {
        self.row - 1
    }

    // Starts from 1
    pub fn row_one_indexed(&self) -> u32 {
        self.row
    }
}

impl<'a> FromLua<'a> for WinCursor {
    fn from_lua(value: LuaValue<'a>, lua: &'a Lua) -> LuaResult<Self> {
        match value {
            LuaValue::Table(table) => {
                let row: u32 = table.get(1)?;
                let column: u32 = table.get(2)?;

                Ok(Self { row, column })
            }
            _ => Err(LuaError::DeserializeError(
                "Supposed to be a table".to_string(),
            )),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

    pub fn set_option_value<'a, V: IntoLua<'a>>(
        &self,
        lua: &'a Lua,
        key: &str,
        value: V,
    ) -> LuaResult<()> {
        NeoApi::set_option_value(lua, key, value, OptValueType::Window(*self))
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

    /**
    Sets the current buffer.

    Attributes: ~
        not allowed when |textlock| is active or in the |cmdwin|
    */
    pub fn set_current_buf(&self, lua: &mlua::Lua) -> LuaResult<()> {
        NeoApi::set_current_buf(lua, self.id())
    }

    /**
    Deletes the buffer. See |:bwipeout|

    Attributes: ~
        not allowed when |textlock| is active or in the |cmdwin|

    Parameters: ~
      • {buffer}  Buffer handle, or 0 for current buffer
      • {opts}    Optional parameters. Keys:
                  • force: Force deletion and ignore unsaved changes.
                  • unload: Unloaded only, do not delete. See |:bunload|
    */
    pub fn delete<'a>(&self, lua: &'a Lua, opts: Option<LuaTable<'a>>) -> LuaResult<()> {
        NeoApi::buf_delete(lua, self.id(), opts)
    }

    /**
    Sets the value of an option. The behavior of this function matches that of
    |:set|: for global-local options, both the global and local value are set
    unless otherwise specified with {scope}.

    Note the options {win} and {buf} cannot be used together.

    Parameters: ~
      • {name}   Option name
      • {value}  New option value
      • {opts}   Optional parameters
                 • scope: One of "global" or "local". Analogous to
                   |:setglobal| and |:setlocal|, respectively.
                 • win: |window-ID|. Used for setting window local option.
                 • buf: Buffer number. Used for setting buffer local option.
    */
    pub fn set_option_value<'a, V: IntoLua<'a>>(
        &self,
        lua: &'a Lua,
        key: &str,
        value: V,
    ) -> LuaResult<()> {
        NeoApi::set_option_value(lua, key, value, OptValueType::Buffer(*self))
    }

    /**
    Adds a highlight to buffer.

    Useful for plugins that dynamically generate highlights to a buffer (like
    a semantic highlighter or linter). The function adds a single highlight to
    a buffer. Unlike |matchaddpos()| highlights follow changes to line
    numbering (as lines are inserted/removed above the highlighted line), like
    signs and marks do.

    Namespaces are used for batch deletion/updating of a set of highlights. To
    create a namespace, use |nvim_create_namespace()| which returns a
    namespace id. Pass it in to this function as `ns_id` to add highlights to
    the namespace. All highlights in the same namespace can then be cleared
    with single call to |nvim_buf_clear_namespace()|. If the highlight never
    will be deleted by an API call, pass `ns_id = -1`.

    As a shorthand, `ns_id = 0` can be used to create a new namespace for the
    highlight, the allocated id is then returned. If `hl_group` is the empty
    string no highlight is added, but a new `ns_id` is still returned. This is
    supported for backwards compatibility, new code should use
    |nvim_create_namespace()| to create a new empty namespace.

    Parameters: ~
      • {buffer}     Buffer handle, or 0 for current buffer
      • {ns_id}      namespace to use or -1 for ungrouped highlight
      • {hl_group}   Name of the highlight group to use
      • {line}       Line to highlight (zero-indexed)
      • {col_start}  Start of (byte-indexed) column range to highlight
      • {col_end}    End of (byte-indexed) column range to highlight, or -1 to
                     highlight to end of line

    Return: ~
        The ns_id that was used
    */
    pub fn add_highlight(
        &self,
        lua: &Lua,
        ns_id: i32,
        hl_group: &str,
        line: usize,
        col_start: u32,
        col_end: i32,
    ) -> LuaResult<i32> {
        NeoApi::buf_add_highlight(lua, self.0, ns_id, hl_group, line, col_start, col_end)
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
