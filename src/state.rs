use crate::neo_api_types::{Buffer, Mode, StdpathType, Window};
use crate::{neo_api::NeoApi, theme::Theme};
use mlua::prelude::*;
use std::{
    fs::{self, DirEntry},
    path::PathBuf,
    sync::Arc,
};
use tokio::sync::RwLock;

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
            buf: Buffer::ZERO,
            win: Window::ZERO,
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
    pub fn set_buf_name_navigator(lua: &Lua) -> LuaResult<()> {
        let lfn: mlua::Function = lua.load("vim.cmd.file").eval()?;

        Ok(lfn.call::<&str, ()>("Traveller")?)
    }

    pub fn open_navigation(&mut self, lua: &Lua) -> LuaResult<()> {
        self.buf = NeoApi::create_buf(lua, false, true)?;

        self.buf.set_option_value(lua, "bufhidden", "wipe")?;
        self.cwd = NeoApi::get_cwd(lua)?;
        self.history_dir = NeoApi::stdpath(lua, StdpathType::State)?;
        self.win = NeoApi::get_current_win(lua)?;

        NeoApi::set_current_buf(lua, self.buf)?;

        // Set buffer content
        self.buf.set_option_value(lua, "modifiable", true)?;
        self.buf_content = nav_buffer_lines(&self.cwd)?;
        NeoApi::buf_set_lines(lua, self.buf.id(), 0, -1, true, self.buf_content.clone())?;
        self.buf.set_option_value(lua, "modifiable", false)?;

        self.theme_nav_buffer(lua)?;

        // Display in bar below
        Self::set_buf_name_navigator(lua)?;

        self.add_keymaps(lua)?;

        Ok(())
    }

    fn add_keymaps(&self, lua: &Lua) -> LuaResult<()> {
        let km_opts = NeoApi::buf_keymap_opts(lua, true, self.buf.id())?;

        let close_navigation = lua.create_function(close_navigation)?;
        NeoApi::set_keymap(lua, Mode::Normal, "q", close_navigation, km_opts)?;

        //NeoApi::set_keymap(
        //lua,
        //Mode::Normal,
        //"h",
        //lua.create_function(Self::close_navigation)?,
        //km_opts,
        //)?;

        Ok(())
    }
}

fn close_navigation(lua: &Lua, _: ()) -> mlua::prelude::LuaResult<()> {
    let lfn: mlua::Function = lua.load("vim.cmd.e").eval()?;

    Ok(lfn.call::<&str, ()>("#")?)
}

fn nav_buffer_lines(path: &PathBuf) -> LuaResult<Vec<String>> {
    let dir = fs::read_dir(path).map_err(|e| LuaError::RuntimeError(e.to_string()))?;

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
