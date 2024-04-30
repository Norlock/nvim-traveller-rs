use crate::{state::AppInstance, CONTAINER, RUNTIME};
use futures::Future;
use neo_api_rs::{
    mlua::{prelude::LuaResult, Lua},
    prelude::{
        AutoCmdCbEvent, AutoCmdEvent, Mode, NeoApi, NeoBuffer, NeoPopup, NuiAlign, NuiApi,
        NuiBorder, NuiBorderPadding, NuiBorderStyle, NuiBorderText, NuiDimension, NuiPopupOpts,
        NuiRelative, NuiSize, PopupBorder, PopupRelative, PopupSize, PopupStyle, TextType,
        WinOptions,
    },
};
use std::{fs, io};

pub async fn delete_items_popup(lua: &Lua, _: ()) -> LuaResult<()> {
    //

    Ok(())
}

pub async fn create_items_popup(lua: &Lua, _: ()) -> LuaResult<()> {
    //let popup_id = "create_items";

    let buf = NeoApi::create_buf(lua, false, true)?;

    let popup = NeoPopup::open_win(
        lua,
        buf.id(),
        true,
        WinOptions {
            relative: PopupRelative::Editor,
            width: Some(PopupSize::Percentage(0.6)),
            height: Some(PopupSize::Fixed(1)),
            col: Some(PopupSize::Percentage(0.5)),
            row: Some(PopupSize::Percentage(0.2)),
            style: Some(PopupStyle::Minimal),
            border: PopupBorder::Rounded,
            title: Some(TextType::String(" Create items (split by space) ".to_string())),
            noautocmd: true,
            ..Default::default()
        },
    )?;

    // TODO add to state

    NeoApi::set_insert_mode(lua, true)?;

    //let close_popup_event = lua.create_function(move |lua: &Lua, _: AutoCmdCbEvent| {
    //let popup = NuiApi::get_popup(lua, &popup_id)?;

    //popup.unmount(lua)?;

    //NeoApi::set_insert_mode(lua, false)
    //})?;

    //popup.on(lua, &[AutoCmdEvent::BufLeave], close_popup_event)?;

    //let close_popup_cb = lua.create_function(move |lua: &Lua, _: ()| {
    //let popup = NuiApi::get_popup(lua, &popup_id)?;

    //popup.unmount(lua)?;

    //NeoApi::set_insert_mode(lua, false)
    //})?;

    //popup.map(lua, Mode::Insert, "<Esc>", close_popup_cb, true)?;

    //let confirm_selection = lua.create_function(move |lua: &Lua, _: ()| {
    //let popup = NuiApi::get_popup(lua, &popup_id)?;

    //if let Some(buf_id) = popup.bufnr(lua)? {
    //let lines = NeoBuffer::new(buf_id).get_lines(lua, 0, 1, false)?;

    //let items_cmd = lines[0].to_string();

    //let quote_count = items_cmd.chars().filter(|c| *c == '"').count();

    //if quote_count % 2 == 1 {
    //// TODO feedback
    //return Ok(());
    //}

    //RUNTIME.block_on(async move {
    //let mut app = CONTAINER.lock().await;
    //let theme = app.theme.clone();
    //let instance = app.active_instance();

    //let _ = create_items(instance, items_cmd).await;
    //let _ = instance.set_buffer_content(&theme, lua);
    //});

    //// TODO feedback
    //popup.unmount(lua)?;
    //}

    //Ok(())
    //})?;

    //popup.map(lua, Mode::Insert, "<Cr>", confirm_selection, true)?;

    Ok(())
}

fn split_items(mut items_cmd: String) -> Vec<String> {
    let mut items = vec![];

    const SKIP_OFFSET: usize = 1;

    loop {
        let start_quote = items_cmd.chars().position(|c| c == '"');

        if let Some(start_quote) = start_quote {
            let end_quote = items_cmd
                .chars()
                .skip(start_quote + SKIP_OFFSET)
                .position(|c| c == '"');

            if let Some(end_quote) = end_quote {
                let end_quote = start_quote + SKIP_OFFSET + end_quote;
                items.push(items_cmd[start_quote + SKIP_OFFSET..end_quote].to_string());
                items_cmd.replace_range(start_quote..=end_quote, "");

                continue;
            }
        }

        break;
    }

    for item in items_cmd.split(" ") {
        if !item.is_empty() {
            items.push(item.to_string());
        }
    }

    items
}

async fn create_items(instance: &AppInstance, items_cmd: String) -> io::Result<()> {
    let items = split_items(items_cmd);

    for item in items.iter() {
        let path = instance.cwd.join(item);

        if item.ends_with("/") {
            if path.is_dir() {
                continue;
            }
            fs::create_dir_all(path)?;
        } else if path.is_file() || path.is_symlink() {
            // TODO maybe overwrite?
            continue;
        } else {
            if let Some(parent) = path.parent() {
                if !parent.is_dir() {
                    fs::create_dir_all(parent)?;
                }
            }

            fs::File::create(path)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::popup::split_items;

    #[test]
    pub fn test() {
        let items_cmd = "\"this is.txt\" \"another one.txt\" css/".to_string();
        let items = split_items(items_cmd);

        assert_eq!("this is.txt", items[0].as_str());
        assert_eq!("another one.txt", items[1].as_str());
        assert_eq!("css/", items[2].as_str());
        assert_eq!(items.len(), 3);
    }
}
