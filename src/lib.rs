use crate::state::AppContainer;
use nvim_oxi::{
    lua,
    mlua::{self},
    Dictionary, Function, Object,
};

pub mod state;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    pub static ref CONTAINER: AppContainer = AppContainer::dummy();
}

#[nvim_oxi::plugin]
fn nvim_traveller_rs() -> Dictionary {
    //api::set_keymap(Mode::Insert, "hi", "hello", &Default::default()).unwrap();
    //api::set_keymap(
    //Mode::Normal,
    //"hi",
    //r#"require("nvim_traveller_rs").stdpath()"#,
    //&Default::default(),
    //)
    //.unwrap();

    //if let Err(err) = container {
    //lua::print!("Error: {err:?}");
    //return Dictionary::default();
    //}

    //let open_navigation = Function::from_fn_mut::<_, nvim_oxi::Error>(move |()| {
    //let lua = mlua::lua();
    //let mut app = c_open.0.blocking_write();

    //lua::print!("{app:?}");

    //let _ = app.open_navigation(lua);

    //Ok(())
    //});

    let open_navigation_ptr = Function::from_fn(open_navigation);
    let close_navigation_ptr = Function::from_fn(close_navigation);

    Dictionary::from_iter([
        ("open_navigation", Object::from(open_navigation_ptr)),
        ("close_navigation", Object::from(close_navigation_ptr)),
    ])
}

fn open_navigation(_: ()) -> nvim_oxi::Result<()> {
    let lua = mlua::lua();
    let mut app = CONTAINER.0.blocking_write();

    lua::print!("{app:?}");

    app.open_navigation(lua)
}

fn close_navigation(_: ()) -> nvim_oxi::Result<()> {
    lua::print!("KOMT HIER");
    let lua = mlua::lua();
    let app = CONTAINER.0.blocking_read();

    app.close_navigation(lua)
}
