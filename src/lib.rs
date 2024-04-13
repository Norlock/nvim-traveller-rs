use crate::state::AppContainer;
use mlua::prelude::*;
use neo_api::NeoApi;

mod neo_api;
mod neo_api_types;
mod state;
mod theme;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    pub static ref CONTAINER: AppContainer = AppContainer::default();
}

#[mlua::lua_module]
fn nvim_traveller_rs(lua: &Lua) -> LuaResult<LuaTable> {
    let module = lua.create_table()?;

    let mut app = CONTAINER.0.blocking_write();

    if let Err(err) = app.theme.init(lua) {
        NeoApi::notify(lua, &err)?;
    }

    module.set(
        "open_navigation",
        lua.create_async_function(open_navigation)?,
    )?;

    Ok(module)
}

async fn open_navigation(lua: &Lua, _: ()) -> LuaResult<()> {
    let mut app = CONTAINER.0.write().await;

    if let Err(err) = app.open_navigation(lua) {
        NeoApi::notify(lua, &err)?;
    }

    Ok(())
}
