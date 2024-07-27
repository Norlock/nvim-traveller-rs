use fuzzy_config::TravellerFuzzy;
use neo_api_rs::mlua;
use neo_api_rs::mlua::prelude::*;
use neo_api_rs::*;
use once_cell::sync::Lazy;
use state::AppState;
use utils::NeoUtils;
use std::collections::HashMap;
use theme::Theme;

mod popup;
mod state;
mod theme;
mod utils;
mod fuzzy_config;

static CONTAINER: Lazy<AppState> = Lazy::new(|| AppState {
    theme: Theme::default().into(),
    active_buf: 0.into(),
    instances: HashMap::new().into(),
    selection: HashMap::new().into(),
});

#[mlua::lua_module]
fn nvim_traveller_rs(lua: &Lua) -> LuaResult<LuaTable> {
    NeoApi::init(lua)?;

    if let Err(err) = AppState::init(lua) {
        NeoApi::notify(lua, &err)?;
    }

    let module = lua.create_table()?;

    module.set(
        "open_navigation",
        lua.create_async_function(open_navigation)?,
    )?;

    module.set(
        "directory_search",
        lua.create_async_function(directory_search)?,
    )?;

    module.set("file_search", lua.create_async_function(file_search)?)?;
    module.set("git_file_search", lua.create_async_function(git_file_search)?)?;
    module.set("buffer_search", lua.create_async_function(buffer_search)?)?;

    Ok(module)
}

async fn open_navigation(lua: &Lua, _: ()) -> LuaResult<()> {
    let mut started_from = NeoApi::get_filepath(lua)?;

    if !started_from.is_file() {
        started_from = started_from.parent().unwrap().to_path_buf();
    }

    if let Err(err) = AppState::open_navigation(lua, started_from).await {
        NeoApi::notify(lua, &err)?;
    }

    Ok(())
}

async fn directory_search(lua: &Lua, _: ()) -> LuaResult<()> {
    let home = NeoUtils::home_directory();
    let config = TravellerFuzzy::new(home, FuzzySearch::Directories);

    if let Err(err) = NeoFuzzy::files_or_directories(lua, Box::new(config)).await {
        NeoApi::notify(lua, &err)?;
    }

    Ok(())
}

async fn file_search(lua: &Lua, _: ()) -> LuaResult<()> {
    let cwd = NeoApi::get_cwd(lua)?;
    let config = TravellerFuzzy::new(cwd, FuzzySearch::Files);

    if let Err(err) = NeoFuzzy::files_or_directories(lua, Box::new(config)).await {
        NeoApi::notify(lua, &err)?;
    }

    Ok(())
}

async fn buffer_search(lua: &Lua, _: ()) -> LuaResult<()> {
    let cwd = NeoApi::get_cwd(lua)?;
    let config = TravellerFuzzy::new(cwd, FuzzySearch::Buffer);

    if let Err(err) = NeoFuzzy::files_or_directories(lua, Box::new(config)).await {
        NeoApi::notify(lua, &err)?;
    }

    Ok(())
}

async fn git_file_search(lua: &Lua, _: ()) -> LuaResult<()> {
    let cwd = NeoApi::get_cwd(lua).unwrap();

    let cwd = if let Some(git_root) = NeoUtils::git_root(&cwd) {
        git_root
    } else {
        cwd
    };

    let config = TravellerFuzzy::new(cwd, FuzzySearch::GitFiles);

    if let Err(err) = NeoFuzzy::files_or_directories(lua, Box::new(config)).await {
        NeoApi::notify(lua, &err)?;
    }

    Ok(())
}
