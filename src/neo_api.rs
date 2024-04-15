use super::neo_api_types::ExtmarkOpts;
use crate::neo_api_types::{
    AutoCmd, AutoCmdEvent, AutoCmdOpts, Buffer, LogLevel, Mode, OpenIn, OptValueType, StdpathType, Ui, WinCursor, Window
};
use mlua::{
    prelude::{LuaError, LuaFunction, LuaResult, LuaTable, LuaValue},
    IntoLua,
};
use mlua::{Lua, Table};
use std::path::PathBuf;

pub struct NeoApi;

#[allow(unused)]
impl NeoApi {
    /**
    Creates a new, empty, unnamed buffer.

    Parameters: ~
      • {listed}   Sets 'buflisted'
      • {scratch}  Creates a "throwaway" |scratch-buffer| for temporary work
                   (always 'nomodified'). Also sets 'nomodeline' on the
                   buffer.

    Return: ~
        Buffer handle, or 0 on error

    See also: ~
      • buf_open_scratch
    */
    pub fn create_buf(lua: &mlua::Lua, listed: bool, scratch: bool) -> LuaResult<Buffer> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_create_buf").eval()?;
        let buf_id: u32 = lfn.call::<(bool, bool), u32>((listed, scratch))?;

        if buf_id == 0 {
            return Err(LuaError::RuntimeError("Retrieved buffer 0".to_string()));
        }

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
    pub fn buf_delete<'a>(
        lua: &'a mlua::Lua,
        buf_id: u32,
        opts: Option<LuaTable<'a>>,
    ) -> LuaResult<()> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_buf_delete").eval()?;

        // Bug in nvim API, it won't allow opts not being passed, so create an empty table
        let opts = if opts.is_none() {
            lua.create_table()?
        } else {
            opts.unwrap()
        };

        lfn.call::<(u32, LuaTable), ()>((buf_id, opts))
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
        let lfn: LuaFunction = lua.load("vim.notify").eval()?;

        lfn.call::<String, ()>(format!("{display:?}"))
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
        let lfn: LuaFunction = lua.load("vim.notify").eval()?;

        lfn.call::<(String, usize), ()>((format!("{display:?}"), level as usize))
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
        lua: &'a mlua::Lua,
        key: &str,
        value: V,
        opt_type: OptValueType,
    ) -> LuaResult<()> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_set_option_value").eval()?;

        let opts = lua.create_table()?;

        match opt_type {
            OptValueType::Window(window) => opts.set("win", window.id())?,
            OptValueType::Buffer(buffer) => opts.set("buf", buffer.id())?,
        }

        lfn.call::<(&str, V, mlua::Table), ()>((key, value, opts))
    }

    pub fn get_current_win(lua: &mlua::Lua) -> LuaResult<Window> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_get_current_win").eval()?;
        let win_id = lfn.call::<(), u32>(())?;

        Ok(Window::new(win_id))
    }

    pub fn set_current_buf(lua: &mlua::Lua, buf_id: u32) -> LuaResult<()> {
        let lfn: mlua::Function = lua.load("vim.api.nvim_set_current_buf").eval()?;

        lfn.call::<u32, ()>(buf_id)
    }

    /**
    Sets the current window

    Attributes: ~
    &emsp; not allowed when |textlock| is active or in the |cmdwin|
    */
    pub fn set_current_win(lua: &mlua::Lua, win_id: u32) -> LuaResult<()> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_set_current_win").eval()?;

        lfn.call::<u32, ()>(win_id)
    }

    pub fn open_file(lua: &mlua::Lua, open_in: OpenIn, path: &str) -> LuaResult<()> {
        let lfn: LuaFunction = lua.load(format!("vim.cmd.{open_in}")).eval()?;

        lfn.call::<&str, ()>(path)
    }

    /**
    Gets the (1,0)-indexed, buffer-relative cursor position for a given window
    (different windows showing the same buffer have independent cursor
    positions). |api-indexing|

    Parameters: ~
    &nbsp; • {window}  Window handle, or 0 for current window

    See also: ~
    &nbsp; • |getcurpos()|
    */
    pub fn win_get_cursor(lua: &Lua, win_id: u32) -> LuaResult<WinCursor> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_win_get_cursor").eval()?;

        lfn.call::<u32, WinCursor>(win_id)
    }

    /**
    Sets the (1,0)-indexed cursor position in the window. |api-indexing| This
    scrolls the window even if it is not the current one.

    Parameters: ~
      • Window handle, or 0 for current window
      • WinCursor
    */
    pub fn win_set_cursor(lua: &Lua, win_id: u32, cursor: WinCursor) -> LuaResult<()> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_win_set_cursor").eval()?;

        lfn.call::<(u32, WinCursor), ()>((win_id, cursor))
    }

    pub fn set_cwd(lua: &Lua, path: &PathBuf) -> LuaResult<()> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_set_current_dir").eval()?;

        lfn.call::<String, ()>(path.to_string_lossy().to_string())
    }

    pub fn get_cwd(lua: &mlua::Lua) -> LuaResult<PathBuf> {
        let lfn: LuaFunction = lua.load("vim.fn.getcwd").eval()?;

        Ok(lfn.call::<(), String>(())?.into())
    }

    pub fn get_filepath(lua: &mlua::Lua) -> LuaResult<PathBuf> {
        let lfn: LuaFunction = lua.load("vim.fn.expand").eval()?;

        Ok(lfn.call::<&str, String>("%:p")?.into())
    }

    pub fn get_filename(lua: &mlua::Lua) -> LuaResult<String> {
        let lfn: LuaFunction = lua.load("vim.fn.expand").eval()?;

        lfn.call::<&str, String>("%:p:t")
    }

    pub fn get_filedir(lua: &mlua::Lua) -> LuaResult<PathBuf> {
        let lfn: LuaFunction = lua.load("vim.fn.expand").eval()?;

        Ok(lfn.call::<&str, String>("%:p:h")?.into())
    }

    pub fn set_cmd_file(lua: &Lua, name: &str) -> LuaResult<()> {
        let lfn: mlua::Function = lua.load("vim.cmd.file").eval()?;

        lfn.call::<&str, ()>(name)
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
        let lfn: LuaFunction = lua.load("vim.fn.stdpath").eval()?;

        Ok(lfn.call::<String, String>(stdpath.to_string())?.into())
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
        let lfn: LuaFunction = lua.load("vim.api.nvim_create_namespace").eval()?;

        lfn.call::<&str, u32>(ns)
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
        let lfn: LuaFunction = lua.load("vim.api.nvim_buf_add_highlight").eval()?;

        lfn.call::<(u32, i32, &str, usize, u32, i32), i32>((
            buf_id, ns_id, hl_group, line, col_start, col_end,
        ))
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

    /**
    Clears |namespace|d objects (highlights, |extmarks|, virtual text) from a
    region.

    Lines are 0-indexed. |api-indexing| To clear the namespace in the entire
    buffer, specify line_start=0 and line_end=-1.

    Parameters: ~
      • {buffer}      Buffer handle, or 0 for current buffer
      • {ns_id}       Namespace to clear, or -1 to clear all namespaces.
      • {line_start}  Start of range of lines to clear
      • {line_end}    End of range of lines to clear (exclusive) or -1 to
                      clear to end of buffer.
    */
    pub fn buf_clear_namespace(
        lua: &mlua::Lua,
        buf_id: u32,
        ns_id: i32,
        start: u32,
        end: i32,
    ) -> mlua::Result<()> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_buf_clear_namespace").eval()?;

        lfn.call::<(u32, i32, u32, i32), ()>((buf_id, ns_id, start, end))
    }

    /**
    Sets (replaces) a line-range in the buffer.

    Indexing is zero-based, end-exclusive. Negative indices are interpreted as
    length+1+index: -1 refers to the index past the end. So to change or
    delete the last element use start=-2 and end=-1.

    To insert lines at a given index, set `start` and `end` to the same index.
    To delete a range of lines, set `replacement` to an empty array.

    Out-of-bounds indices are clamped to the nearest valid value, unless
    `strict_indexing` is set.

    Attributes: ~
        not allowed when |textlock| is active

    Parameters: ~
      • {buffer}           Buffer handle, or 0 for current buffer
      • {start}            First line index
      • {end}              Last line index, exclusive
      • {strict_indexing}  Whether out-of-bounds should be an error.
      • {replacement}      Array of lines to use as replacement

    See also: ~
      • |nvim_buf_set_text()|
    */
    pub fn buf_set_lines(
        lua: &mlua::Lua,
        buf_id: u32,
        start: i32,
        end: i32,
        strict_indexing: bool,
        lines: Vec<String>,
    ) -> mlua::Result<()> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_buf_set_lines").eval()?;

        lfn.call::<(u32, i32, i32, bool, Vec<String>), ()>((
            buf_id,
            start,
            end,
            strict_indexing,
            lines,
        ))
    }

    pub fn list_uis(lua: &mlua::Lua) -> LuaResult<Vec<Ui>> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_list_uis").eval()?;

        lfn.call::<(), Vec<Ui>>(())
    }

    /**
    Creates or updates an |extmark|.

    By default a new extmark is created when no id is passed in, but it is
    also possible to create a new mark by passing in a previously unused id or
    move an existing mark by passing in its id. The caller must then keep
    track of existing and unused ids itself. (Useful over RPC, to avoid
    waiting for the return value.)

    Using the optional arguments, it is possible to use this to highlight a
    range of text, and also to associate virtual text to the mark.

    If present, the position defined by `end_col` and `end_row` should be
    after the start position in order for the extmark to cover a range. An
    earlier end position is not an error, but then it behaves like an empty
    range (no highlighting).

    Parameters: ~
      • {buffer}  Buffer handle, or 0 for current buffer
      • {ns_id}   Namespace id from |nvim_create_namespace()|
      • {line}    Line where to place the mark, 0-based. |api-indexing|
      • {col}     Column where to place the mark, 0-based. |api-indexing|
      • {opts}    Optional parameters.
    */
    pub fn buf_set_extmark<'a>(
        lua: &'a mlua::Lua,
        buf_id: u32,
        ns_id: u32,
        line: u32,
        col: u32,
        opts: ExtmarkOpts,
    ) -> mlua::Result<()> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_buf_set_extmark").eval()?;
        let opts: LuaValue = opts.into_lua(lua)?;

        lfn.call::<(u32, u32, u32, u32, LuaValue), ()>((buf_id, ns_id, line, col, opts))
    }

    pub fn set_keymap<'a>(
        lua: &'a mlua::Lua,
        mode: Mode,
        lhs: &'a str,
        rhs: mlua::Function,
        keymap_opts: Table<'a>,
    ) -> mlua::Result<()> {
        let lfn: LuaFunction = lua.load("vim.keymap.set").eval()?;

        let mode = match mode {
            Mode::Insert => "i",
            Mode::Normal => "n",
            Mode::Visual => "v",
            Mode::Select => "s",
        };

        lfn.call::<(&str, &str, mlua::Function, mlua::Table), ()>((mode, lhs, rhs, keymap_opts))
    }

    /// Creates an |autocommand| event handler, defined by `callback`
    pub fn create_autocmd<'a>(lua: &'a Lua, events: Vec<AutoCmdEvent>, opts: AutoCmdOpts<'a>) ->
        LuaResult<AutoCmd> {
        let lfn: LuaFunction = lua.load("vim.api.nvim_create_autocmd").eval()?;

        let events = events.iter().map(|e| e.to_string()).collect();

        let id = lfn.call::<(Vec<String>, AutoCmdOpts), u32>((events, opts))?;

        Ok(AutoCmd::new(id))
    }
}
