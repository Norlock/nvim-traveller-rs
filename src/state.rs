use std::path::PathBuf;

use nvim_oxi::{api::{self, Buffer, Window}, mlua::{self, Lua}};

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

impl AppState {
    pub fn new(lua: &Lua) -> Self {
        Self {
            show_hidden: false,
            history: vec![],
            selection: vec![],
            buf_content: vec![],
            cwd: Self::get_cwd(lua),
            history_dir: Self::get_history_dir(lua),
            win: api::get_current_win(),
            buf: api::get_current_buf(),
        }
    }

    pub fn get_cwd(lua: &Lua) -> PathBuf {
        let cwd_fn: mlua::Function  = lua.load("vim.fn.getcwd").eval().unwrap();
        cwd_fn.call::<(), String>(()).expect("Can't call").into()
    }
    
    pub fn get_history_dir(lua: &Lua) -> PathBuf {
        let stdpath_fn: mlua::Function = lua.load("vim.fn.stdpath").eval().unwrap();
        stdpath_fn.call::<&str, String>("state").unwrap().into()
    }
}
