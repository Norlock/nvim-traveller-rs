use crate::state::AppContainer;
use nvim_oxi::{
    lua,
    mlua::{self},
    Dictionary, Function, Object,
};

mod state;
mod lua_api;
mod lua_api_types;
mod theme;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    pub static ref CONTAINER: AppContainer = AppContainer::default();
}

#[nvim_oxi::plugin]
fn nvim_traveller_rs() -> Dictionary {
    let open_navigation_ptr = Function::from_fn(open_navigation);

    Dictionary::from_iter([("open_navigation", Object::from(open_navigation_ptr))])
}

fn open_navigation(_: ()) -> nvim_oxi::Result<()> {
    let lua = mlua::lua();
    let mut app = CONTAINER.0.blocking_write();

    lua::print!("{app:?}");

    app.open_navigation(lua)
}
