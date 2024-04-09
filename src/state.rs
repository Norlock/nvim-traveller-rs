use std::path::PathBuf;

use nvim_oxi::{
    api::{self, opts::CmdOpts, types::CmdInfos, Buffer, Window},
    mlua::{self, Lua},
};

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
    pub fn new(lua: &Lua) -> nvim_oxi::Result<Self> {
        let mut buf = api::create_buf(false, true).unwrap();
        buf.set_option("bufhidden", "wipe")?;

        Ok(Self {
            show_hidden: false,
            history: vec![],
            selection: vec![],
            buf_content: vec![],
            cwd: Self::get_cwd(lua)?,
            history_dir: Self::get_history_dir(lua)?,
            win: api::get_current_win(),
            buf,
        })
    }

    pub fn get_cwd(lua: &Lua) -> nvim_oxi::Result<PathBuf> {
        let lfn: mlua::Function = lua.load("vim.fn.getcwd").eval()?;

        Ok(lfn.call::<(), String>(())?.into())
    }

    pub fn get_history_dir(lua: &Lua) -> nvim_oxi::Result<PathBuf> {
        let lfn: mlua::Function = lua.load("vim.fn.stdpath").eval()?;

        Ok(lfn.call::<&str, String>("state")?.into())
    }

    pub fn set_buf_name_navigator(lua: &Lua) -> nvim_oxi::Result<()> {
        let lfn: mlua::Function = lua.load("vim.cmd").eval()?;

        Ok(lfn.call::<&str, ()>("file Traveller")?)
    }

    pub fn open_navigation(&mut self, lua: &Lua) -> nvim_oxi::Result<()> {
        api::set_current_buf(&self.buf)?;

        // Display in bar below
        api::cmd(
            &CmdInfos::builder()
                .cmd("echo")
                .args(vec!["123"])
                .bang(false)
                .build(),
            &CmdOpts::default(),
        )?;

        Ok(())
    }
}
