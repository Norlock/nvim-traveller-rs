use mlua::Table;
use nvim_oxi::api::types::Mode;
use nvim_oxi::mlua;
use std::path::PathBuf;

use crate::lua_opts::ExtmarkOpts;

pub struct LuaApi;

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

impl LuaApi {
    pub fn get_cwd(lua: &mlua::Lua) -> mlua::Result<PathBuf> {
        let lfn: mlua::Function = lua.load("vim.fn.getcwd").eval()?;

        Ok(lfn.call::<(), String>(())?.into())
    }

    pub fn stdpath(lua: &mlua::Lua, stdpath: StdpathType) -> mlua::Result<PathBuf> {
        let lfn: mlua::Function = lua.load("vim.fn.stdpath").eval()?;

        Ok(lfn.call::<String, String>(format!("{stdpath:?}"))?.into())
    }

    pub fn buf_keymap_opts<'a>(
        lua: &'a mlua::Lua,
        silent: bool,
        buf_id: i32,
    ) -> mlua::Result<Table<'a>> {
        let table = lua.create_table()?;

        table.set("silent", silent)?;
        table.set("buffer", buf_id)?;

        Ok(table)
    }

    pub fn buf_clear_namespace(
        lua: &mlua::Lua,
        buf_id: i32,
        ns: u32,
        start: i32,
        end: i32,
    ) -> mlua::Result<()> {
        let lfn: mlua::Function = lua.load("vim.api.nvim_buf_clear_namespace").eval()?;

        Ok(lfn.call::<(i32, u32, i32, i32), ()>((buf_id, ns, start, end))?)
    }

    pub fn buf_set_lines(
        lua: &mlua::Lua,
        buf_id: i32,
        start: i32,
        end: i32,
        strict_indexing: bool,
        lines: Vec<String>,
    ) -> mlua::Result<()> {
        let lfn: mlua::Function = lua.load("vim.api.nvim_buf_set_lines").eval()?;

        Ok(lfn.call::<(i32, i32, i32, bool, Vec<String>), ()>((
            buf_id,
            start,
            end,
            strict_indexing,
            lines,
        ))?)
    }

    pub fn buf_extmark_opts<'a>(lua: &'a mlua::Lua, opts: ExtmarkOpts) -> mlua::Result<Table<'a>> {
        let table = lua.create_table()?;

        if let Some(val) = opts.id {
            //
        }

        if let Some(val) = opts.hl_eol {
            //
        }

        Ok(table)
    }

    pub fn buf_set_extmark<'a>(
        lua: &'a mlua::Lua,
        buf_id: i32,
        ns_id: u32,
        line: u32,
        col: u32,
        opts: Table<'a>,
    ) -> mlua::Result<()> {
        let lfn: mlua::Function = lua.load("vim.api.nvim_buf_set_extmark").eval()?;

        Ok(lfn.call::<(i32, u32, u32, u32, mlua::Table), ()>((buf_id, ns_id, line, col, opts))?)
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
            _ => "n",
        };

        Ok(lfn.call::<(&str, &str, mlua::Function, mlua::Table), ()>((
            mode,
            lhs,
            rhs,
            keymap_opts,
        ))?)
    }
}
