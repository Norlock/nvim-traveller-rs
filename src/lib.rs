use neo_api_rs::mlua;
use neo_api_rs::mlua::prelude::*;
use neo_api_rs::*;
use once_cell::sync::Lazy;
use state::AppState;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;
use theme::Theme;
use tokio::sync::Mutex;

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

    let mut app = CONTAINER.blocking_lock();

    if let Err(err) = app.init(lua) {
        NeoApi::notify(lua, &err)?;
    }

    module.set(
        "open_navigation",
        lua.create_async_function(open_navigation)?,
    )?;

    Ok(module)
}

async fn open_navigation(lua: &Lua, _: ()) -> LuaResult<()> {
    let mut app = CONTAINER.lock().await;

    if let Err(err) = app.open_navigation(&lua) {
        NeoApi::notify(&lua, &err)?;
    }

    Ok(())
}
