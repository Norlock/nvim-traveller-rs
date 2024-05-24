use crate::{
    state::{AppInstance, AppState, SelectionData},
    CONTAINER,
};
use neo_api_rs::{
    mlua::prelude::{Lua, LuaResult},
    *,
};
use std::{fs, io, path::PathBuf, time::Duration};

#[derive(Clone)]
struct DeleteItemsCb {
    file_path: PathBuf,
    popup_win: NeoWindow,
}

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

    NeoBridge::insert(
        "del_popup",
        Box::new(DeleteItemsCb {
            file_path,
            popup_win,
        }),
    )
    .await;

    let delete_item = lua.create_async_function(|lua: &Lua, ()| async move {
        let DeleteItemsCb {
            popup_win,
            file_path,
        } = NeoBridge::consume("del_popup").await?;

        if file_path.is_file() {
            let _ = fs::remove_file(&*file_path);
        } else if file_path.is_dir() {
            let _ = fs::remove_dir_all(&*file_path);
        }

        let mut instances = CONTAINER.instances.write().await;
        let instance = instances.get_mut(&AppState::active_buf()).unwrap();

        let selection = CONTAINER.selection.read().await;
        instance.set_buffer_content(lua, &selection).await?;

        popup_win.close(lua, false)
    })?;

    popup_buf.set_keymap(lua, Mode::Normal, "q", close_popup)?;
    popup_buf.set_keymap(lua, Mode::Normal, "<Cr>", delete_item)?;

    Ok(())
}

pub async fn rename_items_popup(lua: &Lua, _: ()) -> LuaResult<()> {
    let popup_buf = NeoBuffer::create(lua, false, true)?;

    let instances = CONTAINER.instances.read().await;
    let instance = instances.get(&AppState::active_buf()).unwrap();

    //if instance.selection.is_empty() {

    //}

    let filename = instance.get_item(lua)?;
    let filename_len = filename.len();
    let source_path = instance.cwd.join(filename);

    NeoBridge::insert("rename_file_path", Box::new(source_path.clone())).await;

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

        let source: PathBuf = NeoBridge::consume("rename_file_path").await?;
        let line = NeoApi::get_current_line(lua)?;
        let target = instance.cwd.join(line);

        // Disallow rename existing files
        if source.is_file() && !target.is_file() || source.is_dir() && !target.is_dir() {
            fs::rename(source, target)?;

            let selection = CONTAINER.selection.read().await;

            instance.set_buffer_content(lua, &selection).await?;
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

pub async fn show_selection_popup(
    lua: &Lua,
    selection: &SelectionData,
    instance: &mut AppInstance,
) -> LuaResult<()> {
    let count: usize = selection.iter().map(|sel| sel.1.len()).sum();

    let lines = [
        &format!("Selected: ({})", count),
        "[u]  undo",
        "[pm] paste as move",
        "[pc] paste as copy",
        "[Rs] rename",
        "[ds] delete",
    ];

    if count == 0 {
        instance.close_selection_popup(lua, selection).await?;
    } else if let Some(popup) = &instance.selection_popup {
        popup.buf.set_lines(lua, 0, -1, false, &lines)?;
        instance.theme_nav_buffer(lua, selection).await?;
    } else {
        let popup_buf = NeoBuffer::create(lua, false, true)?;
        instance.theme_nav_buffer(lua, selection).await?;

        popup_buf.set_lines(lua, 0, -1, false, &lines)?;

        let popup = NeoPopup::open(
            lua,
            popup_buf,
            false,
            WinOptions {
                relative: PopupRelative::Win,
                win: Some(instance.win.id()),
                width: Some(PopupSize::Fixed(20)),
                height: Some(PopupSize::Fixed(lines.len() as u32)),
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

    Ok(())
}

pub async fn update_selection_popup(lua: &Lua, _: ()) -> LuaResult<()> {
    let mut instances = CONTAINER.instances.write().await;
    let instance = instances.get_mut(&AppState::active_buf()).unwrap();

    let item = instance.get_item(lua)?;

    let mut selection = CONTAINER.selection.write().await;
    let path_items = selection.get_mut(&instance.cwd);

    if let Some(path_items) = path_items {
        if path_items.contains(&item) {
            path_items.remove(&item);

            if path_items.is_empty() {
                selection.remove(&instance.cwd);
            }
        } else {
            path_items.insert(item);
        }
    } else {
        selection.insert(instance.cwd.clone(), [item].into());
    }

    show_selection_popup(lua, &selection, instance).await
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

    let popup_leave_event = lua.create_function(move |lua: &Lua, _: ()| {
        let cb = lua.create_function(move |lua: &Lua, ()| {
            if NeoWindow::get_current_win(lua)? == popup_win {
                popup_win.close(lua, false)?;
            }

            NeoApi::set_insert_mode(lua, false)
        })?;

        NeoApi::delay(lua, 64, cb)
    })?;

    NeoApi::create_autocmd(
        lua,
        &[AutoCmdEvent::BufLeave],
        AutoCmdOpts {
            buffer: Some(popup_buf.id()),
            callback: popup_leave_event.clone(),
            desc: None,
            group: None,
            pattern: vec![],
            once: true,
        },
    )?;

    popup_buf.set_keymap(lua, Mode::Insert, "<Esc>", popup_leave_event)?;

    let confirm_selection = lua.create_async_function(move |lua: &Lua, _: ()| async move {
        let lines = popup_buf.get_lines(lua, 0, 1, false)?;

        let items_cmd = lines[0].to_string();

        let quote_count = items_cmd.chars().filter(|c| *c == '"').count();

        if quote_count % 2 == 0 {
            let mut instances = CONTAINER.instances.write().await;
            let instance = instances.get_mut(&AppState::active_buf()).unwrap();

            create_items(instance, items_cmd)?;

            let selection = CONTAINER.selection.read().await;
            instance.set_buffer_content(lua, &selection).await?;

            // TODO feedback
            popup_win.close(lua, false)?;
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
