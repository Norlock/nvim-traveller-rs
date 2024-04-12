use nvim_oxi::{
    api::{self, types::Mode, Buffer, Window},
    mlua::{self, Lua, Table},
};
use std::{
    fs::{self, DirEntry},
    path::PathBuf,
    sync::Arc,
};
use tokio::sync::RwLock;

use crate::CONTAINER;
use crate::{lua_api::*, theme::Theme};

#[derive(Debug)]
pub struct Location {
    pub dir_path: PathBuf,
    pub item: String,
}

#[derive(Debug)]
pub struct AppState {
    pub show_hidden: bool,
    pub history: Vec<Location>,
    pub selection: Vec<Location>,
    pub buf_content: Vec<String>,
    pub cwd: PathBuf,
    pub history_dir: PathBuf,
    pub win: Window,
    pub buf: Buffer,
    pub theme: Theme,
}

unsafe impl Send for AppState {}
unsafe impl Sync for AppState {}

#[derive(Clone)]
pub struct AppContainer(pub Arc<RwLock<AppState>>);

impl Default for AppContainer {
    fn default() -> Self {
        let app = AppState {
            buf: Buffer::from(0),
            win: Window::from(0),
            history: vec![],
            selection: vec![],
            buf_content: vec![],
            show_hidden: false,
            cwd: PathBuf::from("/tmp"),
            history_dir: PathBuf::from("/tmp"),
            theme: Theme::default(),
        };

        Self(Arc::new(RwLock::new(app)))
    }
}

impl AppState {
    pub fn create_nav_buf() -> nvim_oxi::Result<Buffer> {
        Ok(api::create_buf(false, true)?)
    }

    pub fn set_buf_name_navigator(lua: &Lua) -> nvim_oxi::Result<()> {
        let lfn: mlua::Function = lua.load("vim.cmd.file").eval()?;

        Ok(lfn.call::<&str, ()>("Traveller")?)
    }

    pub fn open_navigation(&mut self, lua: &Lua) -> nvim_oxi::Result<()> {
        self.buf = Self::create_nav_buf()?;
        self.buf.set_option("bufhidden", "wipe")?;
        self.cwd = LuaApi::get_cwd(lua)?;
        self.history_dir = LuaApi::stdpath(lua, StdpathType::State)?;
        self.win = api::get_current_win();

        api::set_current_buf(&self.buf)?;

        // Set buffer content
        self.buf.set_option("modifiable", true)?;
        self.buf_content = nav_buffer_lines(&self.cwd)?;
        LuaApi::buf_set_lines(lua, self.buf.bufnr(), 0, -1, true, self.buf_content.clone())?;
        self.buf.set_option("modifiable", false)?;

        self.theme_nav_buffer(lua)?;

        // Display in bar below
        Self::set_buf_name_navigator(lua)?;

        let km_opts = LuaApi::buf_keymap_opts(lua, true, self.buf.bufnr())?;

        LuaApi::set_keymap(
            lua,
            Mode::Normal,
            "q",
            lua.create_function(Self::close_navigation)?,
            km_opts,
        )?;

        Ok(())
    }

    fn close_navigation(lua: &Lua, _: ()) -> mlua::prelude::LuaResult<()> {
        let lfn: mlua::Function = lua.load("vim.cmd.e").eval()?;

        Ok(lfn.call::<&str, ()>("#")?)
    }
}

fn nav_buffer_lines(path: &PathBuf) -> nvim_oxi::Result<Vec<String>> {
    let dir =
        fs::read_dir(path).map_err(|e| nvim_oxi::Error::Api(api::Error::Other(e.to_string())))?;

    let mut lines = vec![];

    for item in dir {
        if let Ok(entry) = item {
            append_item(entry, &mut lines);
        }
    }

    Ok(lines)
}

fn append_item(entry: DirEntry, lines: &mut Vec<String>) {
    if let Ok(file_type) = entry.file_type() {
        let name = entry.file_name().to_string_lossy().to_string();

        if file_type.is_dir() {
            lines.push(format!("{name}/"));
        } else {
            lines.push(name);
        }
    }
}
