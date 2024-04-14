use crate::neo_api_types::{Buffer, Mode, OpenIn, StdpathType, Window};
use crate::CONTAINER;
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

        NeoApi::set_current_buf(lua, self.buf.id())?;

        self.set_buffer_content(lua)?;

        // Display in bar below
        Self::set_buf_name_navigator(lua)?;

        self.add_keymaps(lua)?;

        Ok(())
    }

    fn add_keymaps(&self, lua: &Lua) -> LuaResult<()> {
        let km_opts = NeoApi::buf_keymap_opts(lua, true, self.buf.id())?;

        let close_navigation = lua.create_function(close_navigation)?;
        NeoApi::set_keymap(lua, Mode::Normal, "q", close_navigation, km_opts.clone())?;

        let nav_to_parent = lua.create_async_function(navigate_to_parent)?;
        NeoApi::set_keymap(lua, Mode::Normal, "h", nav_to_parent, km_opts.clone())?;

        let action_on_item = lua.create_async_function(item_action_on_buffer)?;
        NeoApi::set_keymap(lua, Mode::Normal, "l", action_on_item, km_opts)?;

        Ok(())
    }

    fn set_buffer_content(&mut self, lua: &Lua) -> LuaResult<()> {
        self.buf.set_option_value(lua, "modifiable", true)?;
        self.buf_content = nav_buffer_lines(&self.cwd)?;
        NeoApi::buf_set_lines(lua, self.buf.id(), 0, -1, true, self.buf_content.clone())?;
        self.buf.set_option_value(lua, "modifiable", false)?;

        self.theme_nav_buffer(lua)

        // TODO set window cursor
    }
}

async fn navigate_to_parent(lua: &Lua, _: ()) -> LuaResult<()> {
    let mut app = CONTAINER.0.write().await;

    app.cwd.pop();
    app.set_buffer_content(lua)
}

async fn item_action_on_buffer(lua: &Lua, _: ()) -> LuaResult<()> {
    item_action(lua, OpenIn::Buffer).await
}

async fn item_action_on_tab(lua: &Lua, _: ()) -> LuaResult<()> {
    item_action(lua, OpenIn::Tab).await
}

async fn item_action_on_v_split(lua: &Lua, _: ()) -> LuaResult<()> {
    item_action(lua, OpenIn::VSplit).await
}

async fn item_action_on_h_split(lua: &Lua, _: ()) -> LuaResult<()> {
    item_action(lua, OpenIn::HSplit).await
}

async fn item_action(lua: &Lua, open_in: OpenIn) -> LuaResult<()> {
    let mut app = CONTAINER.0.write().await;

    let cursor = NeoApi::win_get_cursor(lua, 0)?;

    let item = app
        .buf_content
        .get(cursor.row_zero_indexed() as usize)
        .map(|item| item.to_string());

    NeoApi::notify(lua, &cursor)?;

    if item.is_none() {
        // Empty directory
        return Ok(());
    }

    let item = item.unwrap();

    if item.ends_with("/") {
        app.cwd.push(item);
        app.set_buffer_content(lua)
    } else {
        // Set git root.
        NeoApi::open_file(lua, open_in, &item)
    }
}

fn close_navigation(lua: &Lua, _: ()) -> LuaResult<()> {
    NeoApi::buf_delete(lua, 0, None)
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
