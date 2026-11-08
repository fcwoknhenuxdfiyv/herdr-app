use gpui::*;
use gpui_shell::action::ShellAction;

actions!(herdr, [Quit, Hide, HideOthers, ShowAll, Settings]);

pub fn init(cx: &mut App) {
    cx.on_action(|_: &Quit, cx| cx.quit());
    cx.on_action(|_: &Hide, cx| cx.hide());
    cx.on_action(|_: &HideOthers, cx| cx.hide_other_apps());
    cx.on_action(|_: &ShowAll, cx| cx.unhide_other_apps());
    cx.on_action(|_: &Settings, _| crate::mods::request(crate::mods::Request::Run { name: "app.settings".into(), args: serde_json::Value::Null }));
    menus(cx);
}

/// The menu bar. Items run app commands (the window's `ShellAction` handler, as a key
/// would); the shortcut shown beside each is whatever keys.json binds to it, read from
/// the keymap when the menus are set, so `keys::install` sets them again after every reload.
pub fn menus(cx: &mut App) {
    let run = |name: &'static str, command: &'static str| MenuItem::action(name, ShellAction::new(command));
    cx.set_menus([
        Menu::new("Herdr").items([
            MenuItem::action("Settings…", Settings),
            MenuItem::separator(),
            MenuItem::os_submenu("Services", SystemMenuType::Services),
            MenuItem::separator(),
            MenuItem::action("Hide Herdr", Hide),
            MenuItem::action("Hide Others", HideOthers),
            MenuItem::action("Show All", ShowAll),
            MenuItem::separator(),
            MenuItem::action("Quit Herdr", Quit),
        ]),
        Menu::new("Edit").items([
            run("Copy", "terminal.copy"),
            run("Paste", "terminal.paste"),
            MenuItem::separator(),
            run("Find…", "terminal.search"),
        ]),
        Menu::new("View").items([
            run("Command Palette…", "app.palette"),
            run("Toggle Sidebar", "layout.toggle:sidebar"),
            run("Next Layout", "layout.switch"),
            MenuItem::separator(),
            run("Zoom In", "app.zoom_in"),
            run("Zoom Out", "app.zoom_out"),
            run("Actual Size", "app.zoom_reset"),
        ]),
        Menu::new("Window").items([
            run("Minimize", "window.minimize"),
            run("Zoom", "window.maximize"),
        ]),
        Menu::new("Help").items([
            run("Keyboard Shortcuts", "app.help"),
            run("What's New", "app.whats_new"),
            run("Send Feedback…", "app.feedback"),
        ]),
    ]);
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
