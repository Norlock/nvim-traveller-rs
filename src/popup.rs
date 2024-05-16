use crate::{
    state::{AppInstance, AppState},
    CONTAINER,
};
use neo_api_rs::{
    mlua::prelude::{Lua, LuaResult},
    *,
};
use std::{fs, io, path::PathBuf, time::Duration};

pub async fn delete_items_popup(lua: &Lua, _: ()) -> LuaResult<()> {
    let popup_buf = NeoBuffer::create(lua, false, true)?;

    let instances = CONTAINER.instances.read().await;
    let instance = instances.get(&AppState::active_buf()).unwrap();

    let filename = instance.get_item(lua)?;
    let delete_info = format!("Delete: {filename}");
    let file_path = instance.cwd.join(filename);

    let popup_win = NeoPopup::open_win(
        lua,
        &popup_buf,
        true,
        WinOptions {
            relative: PopupRelative::Editor,
            width: Some(PopupSize::Percentage(1.)),
            height: Some(PopupSize::Fixed(1)),
            col: Some(PopupSize::Fixed(0)),
            row: Some(PopupSize::Percentage(1.)),
            style: Some(PopupStyle::Minimal),
            border: PopupBorder::Rounded,
            anchor: Anchor::NorthWest,
            title: Some(TextType::Tuples(vec![HLText::new(
                " Confirm: (enter), cancel: (q) ",
                "Question",
            )])),
            title_pos: PopupAlign::Right,
            noautocmd: true,
            ..Default::default()
        },
    )?;

    popup_buf.set_lines(lua, 0, -1, false, &[delete_info])?;

    let close_popup = lua.create_function(move |lua: &Lua, _: ()| popup_win.close(lua, true))?;

    lua.set_named_registry_value("del_popup_fp", file_path.to_string_lossy())?;
    lua.set_named_registry_value("del_popup_nw", popup_win.id())?;

    let delete_item = lua.create_async_function(|lua: &Lua, _: ()| async move {
        let file_path: String = lua.named_registry_value("del_popup_fp")?;
        let file_path: PathBuf = file_path.into();

        if file_path.is_file() {
            let _ = fs::remove_file(&*file_path);
        } else if file_path.is_dir() {
            let _ = fs::remove_dir_all(&*file_path);
        }

        let mut instances = CONTAINER.instances.write().await;
        let instance = instances.get_mut(&AppState::active_buf()).unwrap();
        instance.set_buffer_content(lua).await?;

        let id: u32 = lua.named_registry_value("del_popup_nw")?;
        NeoWindow::new(id).close(lua, false)
    })?;

    popup_buf.set_keymap(lua, Mode::Normal, "q", close_popup)?;
    popup_buf.set_keymap(lua, Mode::Normal, "<Cr>", delete_item)?;

    Ok(())
}

pub async fn rename_item_popup(lua: &Lua, _: ()) -> LuaResult<()> {
    let popup_buf = NeoBuffer::create(lua, false, true)?;

    let instances = CONTAINER.instances.read().await;
    let instance = instances.get(&AppState::active_buf()).unwrap();

    let filename = instance.get_item(lua)?;
    let filename_len = filename.len();
    let source_path = instance.cwd.join(filename);
    let file_path = source_path.to_string_lossy().to_string();
    let file_path_len = file_path.len();

    popup_buf.set_lines(lua, 0, -1, false, &[file_path])?;

    let popup_win = NeoPopup::open_win(
        lua,
        &popup_buf,
        true,
        WinOptions {
            relative: PopupRelative::Editor,
            width: Some(PopupSize::Percentage(0.6)),
            height: Some(PopupSize::Fixed(1)),
            row: Some(PopupSize::Percentage(0.1)),
            col: Some(PopupSize::Percentage(0.2)),
            style: Some(PopupStyle::Minimal),
            border: PopupBorder::Rounded,
            title: Some(TextType::Tuples(vec![HLText::new(
                " Confirm: (enter), cancel: (escape) ",
                "Question",
            )])),
            title_pos: PopupAlign::Right,
            ..Default::default()
        },
    )?;

    let cursor_col = file_path_len - filename_len;

    popup_win.set_cursor(lua, WinCursor::from_zero_indexed(0, cursor_col as u32))?;

    let rename_item = lua.create_async_function(|lua, ()| async move {
        let mut instances = CONTAINER.instances.write().await;
        let instance = instances.get_mut(&AppState::active_buf()).unwrap();
        let source = instance.cwd.join(instance.get_item(lua)?);

        let line = NeoApi::get_current_line(lua)?;
        let target = instance.cwd.join(line);

        // Disallow rename existing files
        if source.is_file() && !target.is_file() || source.is_dir() && !target.is_dir() {
            fs::rename(source, target)?;
            instance.set_buffer_content(lua).await?;
            instance.buf.set_current(lua)
        } else {
            NeoPopup::notify(
                lua,
                PopupNotify {
                    level: PopupLevel::Error,
                    title: "File or directory already exists".to_string(),
                    messages: vec!["If you want overwrite the file do it manually".to_string()],
                    duration: Duration::from_secs(5),
                },
            )
        }
    })?;

    let close_popup = lua.create_function(move |lua: &Lua, _: ()| {
        popup_win.close(lua, true)?;
        NeoApi::set_insert_mode(lua, false)
    })?;

    NeoApi::create_autocmd(
        lua,
        &[AutoCmdEvent::BufLeave],
        AutoCmdOpts {
            buffer: Some(popup_buf.id()),
            callback: close_popup.clone(),
            desc: None,
            group: None,
            pattern: vec![],
            once: false,
        },
    )?;

    popup_buf.set_keymap(lua, Mode::Normal, "<Esc>", close_popup)?;
    popup_buf.set_keymap(lua, Mode::Normal, "<Cr>", rename_item.clone())?;
    popup_buf.set_keymap(lua, Mode::Insert, "<Cr>", rename_item)?;

    Ok(())
}

pub async fn select_items_popup(lua: &Lua, _: ()) -> LuaResult<()> {
    let mut instances = CONTAINER.instances.write().await;
    let instance = instances.get_mut(&AppState::active_buf()).unwrap();

    let item = instance.get_item(lua)?;

    let path_items = instance.selection.get_mut(&instance.cwd);

    if let Some(path_items) = path_items {
        if path_items.contains(&item) {
            path_items.remove(&item);

            if path_items.is_empty() {
                instance.selection.remove(&instance.cwd);
            }
        } else {
            path_items.insert(item);
        }
    } else {
        instance
            .selection
            .insert(instance.cwd.clone(), [item].into());
    }

    let count: usize = instance.selection.iter().map(|sel| sel.1.len()).sum();

    let lines = [
        format!("Selected: ({})", count),
        "[u]  undo".to_string(),
        "[pm] paste as move".to_string(),
        "[pc] paste as copy".to_string(),
        "[ds] delete".to_string(),
    ];

    if count == 0 {
        instance.close_selection_popup(lua).await?;
    } else if let Some(popup) = &instance.selection_popup {
        popup.buf.set_lines(lua, 0, -1, false, &lines)?;
        instance.theme_nav_buffer(lua).await?;
    } else {
        let popup_buf = NeoBuffer::create(lua, false, true)?;
        instance.theme_nav_buffer(lua).await?;

        popup_buf.set_lines(lua, 0, -1, false, &lines)?;

        let popup = NeoPopup::open(
            lua,
            popup_buf,
            false,
            WinOptions {
                relative: PopupRelative::Win,
                win: Some(instance.win.id()),
                width: Some(PopupSize::Fixed(20)),
                height: Some(PopupSize::Fixed(5)),
                col: Some(PopupSize::Fixed(1000)),
                row: Some(PopupSize::Fixed(0)),
                style: Some(PopupStyle::Minimal),
                border: PopupBorder::Rounded,
                anchor: Anchor::NorthWest,
                focusable: Some(false),
                title: None,
                noautocmd: true,
                ..Default::default()
            },
        )?;

        instance.selection_popup = Some(popup);
    }

    NeoApi::notify_dbg(lua, &instance.selection)?;

    Ok(())
}

pub async fn create_items_popup(lua: &Lua, _: ()) -> LuaResult<()> {
    let popup_buf = NeoBuffer::create(lua, false, true)?;

    let popup_win = NeoPopup::open_win(
        lua,
        &popup_buf,
        true,
        WinOptions {
            relative: PopupRelative::Editor,
            width: Some(PopupSize::Percentage(0.6)),
            height: Some(PopupSize::Fixed(1)),
            col: Some(PopupSize::Percentage(0.2)),
            row: Some(PopupSize::Percentage(0.1)),
            style: Some(PopupStyle::Minimal),
            border: PopupBorder::Rounded,
            title: Some(TextType::Tuples(vec![HLText::new(
                " Create items (split by space) ",
                "Question",
            )])),
            noautocmd: true,
            ..Default::default()
        },
    )?;

    NeoApi::set_insert_mode(lua, true)?;

    let close_popup_event = lua.create_function(move |lua: &Lua, _: ()| {
        popup_win.close(lua, true)?;
        NeoApi::set_insert_mode(lua, false)
    })?;

    NeoApi::create_autocmd(
        lua,
        &[AutoCmdEvent::BufLeave],
        AutoCmdOpts {
            buffer: Some(popup_buf.id()),
            callback: close_popup_event.clone(),
            desc: None,
            group: None,
            pattern: vec![],
            once: true,
        },
    )?;

    popup_buf.set_keymap(lua, Mode::Insert, "<Esc>", close_popup_event)?;

    let confirm_selection = lua.create_async_function(move |lua: &Lua, _: ()| async move {
        let lines = popup_buf.get_lines(lua, 0, 1, false)?;

        let items_cmd = lines[0].to_string();

        let quote_count = items_cmd.chars().filter(|c| *c == '"').count();

        if quote_count % 2 == 0 {
            let mut instances = CONTAINER.instances.write().await;
            let instance = instances.get_mut(&AppState::active_buf()).unwrap();

            create_items(instance, items_cmd)?;
            instance.set_buffer_content(lua).await?;

            // TODO feedback
            popup_win.close(lua, true)?;
        }

        Ok(())
    })?;

    popup_buf.set_keymap(lua, Mode::Insert, "<Cr>", confirm_selection)
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

fn create_items(instance: &AppInstance, items_cmd: String) -> io::Result<()> {
    let items = split_items(items_cmd);

    for item in items.iter() {
        let path = instance.cwd.join(item);

        if item.ends_with("/") {
            if path.is_dir() {
                continue;
            }
            fs::create_dir_all(path)?;
        } else if path.is_file() || path.is_symlink() {
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
