use neo_api_rs::mlua;
use neo_api_rs::mlua::prelude::*;
use neo_api_rs::*;
use once_cell::sync::Lazy;
use state::AppState;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::OnceLock;
use theme::Theme;
use utils::Utils;

mod popup;
mod state;
mod theme;
mod utils;

static CONTAINER: Lazy<Mutex<AppState>> = Lazy::new(|| {
    let app = AppState {
        history_dir: PathBuf::new(),
        theme: Theme::default(),
        active_instance_idx: 0,
        instances: HashMap::new(),
    };

    Mutex::new(app)
});

static CB_QUEUE: OnceLock<Mutex<CallBackQueue<AppState>>> = neo_api_rs::create_callback_container();

#[mlua::lua_module]
fn nvim_traveller_rs(lua: &Lua) -> LuaResult<LuaTable> {
    CB_QUEUE.init();

    let module = lua.create_table()?;

    let mut app = CONTAINER.lock().unwrap();

    if let Err(err) = app.init(lua) {
        NeoApi::notify(lua, &err)?;
    }

    module.set(
        "open_navigation",
        lua.create_async_function(open_navigation)?,
    )?;

    module.set(
        "directory_search",
        lua.create_async_function(directory_search)?,
    )?;

    Ok(module)
}

async fn open_navigation(lua: &Lua, _: ()) -> LuaResult<()> {
    let mut app = CONTAINER.lock().unwrap();

    let mut started_from = NeoApi::get_filepath(lua)?;

    if !started_from.is_file() {
        started_from = NeoApi::get_filedir(lua)?;
    }

    if let Err(err) = app.open_navigation(&lua, started_from) {
        NeoApi::notify(&lua, &err)?;
    }

    Ok(())
}

pub struct FuzzyVisitor;

impl FuzzyConfig for FuzzyVisitor {
    fn cwd(&self, _lua: &Lua) -> PathBuf {
        Utils::home_directory()
    }

    fn search_type(&self) -> FilesSearch {
        FilesSearch::DirOnly
    }

    fn on_enter(&self, lua: &Lua, selected: PathBuf) -> LuaResult<()> {
        let mut app = CONTAINER.lock().unwrap();

        if let Err(err) = app.open_navigation(&lua, selected) {
            NeoApi::notify(&lua, &err)?;
        }

        Ok(())
    }
}

async fn directory_search(lua: &Lua, _: ()) -> LuaResult<()> {
    if let Err(err) = NeoFuzzy::files(lua, Arc::new(FuzzyVisitor)).await {
        NeoApi::notify(&lua, &err)?;
    }

    Ok(())
}
