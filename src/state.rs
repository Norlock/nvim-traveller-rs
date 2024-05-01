use crate::theme::Theme;
use crate::utils::Utils;
use crate::{popup, CONTAINER};
use neo_api_rs::mlua::prelude::*;
use neo_api_rs::prelude::NuiApi;
use neo_api_rs::prelude::*;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::{
    fs::{self, DirEntry},
    path::PathBuf,
};

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
    pub history_dir: PathBuf,
    pub theme: Theme,
    pub active_instance_idx: u32,
    pub instances: HashMap<u32, AppInstance>,
}

#[derive(Debug)]
pub struct AppInstance {
    pub win: NeoWindow,
    pub buf: NeoBuffer,
    pub show_hidden: bool,
    pub history: Vec<Location>,
    pub selection: Vec<Location>,
    pub buf_content: Vec<String>,
    pub cwd: PathBuf,
    /// This is where traveller needs to return when quiting manually
    pub started_from: PathBuf,
}

unsafe impl Send for AppState {}
unsafe impl Sync for AppState {}

impl AppState {
    pub fn init(&mut self, lua: &Lua) -> LuaResult<()> {
        self.history_dir = NeoApi::stdpath(lua, StdpathType::State)?;
        NuiApi::init(lua)?;

        self.theme.init(lua)
    }

    pub fn active_instance(&mut self) -> &mut AppInstance {
        self.instances.get_mut(&self.active_instance_idx).unwrap()
    }

    pub fn active_instance_ref<'a>(&'a self) -> &'a AppInstance {
        self.instances.get(&self.active_instance_idx).unwrap()
    }

    pub fn set_active_instance<'a>(&'a mut self, idx: u32) -> &'a mut AppInstance {
        self.active_instance_idx = idx;
        self.instances.get_mut(&idx).unwrap()
    }

    pub fn set_buffer_content(&mut self, lua: &Lua) -> LuaResult<()> {
        let instance = self.instances.get_mut(&self.active_instance_idx).unwrap();
        instance.set_buffer_content(&self.theme, lua)
    }

    pub fn open_navigation(&mut self, lua: &Lua) -> LuaResult<()> {
        let buf = NeoApi::create_buf(lua, false, true)?;
        buf.set_option_value(lua, "bufhidden", "wipe")?;
        let win = NeoApi::get_current_win(lua)?;

        let started_from = NeoApi::get_filepath(lua)?;
        let filename = started_from
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();

        let cwd = started_from.parent().unwrap().to_path_buf();

        buf.set_current(lua)?;
        let buf_id = buf.id();

        let mut instance = AppInstance {
            buf,
            win,
            show_hidden: false,
            history: vec![],
            selection: vec![],
            buf_content: vec![],
            cwd,
            started_from,
        };

        instance.update_history(filename);
        instance.add_keymaps(lua)?;
        instance.set_buffer_content(&self.theme, lua)?;

        self.instances.insert(buf_id, instance);
        self.active_instance_idx = buf_id;

        // Display in bar below
        NeoApi::set_cmd_file(lua, format!("Traveller ({buf_id})"))?;

        // Auto commands
        let buf_enter_aucmd = AutoCmdOpts {
            buffer: Some(buf_id),
            callback: lua.create_async_function(buf_enter_callback)?,
            pattern: vec![],
            group: None,
            desc: None,
            once: false,
        };

        NeoApi::create_autocmd(lua, &[AutoCmdEvent::BufEnter], buf_enter_aucmd)?;

        let buf_hidden_aucmd = AutoCmdOpts {
            buffer: Some(buf_id),
            callback: lua.create_async_function(buf_wipeout_callback)?,
            pattern: vec![],
            group: None,
            desc: None,
            once: true,
        };

        NeoApi::create_autocmd(lua, &[AutoCmdEvent::BufWipeout], buf_hidden_aucmd)?;

        Ok(())
    }
}

impl AppInstance {
    fn add_keymaps(&self, lua: &Lua) -> LuaResult<()> {
        let km_opts = self.buf.keymap_opts(true);

        let close_nav = lua.create_async_function(close_navigation)?;
        NeoApi::set_keymap(lua, Mode::Normal, "q", close_nav, km_opts.clone())?;

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

        let open_in_tab = lua.create_async_function(open_item_in_tab)?;
        NeoApi::set_keymap(lua, Mode::Normal, "t", open_in_tab, km_opts)?;

        let open_in_hsplit = lua.create_async_function(open_item_in_hsplit)?;
        NeoApi::set_keymap(lua, Mode::Normal, "s", open_in_hsplit, km_opts)?;

        let open_in_vsplit = lua.create_async_function(open_item_in_vsplit)?;
        NeoApi::set_keymap(lua, Mode::Normal, "v", open_in_vsplit, km_opts)?;

        let toggle_hidden = lua.create_async_function(toggle_hidden)?;
        NeoApi::set_keymap(lua, Mode::Normal, ".", toggle_hidden, km_opts)?;

        let create_items = lua.create_async_function(popup::create_items_popup)?;
        NeoApi::set_keymap(lua, Mode::Normal, "c", create_items, km_opts)?;

        let delete_items = lua.create_async_function(popup::delete_items_popup)?;
        NeoApi::set_keymap(lua, Mode::Normal, "dd", delete_items, km_opts)?;

        Ok(())
    }

    pub fn set_buffer_content<'a>(&'a mut self, theme: &'a Theme, lua: &Lua) -> LuaResult<()> {
        NeoApi::set_cwd(lua, &self.cwd)?;

        self.buf.set_option_value(lua, "modifiable", true)?;
        self.buf_content = nav_buffer_lines(&self.cwd, self.show_hidden)?;
        self.buf.set_lines(lua, 0, -1, true, &self.buf_content)?;
        self.buf.set_option_value(lua, "modifiable", false)?;

        self.theme_nav_buffer(*theme, lua)?;
        self.set_nav_cursor(lua)?;

        Ok(())
    }

    fn set_nav_cursor(&mut self, lua: &Lua) -> LuaResult<()> {
        if let Some(location) = self.history.iter().find(|loc| &loc.dir_path == &self.cwd) {
            for (row, item) in self.buf_content.iter().enumerate() {
                if &location.item == item {
                    let cursor = WinCursor::from_zero_indexed(row as u32, 0);
                    return self.win.set_cursor(lua, cursor.clone());
                }
            }
        }

        if !self.buf_content.is_empty() {
            let cursor = WinCursor::from_zero_indexed(0, 0);
            self.win.set_cursor(lua, cursor.clone())?;
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

async fn buf_enter_callback<'a>(_: &Lua, ev: AutoCmdCbEvent) -> LuaResult<()> {
    fn callback(lua: &Lua, ev: AutoCmdCbEvent) {
        let mut app = CONTAINER.blocking_lock();
        let instance = app.set_active_instance(ev.buf);
        let _ = NeoApi::set_cwd(lua, &instance.cwd);
    }

    CbContainer::add_to_queue(Box::new(callback), ev).await;

    Ok(())
}

async fn buf_wipeout_callback(_: &Lua, ev: AutoCmdCbEvent) -> LuaResult<()> {
    fn callback(lua: &Lua, ev: AutoCmdCbEvent) {
        let mut app = CONTAINER.blocking_lock();
        let _ = NeoApi::notify(lua, &"Successfully called");
        app.instances.remove(&ev.buf);
    }

    CbContainer::add_to_queue(Box::new(callback), ev).await;

    Ok(())
}

async fn toggle_hidden(lua: &Lua, _: ()) -> LuaResult<()> {
    let mut app = CONTAINER.lock().await;

    let theme = app.theme.clone();
    let instance = app.active_instance();
    instance.show_hidden = !instance.show_hidden;
    instance.set_buffer_content(&theme, lua)
}

async fn navigate_to_parent(lua: &Lua, _: ()) -> LuaResult<()> {
    let mut app = CONTAINER.lock().await;
    let theme = app.theme.clone();

    let instance = app.active_instance();

    if instance.cwd.parent().is_none() {
        return Ok(());
    }

    if !instance.buf_content.is_empty() {
        let cursor = NeoWindow::CURRENT.get_cursor(lua)?;
        let item = instance.buf_content[cursor.row_zero_indexed() as usize].clone();

        instance.update_history(item);
    }

    // Before navigating to parent add to history to the parent directory already knows to which it
    // needs to point its cursor
    let item = instance
        .cwd
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();

    instance.cwd.pop();
    instance.update_history(format!("{item}/"));
    instance.set_buffer_content(&theme, lua)
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
    let mut app = CONTAINER.lock().await;

    let theme = app.theme.clone();
    let instance = app.active_instance();

    let cursor = NeoWindow::CURRENT.get_cursor(lua)?;

    let item = instance
        .buf_content
        .get(cursor.row_zero_indexed() as usize)
        .map(|item| item.to_string());

    if item.is_none() {
        // Empty directory
        return Ok(());
    }

    let item = item.unwrap();

    if item.ends_with("/") {
        instance.cwd.push(item.to_string());
        instance.set_buffer_content(&theme, lua)?;
    } else {
        NeoApi::open_file(lua, open_in, &item)?;

        if let Some(git_root) = Utils::git_root(&instance.cwd) {
            NeoApi::set_cwd(lua, &git_root)?;
        }
    }

    CbContainer::exec_drop_lock(app, lua).await;

    Ok(())
}

async fn close_navigation(lua: &Lua, _: ()) -> LuaResult<()> {
    let app = CONTAINER.lock().await;

    let instance = app.active_instance_ref();
    let path = instance.started_from.clone();

    if let Some(git_root) = Utils::git_root(&instance.started_from) {
        NeoApi::set_cwd(lua, &git_root)?;
    }

    NeoApi::open_file(lua, OpenIn::Buffer, path.to_str().unwrap())?;

    CbContainer::exec_drop_lock(app, lua).await;

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
