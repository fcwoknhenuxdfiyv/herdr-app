use gpui::*;
use gpui_shell::action::ShellAction;

actions!(herdr, [Quit, Hide, HideOthers, ShowAll, Settings]);

pub fn init(cx: &mut App) {
    cx.on_action(|_: &Quit, cx| cx.quit());
    cx.on_action(|_: &Hide, cx| cx.hide());
    cx.on_action(|_: &HideOthers, cx| cx.hide_other_apps());
    cx.on_action(|_: &ShowAll, cx| cx.unhide_other_apps());
    cx.on_action(|_: &Settings, _| crate::mods::request(crate::mods::Request::Run { name: "app.settings".into(), args: serde_json::Value::Null }));
    // A broken menus.json still gets the person the default menus; the window reports it.
    if crate::menus::apply(cx).is_err() {
        menus(cx, &crate::menus::parse(crate::menus::DEFAULT).unwrap_or_default());
    }
}

/// The menu bar: the app's own menu, then `menus` (menus.json). Items run app and mod
/// commands as a key would (the window's `ShellAction` handler); the shortcut shown
/// beside each is whatever keys.json binds to its command, read from the keymap when
/// the menus are set, so this runs again after every keys.json or menus.json reload.
pub fn menus(cx: &mut App, menus: &[crate::menus::Menu]) {
    let mut all = vec![Menu::new("Herdr").items([
        MenuItem::action("Settings…", Settings),
        MenuItem::separator(),
        MenuItem::os_submenu("Services", SystemMenuType::Services),
        MenuItem::separator(),
        MenuItem::action("Hide Herdr", Hide),
        MenuItem::action("Hide Others", HideOthers),
        MenuItem::action("Show All", ShowAll),
        MenuItem::separator(),
        MenuItem::action("Quit Herdr", Quit),
    ])];
    all.extend(menus.iter().map(menu));
    cx.set_menus(all);
}

fn menu(m: &crate::menus::Menu) -> Menu {
    Menu::new(m.name.clone()).items(m.items.iter().map(item))
}

fn item(i: &crate::menus::Item) -> MenuItem {
    use crate::menus::Item;
    match i {
        Item::Separator => MenuItem::separator(),
        Item::Submenu(m) => MenuItem::submenu(menu(m)),
        Item::Command { name, command } => MenuItem::Action {
            name: name.clone().into(),
            // Native where the key bindings use a native action too, so the shortcut matches.
            action: action(command).unwrap_or_else(|| Box::new(ShellAction::new(command.clone()))),
            os_action: None,
            checked: false,
            disabled: false,
        },
    }
}

pub fn action(name: &str) -> Option<Box<dyn Action>> {
    match name {
        "app.quit" => Some(Box::new(Quit)),
        "app.hide" => Some(Box::new(Hide)),
        "app.hide_others" => Some(Box::new(HideOthers)),
        "app.show_all" => Some(Box::new(ShowAll)),
        "app.settings" => Some(Box::new(Settings)),
        _ => None,
    }
}
