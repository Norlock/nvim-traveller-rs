use super::neo_api_types::ExtmarkOpts;
use crate::neo_api_types::{Buffer, LogLevel, Mode, OptValueType, StdpathType, Ui, Window};
use mlua::Table;
use mlua::{prelude::{LuaResult, LuaTable, LuaValue}, IntoLua};
use std::path::PathBuf;

pub struct NeoApi;

#[allow(unused)]
impl NeoApi {
    pub fn create_buf(lua: &mlua::Lua, listed: bool, scratch: bool) -> LuaResult<Buffer> {
        let lfn: mlua::Function = lua.load("vim.api.nvim_create_buf").eval()?;
        let buf_id: u32 = lfn.call::<(bool, bool), u32>((listed, scratch))?;

        Ok(Buffer::new(buf_id))
    }

    /**
    nvim_buf_delete({buffer}, {opts})
    Deletes the buffer. See |:bwipeout|

    Attributes: ~
        not allowed when |textlock| is active or in the |cmdwin|

    Parameters: ~
      • {buffer}  Buffer handle, or 0 for current buffer
      • {opts}    Optional parameters. Keys:
                  • force: Force deletion and ignore unsaved changes.
                  • unload: Unloaded only, do not delete. See |:bunload|
    */
    pub fn buf_delete<'a>(lua: &'a mlua::Lua, buf_id: u32, opts: Option<LuaTable<'a>>) -> LuaResult<()> {
        let lfn: mlua::Function = lua.load("vim.api.nvim_buf_delete").eval()?;

        Ok(lfn.call::<(u32, Option<LuaTable>), ()>((buf_id, opts))?)
    }

    /**
    Displays a notification to the user.

    This function can be overridden by plugins to display notifications using
    a custom provider (such as the system notification provider). By default,
    writes to |:messages|.

    Parameters: ~
      • {msg}    Content of the notification to show to the user.
    */
    pub fn notify(lua: &mlua::Lua, display: &impl std::fmt::Debug) -> LuaResult<()> {
        let lfn: mlua::Function = lua.load("vim.notify").eval()?;

        Ok(lfn.call::<String, ()>(format!("{display:?}"))?)
    }

    /**
    Displays a notification to the user.

    This function can be overridden by plugins to display notifications using
    a custom provider (such as the system notification provider). By default,
    writes to |:messages|.

    Parameters: ~
      • {msg}    Content of the notification to show to the user.
      • {level}  A log level
    */
    pub fn notify_level(
        lua: &mlua::Lua,
        display: &impl std::fmt::Debug,
        level: LogLevel,
    ) -> LuaResult<()> {
        let lfn: mlua::Function = lua.load("vim.notify").eval()?;

        Ok(lfn.call::<(String, usize), ()>((format!("{display:?}"), level as usize))?)
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

    /**
    Returns |standard-path| locations of various default files and directories.

    What          Type     Description
    cache         String   Cache directory: arbitrary temporary storage for plugins, etc.
    config        String   User configuration directory. |init.vim| is stored here.
    config_dirs   List     Other configuration directories. (TODO)
    data          String   User data directory.
    data_dirs     List     Other data directories. (TODO)
    log           String   Logs directory (for use by plugins too).
    run           String   Run directory: temporary, local storage for sockets, named pipes, etc.
    state         String   Session state directory: storage for file drafts, swap, undo, |shada|.
    */
    pub fn stdpath(lua: &mlua::Lua, stdpath: StdpathType) -> LuaResult<PathBuf> {
        let lfn: mlua::Function = lua.load("vim.fn.stdpath").eval()?;

        Ok(lfn.call::<String, String>(format!("{stdpath:?}"))?.into())
    }

    /**
    Creates a new namespace or gets an existing one.

    Namespaces are used for buffer highlights and virtual text, see
    |nvim_buf_add_highlight()| and |nvim_buf_set_extmark()|.

    Namespaces can be named or anonymous. If `name` matches an existing
    namespace, the associated id is returned. If `name` is an empty string a
    new, anonymous namespace is created.

    Parameters: ~
      • {name}  Namespace name or empty string

    Return: ~
        Namespace id
    */
    pub fn create_namespace(lua: &mlua::Lua, ns: &str) -> LuaResult<u32> {
        let lfn: mlua::Function = lua.load("vim.api.nvim_create_namespace").eval()?;

        Ok(lfn.call::<&str, u32>(ns)?)
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
    pub fn buf_add_highlight(
        lua: &mlua::Lua,
        buf_id: u32,
        ns_id: i32,
        hl_group: &str,
        line: usize,
        col_start: u32,
        col_end: i32,
    ) -> LuaResult<i32> {
        let lfn: mlua::Function = lua.load("vim.api.nvim_buf_add_highlight").eval()?;

        Ok(lfn.call::<(u32, i32, &str, usize, u32, i32), i32>((
            buf_id, ns_id, hl_group, line, col_start, col_end,
        ))?)
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
        opts: ExtmarkOpts,
    ) -> mlua::Result<()> {
        let lfn: mlua::Function = lua.load("vim.api.nvim_buf_set_extmark").eval()?;

        let opts: LuaValue = opts.into_lua(lua)?;
        Ok(lfn.call::<(u32, u32, u32, u32, LuaValue), ()>((buf_id, ns_id, line, col, opts))?)
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
