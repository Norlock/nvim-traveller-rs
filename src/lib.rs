use neo_api_rs::mlua;
use neo_api_rs::mlua::prelude::*;
use neo_api_rs::*;
use once_cell::sync::Lazy;
use state::AppState;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use theme::Theme;
use utils::Utils;

mod popup;
mod state;
mod theme;
mod utils;

static CONTAINER: Lazy<AppState> = Lazy::new(|| AppState {
    history_dir: PathBuf::new().into(),
    theme: Theme::default().into(),
    active_instance_idx: 0.into(),
    instances: HashMap::new().into(),
});

#[mlua::lua_module]
fn nvim_traveller_rs(lua: &Lua) -> LuaResult<LuaTable> {
    let module = lua.create_table()?;

    if let Err(err) = AppState::init(lua) {
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
    let mut started_from = NeoApi::get_filepath(lua)?;

    if !started_from.is_file() {
        started_from = NeoApi::get_filedir(lua)?;
    }

    if let Err(err) = AppState::open_navigation(&lua, started_from).await {
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

    fn on_enter(&self, lua: &Lua, selected: PathBuf) {
        RTM.block_on(async move {
            if let Err(err) = AppState::open_navigation(&lua, selected).await {
                let _ = NeoApi::notify(&lua, &err);
            }
        })
    }

    //async fn on_enter(&self, lua: &Lua, selected: PathBuf) -> Result<()> {
    //if let Err(err) = AppState::open_navigation(&lua, selected).await {
    //NeoApi::notify(&lua, &err);
    //}

    //Ok(())
    //}
}

async fn directory_search(lua: &Lua, _: ()) -> LuaResult<()> {
    if let Err(err) = NeoFuzzy::files(lua, Box::new(FuzzyVisitor)).await {
        NeoApi::notify(&lua, &err)?;
    }

    Ok(())
}
