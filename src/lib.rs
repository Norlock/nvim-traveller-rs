use crate::state::AppContainer;
use neo_api::NvApi;
use mlua::prelude::*;

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

    module.set("open_navigation", lua.create_async_function(open_navigation)?)?;

    Ok(module)
}

async fn open_navigation(lua: &Lua, _: ()) -> LuaResult<()> {
    let mut app = CONTAINER.0.write().await;

    app.theme.init(lua)?;

    NvApi::notify(lua, &app)?;

    app.open_navigation(lua)
}
