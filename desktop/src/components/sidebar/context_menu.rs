use std::path::PathBuf;

use dioxus::desktop::tao::window::WindowId;
use dioxus::prelude::*;

use crate::bookmarks::BOOKMARKS;
use crate::components::icon::{Icon, IconName};
use crate::keybindings::{shortcut_hint_for_context_action, KeyContext};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SidebarItemKind {
    File,
    Directory,
}

#[derive(Clone, PartialEq, Debug)]
pub struct SidebarContextMenuData {
    pub position: (i32, i32),
    pub path: PathBuf,
    pub kind: SidebarItemKind,
    pub refresh_counter: Signal<u32>,
}

#[component]
pub fn SidebarContextMenu(
    position: (i32, i32),
    path: PathBuf,
    kind: SidebarItemKind,
    on_close: EventHandler<()>,
    on_open: EventHandler<()>,
    on_open_in_new_window: EventHandler<()>,
    on_move_to_window: EventHandler<WindowId>,
    on_change_root_directory: EventHandler<()>,
    on_toggle_bookmark: EventHandler<()>,
    on_copy_path: EventHandler<()>,
    on_reveal_in_finder: EventHandler<()>,
    on_reload: EventHandler<()>,
    other_windows: Vec<(WindowId, String)>,
) -> Element {
    let mut show_submenu = use_signal(|| false);
    let shortcut = |action| shortcut_hint_for_context_action(KeyContext::Sidebar, action);

    let is_file = kind == SidebarItemKind::File;
    let is_bookmarked = BOOKMARKS.read().contains(&path);

    // Dynamic labels based on item kind
    let open_label = if is_file {
        "Open File"
    } else {
        "Open Directory"
    };
    let copy_path_label = if is_file {
        "Copy File Path"
    } else {
        "Copy Directory Path"
    };

    rsx! {
        // Backdrop to close menu on outside click
        div {
            class: "context-menu-backdrop",
            onclick: move |_| on_close.call(()),
        }

        // Context menu
        div {
            class: "context-menu",
            style: "left: {position.0}px; top: {position.1}px;",
            onclick: move |evt| evt.stop_propagation(),

            // === Section 1: Open operations ===
            ContextMenuItem {
                label: open_label,
                icon: Some(if is_file { IconName::File } else { IconName::FolderOpen }),
                on_click: move |_| on_open.call(()),
            }

            if !is_file {
                ContextMenuItem {
                    label: "Change Root Directory",
                    shortcut: shortcut("cursor.enter"),
                    icon: Some(IconName::FolderOpen),
                    on_click: move |_| on_change_root_directory.call(()),
                }
            }

            ContextMenuItem {
                label: "Open in New Window",
                on_click: move |_| on_open_in_new_window.call(()),
            }

            // Open in Window (with submenu)
            div {
                class: "context-menu-item has-submenu",
                onmouseenter: move |_| show_submenu.set(true),
                onmouseleave: move |_| show_submenu.set(false),

                span { class: "context-menu-label", "Open in Window" }
                span { class: "submenu-arrow", "›" }

                if *show_submenu.read() {
                    div {
                        class: "context-submenu",

                        if other_windows.is_empty() {
                            div {
                                class: "context-menu-item disabled",
                                "No other windows"
                            }
                        } else {
                            for (window_id, title) in other_windows.iter() {
                                {
                                    let window_id = *window_id;
                                    let title = title.clone();
                                    rsx! {
                                        div {
                                            key: "{window_id:?}",
                                            class: "context-menu-item",
                                            onclick: move |_| on_move_to_window.call(window_id),
                                            "{title}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // === Section 2: Quick Access ===
            ContextMenuSeparator {}

            div {
                class: "context-menu-item",
                onclick: move |_| on_toggle_bookmark.call(()),

                Icon {
                    name: if is_bookmarked { IconName::StarFilled } else { IconName::Star },
                    size: 14,
                    class: "context-menu-icon",
                }

                span {
                    class: "context-menu-label",
                    if is_bookmarked { "Remove from Quick Access" } else { "Add to Quick Access" }
                }
            }

            // === Section 3: File operations ===
            ContextMenuSeparator {}

            ContextMenuItem {
                label: copy_path_label,
                shortcut: shortcut("clipboard.copy_file_path"),
                icon: Some(IconName::Copy),
                on_click: move |_| on_copy_path.call(()),
            }

            ContextMenuItem {
                label: "Reveal in Finder",
                shortcut: shortcut("file.reveal_in_finder"),
                icon: Some(IconName::Folder),
                on_click: move |_| on_reveal_in_finder.call(()),
            }

            // === Section 4: Reload ===
            ContextMenuSeparator {}

            ContextMenuItem {
                label: "Reload",
                shortcut: shortcut("window.reload"),
                icon: Some(IconName::Refresh),
                on_click: move |_| on_reload.call(()),
            }
        }
    }
}

// ============================================================================
// Helper Components
// ============================================================================

#[derive(Props, Clone, PartialEq)]
struct ContextMenuItemProps {
    label: &'static str,
    #[props(default)]
    shortcut: Option<String>,
    #[props(default)]
    icon: Option<IconName>,
    #[props(default = false)]
    disabled: bool,
    on_click: EventHandler<()>,
}

#[component]
fn ContextMenuItem(props: ContextMenuItemProps) -> Element {
    rsx! {
        div {
            class: if props.disabled { "context-menu-item disabled" } else { "context-menu-item" },
            onclick: move |_| {
                if !props.disabled {
                    props.on_click.call(());
                }
            },

            if let Some(icon) = props.icon {
                Icon {
                    name: icon,
                    size: 14,
                    class: "context-menu-icon",
                }
            }

            span { class: "context-menu-label", "{props.label}" }

            if let Some(shortcut) = props.shortcut {
                span { class: "context-menu-shortcut", "{shortcut}" }
            }
        }
    }
}

#[component]
fn ContextMenuSeparator() -> Element {
    rsx! {
        div { class: "context-menu-separator" }
    }
}

#[component]
pub fn SidebarContextMenuRoot() -> Element {
    let mut state = use_context::<crate::state::AppState>();
    
    // Evaluate other_windows on mount of the context menu.
    // When context_menu_data becomes Some, this component will mount and evaluate this.
    let other_windows = {
        let windows = crate::window::main::list_visible_main_windows();
        let current_id = dioxus::desktop::window().id();
        windows
            .iter()
            .filter(|w| w.window.id() != current_id)
            .map(|w| (w.window.id(), w.window.title()))
            .collect::<Vec<_>>()
    };

    let menu_data = state.sidebar.read().context_menu_data.clone();

    if let Some(data) = menu_data {
        let is_dir = data.kind == SidebarItemKind::Directory;
        let path = data.path.clone();
        
        let on_close = move |_| {
            state.sidebar.write().context_menu_data = None;
        };

        let on_open = {
            let path = path.clone();
            let mut state = state;
            move |_| {
                if is_dir {
                    state.set_root_directory(&path);
                } else {
                    state.open_file(&path);
                }
                state.sidebar.write().context_menu_data = None;
            }
        };

        let on_change_root_directory = {
            let path = path.clone();
            let mut state = state;
            move |_| {
                state.set_root_directory(&path);
                state.sidebar.write().context_menu_data = None;
            }
        };

        let on_open_in_new_window = {
            let path = path.clone();
            move |_| {
                let path = path.clone();
                spawn(async move {
                    let (tab, directory) = if is_dir {
                        (crate::state::Tab::default(), Some(path))
                    } else {
                        (
                            crate::state::Tab::new(&path),
                            path.parent().map(|p| p.to_path_buf()),
                        )
                    };
                    let params = crate::window::main::CreateMainWindowConfigParams {
                        directory,
                        ..Default::default()
                    };
                    crate::window::main::create_main_window(tab, params).await;
                });
                state.sidebar.write().context_menu_data = None;
            }
        };

        let on_move_to_window = {
            let path = path.clone();
            move |target_id: WindowId| {
                let path = path.clone();
                let result = if is_dir {
                    crate::events::OPEN_DIRECTORY_IN_WINDOW.send((target_id, path))
                } else {
                    crate::events::OPEN_FILE_IN_WINDOW.send((target_id, path))
                };
                if result.is_err() {
                    tracing::warn!(
                        ?target_id,
                        "Failed to open in window: target window may be closed"
                    );
                    state.sidebar.write().context_menu_data = None;
                    return;
                }
                crate::window::main::focus_window(target_id);
                state.sidebar.write().context_menu_data = None;
            }
        };

        let on_copy_path = {
            let path = path.clone();
            move |_| {
                crate::utils::clipboard::copy_text(path.to_string_lossy());
                state.sidebar.write().context_menu_data = None;
            }
        };

        let on_reveal_in_finder = {
            let path = path.clone();
            move |_| {
                crate::utils::file_operations::reveal_in_finder(&path);
                state.sidebar.write().context_menu_data = None;
            }
        };

        let on_reload = {
            let mut refresh_counter = data.refresh_counter;
            move |_| {
                refresh_counter.set(refresh_counter() + 1);
                state.sidebar.write().context_menu_data = None;
            }
        };

        let on_toggle_bookmark = {
            let path = path.clone();
            move |_| {
                crate::bookmarks::toggle_bookmark(&path);
                state.sidebar.write().context_menu_data = None;
            }
        };

        rsx! {
            SidebarContextMenu {
                position: data.position,
                path: data.path,
                kind: data.kind,
                on_close,
                on_open,
                on_open_in_new_window,
                on_move_to_window,
                on_change_root_directory,
                on_toggle_bookmark,
                on_copy_path,
                on_reveal_in_finder,
                on_reload,
                other_windows,
            }
        }
    } else {
        rsx! { "" }
    }
}
