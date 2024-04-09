use nvim_oxi::{
    lua,
    mlua::{self},
    Dictionary, Function, Object,
};

use crate::state::AppState;

pub mod state;

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

    let lua = mlua::lua();
    let state = AppState::new(lua);
    lua::print!("{state:?}");

    let open_navigation = Function::from_fn::<_, nvim_oxi::Error>(move |()| {
        // TODO serde json state
        lua::print!("open navigation");

        Ok(())
    });

    Dictionary::from_iter([("open_navigation", Object::from(open_navigation))])
}

//fn open_navigation() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        //let result = nvim_traveller_rs();
        //assert_eq!(result, 46);
    }
}
