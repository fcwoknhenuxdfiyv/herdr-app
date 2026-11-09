//! The menu bar as a file: `menus.json` in the app home lists the menus after the
//! app's own (Herdr: settings, hide, quit), each item naming an app or mod command.
//! Shortcuts aren't in the file: a menu item shows whatever keys.json binds to its
//! command. Without the file, `DEFAULT` applies. Only macOS draws a menu bar.
//!
//! ```json
//! [ { "name": "File", "items": [ { "name": "New Tab", "command": "tab.new" }, "-",
//!     { "name": "Go to Tab", "items": [ { "name": "Tab 1", "command": "tab.select:1" } ] } ] } ]
//! ```

use serde::Deserialize;
use std::path::PathBuf;

pub const DEFAULT: &str = include_str!("../defaults/menus.json");

#[cfg(target_os = "macos")]
struct CurrentMenus(Vec<Menu>);
#[cfg(target_os = "macos")]
impl gpui::Global for CurrentMenus {}

#[cfg(target_os = "macos")]
pub fn refresh(cx: &mut gpui::App) {
    if let Some(menus) = cx.try_global::<CurrentMenus>() {
        let menus = menus.0.clone();
        crate::macos::menus(cx, &menus);
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Item {
    Separator,
    Command { name: String, command: String },
    Submenu(Menu),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Menu {
    pub name: String,
    pub items: Vec<Item>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum RawItem {
    Separator(String),
    Entry {
        name: String,
        #[serde(default)]
        command: Option<String>,
        #[serde(default)]
        items: Option<Vec<RawItem>>,
    },
}

#[derive(Deserialize)]
struct RawMenu {
    name: String,
    items: Vec<RawItem>,
}

pub fn path() -> PathBuf {
    crate::mods::app_dir().join("menus.json")
}

/// The person's menus, or the default when there's no file. A broken file is an error
/// (the caller keeps the menus it had).
pub fn load() -> Result<Vec<Menu>, String> {
    match std::fs::read_to_string(path()) {
        Ok(text) => parse(&text),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => parse(DEFAULT),
        Err(e) => Err(format!("cannot read menus.json: {e}")),
    }
}

/// Sets the menu bar from menus.json (or the default). On an error the menus stay as
/// they were; the caller reports it. Only macOS has a menu bar; elsewhere this only checks.
pub fn apply(cx: &mut gpui::App) -> Result<(), String> {
    let menus = load()?;
    #[cfg(target_os = "macos")]
    {
        crate::macos::menus(cx, &menus);
        cx.set_global(CurrentMenus(menus));
    }
    #[cfg(not(target_os = "macos"))]
    let _ = (cx, menus);
    Ok(())
}

pub fn parse(text: &str) -> Result<Vec<Menu>, String> {
    let raw: Vec<RawMenu> = serde_json::from_str(text).map_err(|e| format!("menus.json: {e}"))?;
    let mut problems = Vec::new();
    let menus = raw.into_iter().map(|m| convert_menu(m, &mut problems)).collect();
    if problems.is_empty() { Ok(menus) } else { Err(problems.join("; ")) }
}

fn convert_menu(menu: RawMenu, problems: &mut Vec<String>) -> Menu {
    let items = menu.items.into_iter().filter_map(|item| convert_item(item, &menu.name, problems)).collect();
    Menu { name: menu.name, items }
}

fn convert_item(item: RawItem, menu: &str, problems: &mut Vec<String>) -> Option<Item> {
    match item {
        RawItem::Separator(s) if s.trim_matches('-').is_empty() && !s.is_empty() => Some(Item::Separator),
        RawItem::Separator(s) => {
            problems.push(format!("{menu}: {s:?} is not an item; a separator is \"-\""));
            None
        }
        RawItem::Entry { name, command: Some(command), items: None } => {
            if !valid_command(&command) {
                problems.push(format!("{menu} > {name}: unknown command {command:?}; see `herdr-app commands`, a mod's are <mod>.<command>"));
                return None;
            }
            Some(Item::Command { name, command })
        }
        RawItem::Entry { name, command: None, items: Some(items) } => {
            let items = items.into_iter().filter_map(|i| convert_item(i, &name, problems)).collect();
            Some(Item::Submenu(Menu { name, items }))
        }
        RawItem::Entry { name, .. } => {
            problems.push(format!("{menu} > {name}: an item has a command, or items (a submenu), not both or neither"));
            None
        }
    }
}

/// An app command (with an optional `:value`) or a mod's `<mod>.<command>`.
fn valid_command(command: &str) -> bool {
    let name = command.split_once(':').map_or(command, |(n, _)| n);
    crate::commands::find(name).is_some() || name.split_once('.').is_some_and(|(m, c)| !m.is_empty() && !c.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_menus_parse_and_name_only_real_commands() {
        let menus = parse(DEFAULT).unwrap();
        let names: Vec<_> = menus.iter().map(|m| m.name.as_str()).collect();
        assert_eq!(names, ["Edit", "View", "Window", "Help"]);
        fn commands(items: &[Item], out: &mut Vec<String>) {
            for i in items {
                match i {
                    Item::Command { command, .. } => out.push(command.clone()),
                    Item::Submenu(m) => commands(&m.items, out),
                    Item::Separator => {}
                }
            }
        }
        let mut all = Vec::new();
        for m in &menus {
            commands(&m.items, &mut all);
        }
        assert!(all.iter().all(|c| crate::commands::find(c.split(':').next().unwrap()).is_some()), "{all:?}");
    }

    #[test]
    fn menus_take_commands_separators_and_submenus_and_refuse_the_rest() {
        let text = r#"[{ "name": "Tab", "items": [
            { "name": "Next", "command": "tab.next" }, "-", "---",
            { "name": "Go to", "items": [ { "name": "1", "command": "tab.select:1" }, { "name": "Picker", "command": "mod.open:picker" } ] },
            { "name": "Balance", "command": "tmux.balance" } ] }]"#;
        let menus = parse(text).unwrap();
        assert_eq!(menus.len(), 1);
        assert_eq!(menus[0].items.len(), 5);
        assert!(matches!(&menus[0].items[3], Item::Submenu(m) if m.items.len() == 2));
        assert!(matches!(&menus[0].items[4], Item::Command { command, .. } if command == "tmux.balance"));
        for (bad, expect) in [
            (r#"[{ "name": "X", "items": [ { "name": "Nope", "command": "nope" } ] }]"#, "unknown command"),
            (r#"[{ "name": "X", "items": [ "separator" ] }]"#, "not an item"),
            (r#"[{ "name": "X", "items": [ { "name": "Both", "command": "tab.next", "items": [] } ] }]"#, "not both"),
            (r#"[{ "name": "X", "items": [ { "name": "Neither" } ] }]"#, "not both or neither"),
            (r#"{ "name": "X" }"#, "menus.json"),
        ] {
            let error = parse(bad).unwrap_err();
            assert!(error.contains(expect), "{bad}: {error}");
        }
    }
}
