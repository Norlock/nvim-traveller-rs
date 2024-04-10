use nvim_oxi::{
    api::{
        self,
        opts::{BufAttachOpts, BufDeleteOpts},
        types::Mode,
        Buffer, LuaApi, StdpathType, Window,
    },
    lua::Error,
    mlua::{self, Lua, Result, Table},
};
use std::{path::PathBuf, sync::Arc};
use tokio::sync::RwLock;

use crate::CONTAINER;

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
}

unsafe impl Send for AppState {}
unsafe impl Sync for AppState {}

#[derive(Clone)]
pub struct AppContainer(pub Arc<RwLock<AppState>>);

impl AppContainer {
    pub fn dummy() -> Self {
        let dummy_state = AppState {
            buf: Buffer::from(0),
            win: Window::from(0),
            history: vec![],
            selection: vec![],
            buf_content: vec![],
            show_hidden: false,
            cwd: PathBuf::from("/tmp"),
            history_dir: PathBuf::from("/tmp"),
        };

        Self(Arc::new(RwLock::new(dummy_state)))
    }
}

impl AppState {
    pub fn new(lua: &Lua) -> nvim_oxi::Result<Self> {
        let mut buf = Self::create_nav_buf()?;
        buf.set_option("bufhidden", "wipe")?;

        Ok(Self {
            show_hidden: false,
            history: vec![],
            selection: vec![],
            buf_content: vec![],
            cwd: LuaApi::get_cwd(lua)?,
            history_dir: LuaApi::stdpath(lua, StdpathType::State)?,
            win: api::get_current_win(),
            buf,
        })
    }

    pub fn create_nav_buf() -> nvim_oxi::Result<Buffer> {
        Ok(api::create_buf(false, true)?)
    }

    pub fn set_buf_name_navigator(lua: &Lua) -> nvim_oxi::Result<()> {
        let lfn: mlua::Function = lua.load("vim.cmd.file").eval()?;

        Ok(lfn.call::<&str, ()>("Traveller")?)
    }

    pub fn set_keymap<'a>(
        lua: &'a Lua,
        mode: Mode,
        lhs: &'a str,
        rhs: mlua::Function,
        keymap_opts: Table<'a>,
    ) -> nvim_oxi::Result<()> {
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

    pub fn open_navigation(&mut self, lua: &Lua) -> nvim_oxi::Result<()> {
        self.buf = Self::create_nav_buf()?;
        self.buf.set_option("bufhidden", "wipe")?;
        self.cwd = LuaApi::get_cwd(lua)?;
        self.history_dir = LuaApi::stdpath(lua, StdpathType::State)?;
        self.win = api::get_current_win();

        api::set_current_buf(&self.buf)?;

        // Display in bar below
        Self::set_buf_name_navigator(lua)?;

        let km_opts = LuaApi::buf_keymap_opts(lua, true, self.buf.bufnr())?;

        LuaApi::set_keymap(
            lua,
            Mode::Normal,
            "q",
            lua.create_function(Self::close_nav)?,
            km_opts,
        )?;

        Ok(())
    }

    fn close_nav(lua: &Lua, _: ()) -> mlua::prelude::LuaResult<()> {
        let lfn: mlua::Function = lua.load("vim.cmd.e").eval()?;

        Ok(lfn.call::<&str, ()>("#")?)
    }

    pub fn close_navigation(&self, lua: &Lua) -> nvim_oxi::Result<()> {
        // TODO git root
        nvim_oxi::lua::print!("Test 323");
        let lfn: mlua::Function = lua.load("vim.cmd.e").eval()?;
        //let del_buf = self.buf.clone();

        //del_buf
        //.delete(&BufDeleteOpts::default())
        //.unwrap();

        //Ok(())
        Ok(lfn.call::<&str, ()>("#")?)
    }
}
