use neo_api_rs::{
    mlua::{prelude::LuaResult, Lua},
    prelude::{AutoCmdCbEvent, AutoCmdEvent, Mode, NeoApi, NuiAlign, NuiApi, NuiBorder, NuiBorderStyle, NuiBorderText, NuiDimension, NuiPopupOpts, NuiRelative, NuiSize},
};

pub async fn create_items(lua: &Lua, _: ()) -> LuaResult<()> {
    let popup_id = "create_items";

    use NuiSize::*;

    let popup = NuiApi::create_popup(
        lua,
        NuiPopupOpts {
            size: NuiDimension::XorY(Percentage(50), Fixed(1)),
            position: NuiDimension::XandY(Percentage(50)),
            buf_options: None,
            enter: Some(true),
            focusable: None,
            zindex: Some(50),
            relative: Some(NuiRelative::Win),
            border: Some(NuiBorder {
                style: Some(NuiBorderStyle::Rounded),
                padding: None,
                text: Some(NuiBorderText {
                    top: Some("Immaculate".to_string()),
                    top_align: NuiAlign::Center,
                    bottom: None,
                    bottom_align: NuiAlign::Center,
                }),
            }),
            win_options: None,
        },
        &popup_id,
    )?;

    popup.mount(lua)?;

    NeoApi::set_insert_mode(lua, true)?;

    let close_popup_event = lua.create_function(move |lua: &Lua, _: AutoCmdCbEvent| {
        let popup = NuiApi::get_popup(lua, &popup_id)?;

        popup.unmount(lua)?;

        NeoApi::set_insert_mode(lua, false)
    })?;

    popup.on(lua, &[AutoCmdEvent::BufLeave], close_popup_event)?;

    let close_popup_cb = lua.create_function(move |lua: &Lua, _: ()| {
        NeoApi::notify_dbg(lua, &popup_id)?;
        let popup = NuiApi::get_popup(lua, &popup_id)?;

        popup.unmount(lua)?;

        NeoApi::set_insert_mode(lua, false)
    })?;

    popup.map(lua, Mode::Insert, "<Esc>", close_popup_cb, true)?;

    Ok(())
}
