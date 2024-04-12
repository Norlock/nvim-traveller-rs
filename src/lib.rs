use crate::state::AppContainer;
use lua_api::LuaApi;
use mlua::prelude::*;

mod lua_api;
mod lua_api_types;
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

    module.set("open_navigation", lua.create_function(open_navigation)?)?;

    Ok(module)
}

fn open_navigation(lua: &Lua, _: ()) -> LuaResult<()> {
    let mut app = CONTAINER.0.blocking_write();

    app.theme.init(lua)?;

    LuaApi::notify(lua, &"test")?;

    //lua::print!("{app:?}");

    app.open_navigation(lua)
}
