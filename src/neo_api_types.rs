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
    /// Create a cursor where the passed row argument starts from 0
    pub fn from_zero_indexed(row: u32, column: u32) -> Self {
        Self {
            row: row + 1,
            column,
        }
    }

    /// Create a cursor where the passed row argument starts from 1
    pub fn from_one_indexed(row: u32, column: u32) -> Self {
        Self { row, column }
    }

    /// Return value starts from 0
    pub fn row_zero_indexed(&self) -> u32 {
        self.row - 1
    }

    /// Return value starts from 1
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

impl<'a> IntoLua<'a> for WinCursor {
    fn into_lua(self, lua: &'a Lua) -> LuaResult<LuaValue<'a>> {
        let table = lua.create_table()?;
        table.set(1, self.row);
        table.set(2, self.column);
        
        Ok(LuaValue::Table(table))
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Window(u32);

impl Window {
    pub const ZERO: Self = Self(0);

    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn id(&self) -> u32 {
        self.0
    }

    /**
    Sets the value of an option. The behavior of this function matches that of
    |:set|: for global-local options, both the global and local value are set
    unless otherwise specified with {scope}.

    Parameters: ~
      • {name}   Option name
      • {value}  New option value
    */
    pub fn set_option_value<'a, V: IntoLua<'a>>(
        &self,
        lua: &'a Lua,
        key: &str,
        value: V,
    ) -> LuaResult<()> {
        NeoApi::set_option_value(lua, key, value, OptValueType::Window(*self))
    }

    pub fn set_cursor(&self, lua: &Lua, cursor: WinCursor) -> LuaResult<()> {
        NeoApi::win_set_cursor(lua, self.id(), cursor)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

    Parameters: ~
      • {name}   Option name
      • {value}  New option value
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
      • {start}            First line index
      • {end}              Last line index, exclusive
      • {strict_indexing}  Whether out-of-bounds should be an error.
      • {replacement}      Array of lines to use as replacement

    See also: ~
      • |nvim_buf_set_text()|
    */
    pub fn set_lines(
        &self,
        lua: &Lua,
        start: i32,
        end: i32,
        strict_indexing: bool,
        lines: Vec<String>,
    ) -> mlua::Result<()> {
        NeoApi::buf_set_lines(lua, self.id(), start, end, strict_indexing, lines)
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
      • {ns_id}   Namespace id from |nvim_create_namespace()|
      • {line}    Line where to place the mark, 0-based. |api-indexing|
      • {col}     Column where to place the mark, 0-based. |api-indexing|
      • {opts}    Optional parameters.
    */
    pub fn set_extmarks<'a>(
        &self,
        lua: &'a Lua,
        ns_id: u32,
        line: u32,
        col: u32,
        opts: ExtmarkOpts<'a>,
    ) -> LuaResult<()> {
        NeoApi::buf_set_extmark(lua, self.id(), ns_id, line, col, opts)
    }

    /**
    Clears |namespace|d objects (highlights, |extmarks|, virtual text) from a
    region.

    Lines are 0-indexed. |api-indexing| To clear the namespace in the entire
    buffer, specify line_start=0 and line_end=-1.

    Parameters: ~
      • {ns_id}       Namespace to clear, or -1 to clear all namespaces.
      • {line_start}  Start of range of lines to clear
      • {line_end}    End of range of lines to clear (exclusive) or -1 to
                      clear to end of buffer.
    */
    pub fn clear_namespace(&self, lua: &Lua, ns_id: i32, start: u32, end: i32) -> LuaResult<()> {
        NeoApi::buf_clear_namespace(lua, self.id(), ns_id, start, end)
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AutoCmd(u32);

impl AutoCmd {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn id(&self) -> u32 {
        self.0
    }
}

#[derive(Clone, Debug)]
pub enum AutoCmdGroup {
    String(String),
    Integer(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoCmdEvent {
    /// After adding a buffer to the buffer list.
    BufAdd,
    /// Deleting a buffer from the buffer list.
    BufDelete,
    /// After entering a buffer.
    BufEnter,
    /// After renaming a buffer.
    BufFilePost,
    /// Before renaming a buffer.
    BufFilePre,
    /// Just after buffer becomes hidden.
    BufHidden,
    /// Before leaving a buffer.
    BufLeave,
    /// After the 'modified' state of a buffer changes.
    BufModifiedSet,
    /// After creating any buffer.
    BufNew,
    /// When creating a buffer for a new file.
    BufNewFile,
    /// Read buffer using command.
    BufReadCmd,
    /// After reading a buffer.
    BufReadPost,
    /// Before reading a buffer.
    BufReadPre,
    /// Just before unloading a buffer.
    BufUnload,
    /// After showing a buffer in a window.
    BufWinEnter,
    /// Just after buffer removed from window.
    BufWinLeave,
    /// Just before really deleting a buffer.
    BufWipeout,
    /// Write buffer using command.
    BufWriteCmd,
    /// After writing a buffer.
    BufWritePost,
    /// Before writing a buffer.
    BufWritePre,
    /// Info was received about channel.
    ChanInfo,
    /// Channel was opened.
    ChanOpen,
    /// Command undefined.
    CmdUndefined,
    /// After entering the cmdline window.
    CmdWinEnter,
    /// Before leaving the cmdline window.
    CmdWinLeave,
    /// Command line was modified.
    CmdlineChanged,
    /// After entering cmdline mode.
    CmdlineEnter,
    /// Before leaving cmdline mode.
    CmdlineLeave,
    /// After loading a colorscheme.
    ColorScheme,
    /// Before loading a colorscheme.
    ColorSchemePre,
    /// After popup menu changed.
    CompleteChanged,
    /// After finishing insert complete.
    CompleteDone,
    /// Idem, before clearing info.
    CompleteDonePre,
    /// Cursor in same position for a while.
    CursorHold,
    /// Idem, in Insert mode.
    CursorHoldI,
    /// Cursor was moved.
    CursorMoved,
    /// Cursor was moved in Insert mode.
    CursorMovedI,
    /// Diagnostics in a buffer were modified.
    DiagnosticChanged,
    /// Diffs have been updated.
    DiffUpdated,
    /// Directory changed.
    DirChanged,
    /// Directory is going to change.
    DirChangedPre,
    /// After changing the 'encoding' option.
    EncodingChanged,
    /// Before exiting.
    ExitPre,
    /// Append to a file using command.
    FileAppendCmd,
    /// After appending to a file.
    FileAppendPost,
    /// Before appending to a file.
    FileAppendPre,
    /// Before first change to read-only file.
    FileChangedRO,
    /// After shell command that changed file.
    FileChangedShell,
    /// After (not) reloading changed file.
    FileChangedShellPost,
    /// Read from a file using command.
    FileReadCmd,
    /// After reading a file.
    FileReadPost,
    /// Before reading a file.
    FileReadPre,
    /// New file type detected (user defined).
    FileType,
    /// Write to a file using command.
    FileWriteCmd,
    /// After writing a file.
    FileWritePost,
    /// Before writing a file.
    FileWritePre,
    /// After reading from a filter.
    FilterReadPost,
    /// Before reading from a filter.
    FilterReadPre,
    /// After writing to a filter.
    FilterWritePost,
    /// Before writing to a filter.
    FilterWritePre,
    /// Got the focus.
    FocusGained,
    /// Lost the focus to another app.
    FocusLost,
    /// If calling a function which doesn't exist.
    FuncUndefined,
    /// After starting the GUI.
    GUIEnter,
    /// After starting the GUI failed.
    GUIFailed,
    /// When changing Insert/Replace mode.
    InsertChange,
    /// Before inserting a char.
    InsertCharPre,
    /// When entering Insert mode.
    InsertEnter,
    /// Just after leaving Insert mode.
    InsertLeave,
    /// Just before leaving Insert mode.
    InsertLeavePre,
    /// After an LSP client attaches to a buffer.
    LspAttach,
    /// After an LSP client detaches from a buffer.
    LspDetach,
    /// After an LSP request is started, canceled, or completed.
    LspRequest,
    /// After an LSP notice has been sent to the server.
    LspNotify,
    /// After a visible LSP token is updated.
    LspTokenUpdate,
    /// After a LSP progress update.
    LspProgress,
    /// Just before popup menu is displayed.
    MenuPopup,
    /// After changing the mode.
    ModeChanged,
    /// After setting any option.
    OptionSet,
    /// After :make, :grep etc.
    QuickFixCmdPost,
    /// Before :make, :grep etc.
    QuickFixCmdPre,
    /// Before :quit.
    QuitPre,
    /// When starting to record a macro.
    RecordingEnter,
    /// Just before a macro stops recording.
    RecordingLeave,
    /// Upon string reception from a remote vim.
    RemoteReply,
    /// Going to wait for a character.
    SafeState,
    /// After the search wrapped around.
    SearchWrapped,
    /// After loading a session file.
    SessionLoadPost,
    /// After writing a session file.
    SessionWritePost,
    /// After ":!cmd".
    ShellCmdPost,
    /// After ":1,2!cmd", ":w !cmd", ":r !cmd".
    ShellFilterPost,
    /// After nvim process received a signal.
    Signal,
    /// Sourcing a Vim script using command.
    SourceCmd,
    /// After sourcing a Vim script.
    SourcePost,
    /// Before sourcing a Vim script.
    SourcePre,
    /// Spell file missing.
    SpellFileMissing,
    /// After reading from stdin.
    StdinReadPost,
    /// Before reading from stdin.
    StdinReadPre,
    /// Found existing swap file.
    SwapExists,
    /// Syntax selected.
    Syntax,
    /// After a tab has closed.
    TabClosed,
    /// After entering a tab page.
    TabEnter,
    /// Before leaving a tab page.
    TabLeave,
    /// When creating a new tab.
    TabNew,
    /// After entering a new tab.
    TabNewEntered,
    /// After changing 'term'.
    TermChanged,
    /// After the process exits.
    TermClose,
    /// After entering Terminal mode.
    TermEnter,
    /// After leaving Terminal mode.
    TermLeave,
    /// After opening a terminal buffer.
    TermOpen,
    /// After an unhandled OSC sequence is emitted.
    TermRequest,
    /// After setting "v:termresponse".
    TermResponse,
    /// Text was modified.
    TextChanged,
    /// Text was modified in Insert mode(no popup).
    TextChangedI,
    /// Text was modified in Insert mode(popup).
    TextChangedP,
    /// Text was modified in Terminal mode.
    TextChangedT,
    /// After a yank or delete was done (y, d, c).
    TextYankPost,
    /// After UI attaches.
    UIEnter,
    /// After UI detaches.
    UILeave,
    /// User defined autocommand.
    User,
    /// After starting Vim.
    VimEnter,
    /// Before exiting Vim.
    VimLeave,
    /// Before exiting Vim and writing ShaDa file.
    VimLeavePre,
    /// After Vim window was resized.
    VimResized,
    /// After Nvim is resumed.
    VimResume,
    /// Before Nvim is suspended.
    VimSuspend,
    /// After closing a window.
    WinClosed,
    /// After entering a window.
    WinEnter,
    /// Before leaving a window.
    WinLeave,
    /// When entering a new window.
    WinNew,
    /// After a window was resized.
    WinResized,
    /// After a window was scrolled or resized.
    WinScrolled,
}

impl Display for AutoCmdEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format!("{self:?}"))
    }
}

#[derive(Debug, Default)]
pub struct AutoCmdOpts<'a> {
    /// Autocommand group name or id to match against.
    pub group: Option<AutoCmdGroup>,

    /// Optional: pattern(s) to match literally |autocmd-pattern|.
    pub pattern: Vec<String>,

    /// Optional: buffer number for buffer-local
    /// autocommands |autocmd-buflocal|. Cannot be used with {pattern}
    pub buffer: Option<u32>,

    /// description (for documentation and troubleshooting).
    pub desc: Option<String>,

    /**
    Lua function called when the event(s) is triggered.
    Lua callback can return a truthy value (not `false` or `nil`) to delete the autocommand.
    Receives a table argument with these keys:
    • id: (number) autocommand id
    • event: (string) name of the triggered event |autocmd-events|
    • group: (number|nil) autocommand group id, if any
    • match: (string) expanded value of <amatch>
    • buf: (number) expanded value of <abuf>
    • file: (string) expanded value of <afile>
    • data: (any) arbitrary data passed from |nvim_exec_autocmds()|
     */
    pub callback: Option<LuaFunction<'a>>,

    /// defaults to false. Run the autocommand only once |autocmd-once|.
    pub once: bool,
}

impl<'a> IntoLua<'a> for AutoCmdOpts<'a> {
    fn into_lua(self, lua: &'a Lua) -> LuaResult<LuaValue<'a>> {
        let table = lua.create_table()?;

        match self.group {
            Some(AutoCmdGroup::String(name)) => table.set("group", name)?,
            Some(AutoCmdGroup::Integer(id)) => table.set("group", id)?,
            None => {}
        };

        if !self.pattern.is_empty() {
            table.set("pattern", self.pattern)?;
        }

        if let Some(buf_id) = self.buffer {
            table.set("buffer", buf_id)?;
        }

        if let Some(desc) = self.desc {
            table.set("desc", desc)?;
        }

        if let Some(cb) = self.callback {
            table.set("callback", cb)?;
        }

        table.set("once", self.once)?;

        Ok(LuaValue::Table(table))
    }
}
