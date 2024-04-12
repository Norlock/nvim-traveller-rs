use super::lua_api_types::ExtmarkOpts;
use crate::lua_api_types::{Buffer, Mode, OptValueType, Ui, Window};
use mlua::Table;
use mlua::{prelude::LuaResult, IntoLua};
use std::path::PathBuf;

pub struct LuaApi;

#[allow(unused)]
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

impl std::fmt::Debug for StdpathType {
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

#[allow(unused)]
impl LuaApi {
    pub fn create_buf(lua: &mlua::Lua, listed: bool, scratch: bool) -> LuaResult<Buffer> {
        let lfn: mlua::Function = lua.load("vim.api.nvim_create_buf").eval()?;
        let buf_id: u32 = lfn.call::<(bool, bool), u32>((listed, scratch))?;

        Ok(Buffer::new(buf_id))
    }

    pub fn notify(lua: &mlua::Lua, display: &impl std::fmt::Debug) -> LuaResult<()> {
        let lfn: mlua::Function = lua.load("vim.notify").eval()?;

        Ok(lfn.call::<String, ()>(format!("{display:?}"))?)
    }

    pub fn set_option_value<'a, V: IntoLua<'a>>(
        lua: &'a mlua::Lua,
        key: &str,
        value: V,
        opt_type: OptValueType,
    ) -> LuaResult<()> {
        let lfn: mlua::Function = lua.load("vim.api.nvim_set_option_value").eval()?;

        let opts = lua.create_table()?;

        match opt_type {
            OptValueType::Window(window) => opts.set("win", window.id())?,
            OptValueType::Buffer(buffer) => opts.set("buf", buffer.id())?,
        }

        Ok(lfn.call::<(&str, V, mlua::Table), ()>((key, value, opts))?)
    }

    pub fn get_current_win(lua: &mlua::Lua) -> LuaResult<Window> {
        let lfn: mlua::Function = lua.load("vim.api.nvim_get_current_win").eval()?;
        let win_id = lfn.call::<(), u32>(())?;

        Ok(Window::new(win_id))
    }

    pub fn set_current_buf(lua: &mlua::Lua, buffer: Buffer) -> LuaResult<()> {
        let lfn: mlua::Function = lua.load("vim.api.nvim_set_current_buf").eval()?;

        Ok(lfn.call::<u32, ()>(buffer.id())?)
    }

    pub fn set_current_win(lua: &mlua::Lua, window: Window) -> LuaResult<()> {
        let lfn: mlua::Function = lua.load("vim.api.nvim_set_current_win").eval()?;

        Ok(lfn.call::<u32, ()>(window.id())?)
    }

    pub fn get_cwd(lua: &mlua::Lua) -> LuaResult<PathBuf> {
        let lfn: mlua::Function = lua.load("vim.fn.getcwd").eval()?;

        Ok(lfn.call::<(), String>(())?.into())
    }

    pub fn stdpath(lua: &mlua::Lua, stdpath: StdpathType) -> LuaResult<PathBuf> {
        let lfn: mlua::Function = lua.load("vim.fn.stdpath").eval()?;

        Ok(lfn.call::<String, String>(format!("{stdpath:?}"))?.into())
    }

    pub fn create_namespace(lua: &mlua::Lua, ns: &str) -> LuaResult<u32> {
        let lfn: mlua::Function = lua.load("vim.api.nvim_create_namespace").eval()?;

        Ok(lfn.call::<&str, u32>(ns)?)
    }

    pub fn buf_keymap_opts<'a>(
        lua: &'a mlua::Lua,
        silent: bool,
        buf_id: u32,
    ) -> mlua::Result<Table<'a>> {
        let table = lua.create_table()?;

        table.set("silent", silent)?;
        table.set("buffer", buf_id)?;

        Ok(table)
    }

    pub fn buf_clear_namespace(
        lua: &mlua::Lua,
        buf_id: u32,
        ns: u32,
        start: i32,
        end: i32,
    ) -> mlua::Result<()> {
        let lfn: mlua::Function = lua.load("vim.api.nvim_buf_clear_namespace").eval()?;

        Ok(lfn.call::<(u32, u32, i32, i32), ()>((buf_id, ns, start, end))?)
    }

    pub fn buf_set_lines(
        lua: &mlua::Lua,
        buf_id: u32,
        start: i32,
        end: i32,
        strict_indexing: bool,
        lines: Vec<String>,
    ) -> mlua::Result<()> {
        let lfn: mlua::Function = lua.load("vim.api.nvim_buf_set_lines").eval()?;

        Ok(lfn.call::<(u32, i32, i32, bool, Vec<String>), ()>((
            buf_id,
            start,
            end,
            strict_indexing,
            lines,
        ))?)
    }

    pub fn buf_extmark_opts<'a>(lua: &'a mlua::Lua, opts: ExtmarkOpts) -> mlua::Result<Table<'a>> {
        let table = lua.create_table()?;

        if let Some(id) = opts.id {
            table.set("id", id)?;
        }

        if let Some(end_row) = opts.end_row {
            table.set("end_row", end_row)?;
        }

        if let Some(virt_text) = opts.virt_text {
            table.set("virt_text", virt_text)?;
        }

        if let Some(virt_text_win_col) = opts.virt_text_win_col {
            table.set("virt_text_win_col", virt_text_win_col)?;
        }

        Ok(table)
    }

    pub fn list_uis(lua: &mlua::Lua) -> LuaResult<Vec<Ui>> {
        let lfn: mlua::Function = lua.load("vim.api.nvim_list_uis").eval()?;

        Ok(lfn.call::<(), Vec<Ui>>(())?)
    }

    pub fn buf_set_extmark<'a>(
        lua: &'a mlua::Lua,
        buf_id: u32,
        ns_id: u32,
        line: u32,
        col: u32,
        opts: Table<'a>,
    ) -> mlua::Result<()> {
        let lfn: mlua::Function = lua.load("vim.api.nvim_buf_set_extmark").eval()?;

        Ok(lfn.call::<(u32, u32, u32, u32, mlua::Table), ()>((buf_id, ns_id, line, col, opts))?)
    }

    pub fn set_keymap<'a>(
        lua: &'a mlua::Lua,
        mode: Mode,
        lhs: &'a str,
        rhs: mlua::Function,
        keymap_opts: Table<'a>,
    ) -> mlua::Result<()> {
        let lfn: mlua::Function = lua.load("vim.keymap.set").eval()?;

        let mode = match mode {
            Mode::Insert => "i",
            Mode::Normal => "n",
            Mode::Visual => "v",
            Mode::Select => "s",
        };

        Ok(lfn.call::<(&str, &str, mlua::Function, mlua::Table), ()>((
            mode,
            lhs,
            rhs,
            keymap_opts,
        ))?)
    }
}
