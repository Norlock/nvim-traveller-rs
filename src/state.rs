use crate::neo_api_types::{Buffer, Mode, OpenIn, StdpathType, WinCursor, Window};
use crate::utils::Utils;
use crate::CONTAINER;
use crate::{neo_api::NeoApi, theme::Theme};
use mlua::prelude::*;
use std::cmp::Ordering;
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

impl Location {
    pub fn new(dir_path: PathBuf, item: String) -> Self {
        Self { dir_path, item }
    }
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
            cwd: PathBuf::new(),
            history_dir: PathBuf::new(),
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
        NeoApi::set_keymap(
            lua,
            Mode::Normal,
            "q",
            close_navigation.clone(),
            km_opts.clone(),
        )?;
        NeoApi::set_keymap(
            lua,
            Mode::Normal,
            "<Esc>",
            close_navigation,
            km_opts.clone(),
        )?;

        let nav_to_parent = lua.create_async_function(navigate_to_parent)?;
        NeoApi::set_keymap(
            lua,
            Mode::Normal,
            "h",
            nav_to_parent.clone(),
            km_opts.clone(),
        )?;
        NeoApi::set_keymap(lua, Mode::Normal, "<Left>", nav_to_parent, km_opts.clone())?;

        let open_item_in_buffer = lua.create_async_function(open_item_in_buffer)?;
        NeoApi::set_keymap(
            lua,
            Mode::Normal,
            "l",
            open_item_in_buffer.clone(),
            km_opts.clone(),
        )?;
        NeoApi::set_keymap(
            lua,
            Mode::Normal,
            "<Cr>",
            open_item_in_buffer.clone(),
            km_opts.clone(),
        )?;
        NeoApi::set_keymap(
            lua,
            Mode::Normal,
            "<Right>",
            open_item_in_buffer,
            km_opts.clone(),
        )?;

        let toggle_hidden = lua.create_async_function(toggle_hidden)?;
        NeoApi::set_keymap(lua, Mode::Normal, ".", toggle_hidden, km_opts)?;

        Ok(())
    }

    fn set_buffer_content(&mut self, lua: &Lua) -> LuaResult<()> {
        NeoApi::set_cwd(lua, &self.cwd)?;

        self.buf.set_option_value(lua, "modifiable", true)?;
        self.buf_content = nav_buffer_lines(&self.cwd, self.show_hidden)?;
        self.buf
            .set_lines(lua, 0, -1, true, self.buf_content.clone())?;
        self.buf.set_option_value(lua, "modifiable", false)?;

        self.theme_nav_buffer(lua)?;

        if let Some(location) = self.history.iter().find(|loc| &loc.dir_path == &self.cwd) {
            for (row, item) in self.buf_content.iter().enumerate() {
                if &location.item == item {
                    let cursor = WinCursor::from_zero_indexed(row as u32, 0);
                    self.win.set_cursor(lua, cursor)?;
                    break;
                }
            }
        } else {
            let cursor = WinCursor::from_zero_indexed(0, 0);
            self.win.set_cursor(lua, cursor)?;
        }

        Ok(())
    }

    fn get_location<'a>(&'a mut self) -> Option<&'a mut Location> {
        self.history
            .iter_mut()
            .find(|his| &his.dir_path == &self.cwd)
    }

    fn update_history(&mut self, item: String) {
        if let Some(location) = self.get_location() {
            location.item = item;
            return;
        }

        let dir_path = self.cwd.clone();
        self.history.push(Location::new(dir_path, item));
    }
}

async fn toggle_hidden(lua: &Lua, _: ()) -> LuaResult<()> {
    let mut app = CONTAINER.0.write().await;

    app.show_hidden = !app.show_hidden;
    app.set_buffer_content(lua)
}

async fn navigate_to_parent(lua: &Lua, _: ()) -> LuaResult<()> {
    let mut app = CONTAINER.0.write().await;

    let cursor = NeoApi::win_get_cursor(lua, 0)?;
    let item = app.buf_content[cursor.row_zero_indexed() as usize].clone();

    app.update_history(item);

    // Before navigating to parent add to history to the parent directory already knows to which it
    // needs to point its cursor
    let item = app.cwd.file_name().unwrap().to_string_lossy().to_string();
    app.cwd.pop();
    app.update_history(item);

    app.set_buffer_content(lua)
}

async fn open_item_in_buffer(lua: &Lua, _: ()) -> LuaResult<()> {
    open_item(lua, OpenIn::Buffer).await
}

async fn open_item_in_tab(lua: &Lua, _: ()) -> LuaResult<()> {
    open_item(lua, OpenIn::Tab).await
}

async fn open_item_in_vsplit(lua: &Lua, _: ()) -> LuaResult<()> {
    open_item(lua, OpenIn::VSplit).await
}

async fn open_item_in_hsplit(lua: &Lua, _: ()) -> LuaResult<()> {
    open_item(lua, OpenIn::HSplit).await
}

async fn open_item(lua: &Lua, open_in: OpenIn) -> LuaResult<()> {
    let mut app = CONTAINER.0.write().await;

    let cursor = NeoApi::win_get_cursor(lua, 0)?;

    let item = app
        .buf_content
        .get(cursor.row_zero_indexed() as usize)
        .map(|item| item.to_string());

    if item.is_none() {
        // Empty directory
        return Ok(());
    }

    let item = item.unwrap();

    if item.ends_with("/") {
        app.update_history(item.to_string());
        app.cwd.push(item);
        app.set_buffer_content(lua)
    } else {
        NeoApi::open_file(lua, open_in, &item)?;

        if let Some(git_root) = Utils::git_root(&app.cwd) {
            NeoApi::set_cwd(lua, &git_root)?;
        }

        Ok(())
    }
}

fn close_navigation(lua: &Lua, _: ()) -> LuaResult<()> {
    NeoApi::buf_delete(lua, 0, None)?;

    let path = NeoApi::get_filepath(lua)?;
    NeoApi::notify(lua, &path)?;

    if let Some(git_root) = Utils::git_root(&path) {
        NeoApi::set_cwd(lua, &git_root)?;
    }

    Ok(())
}

fn nav_buffer_lines(path: &PathBuf, show_hidden: bool) -> LuaResult<Vec<String>> {
    let dir = fs::read_dir(path).map_err(LuaError::external)?;

    let mut paths: Vec<_> = dir
        .map(|item| item.unwrap())
        .filter(|path| {
            let hidden_file = path
                .file_name()
                .to_str()
                .map(|file| file.starts_with("."))
                .unwrap_or(false);

            (hidden_file && show_hidden) || !hidden_file
        })
        .collect();

    //paths.sort_by_key(|dir| dir.path());
    paths.sort_by(|a, b| {
        let met_a = a.metadata().unwrap();
        let met_b = b.metadata().unwrap();

        if met_a.is_dir() == met_b.is_dir() {
            a.file_name().cmp(&b.file_name())
        } else if met_a.is_dir() {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    });

    let mut lines = vec![];

    for entry in paths {
        append_item(entry, &mut lines);
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
