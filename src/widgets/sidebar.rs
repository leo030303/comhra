use adw::prelude::*;
use std::fs::{self};

use std::path::PathBuf;

use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use crate::models::SavedConversation;
use crate::utils;
/*
- Button to filter list to show/hide archived
- Button to filter list to show only starred
- List of sidebar list item widgets
*/

#[derive(Clone, Debug)]
enum SideBarFilterState {
    NoFilter,
    ShowArchive,
    JustFavourite,
}

pub fn create_sidebar(conversation_file_option_sender: Sender<Option<PathBuf>>) -> gtk::Box {
    let filter_state = Arc::new(Mutex::new(SideBarFilterState::NoFilter));
    let main_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(10)
        .visible(false)
        .build();
    let filter_box = gtk::Box::builder()
        .homogeneous(true)
        .orientation(gtk::Orientation::Horizontal)
        .spacing(0)
        .build();
    let archive_filter_button = gtk::Button::builder()
        .label("Show archived")
        .has_tooltip(true)
        .tooltip_text("Show archived")
        .build();
    let favourite_filter_button = gtk::Button::builder()
        .label("Show only favourites")
        .has_tooltip(true)
        .tooltip_text("Show only favourites")
        .build();
    filter_box.append(&archive_filter_button);
    filter_box.append(&favourite_filter_button);

    let sidebar_scroll_window = gtk::ScrolledWindow::new();
    let conversation_list_box = gtk::ListBox::builder().vexpand(true).build();
    sidebar_scroll_window.set_child(Some(&conversation_list_box));
    let saved_conversations = utils::get_filenames_from_folder(PathBuf::from("./conversations"));
    let mut list_items: Vec<SideBarListItem> = Vec::new();

    saved_conversations.iter().for_each(|file_path| {
        let conversation_file_option_sender_for_button = conversation_file_option_sender.clone();
        let file_path_for_button = PathBuf::from(file_path.file_name().unwrap());
        let sidebar_list_item = SideBarListItem::new(
            file_path_for_button,
            conversation_file_option_sender_for_button,
        );
        conversation_list_box.append(&sidebar_list_item.main_box);
        list_items.push(sidebar_list_item);
    });
    main_box.append(&filter_box);
    main_box.append(&sidebar_scroll_window);
    {
        let list_items = list_items.clone();
        let favourite_filter_button = favourite_filter_button.clone();
        let filter_state = Arc::clone(&filter_state);
        archive_filter_button.connect_clicked(move |archive_filter_button| {
            toggle_archive_filter(
                &favourite_filter_button,
                archive_filter_button,
                Arc::clone(&filter_state),
                &list_items,
            )
        });
    }
    {
        let list_items = list_items.clone();
        let archive_filter_button = archive_filter_button.clone();
        let filter_state = Arc::clone(&filter_state);
        favourite_filter_button.connect_clicked(move |favourite_filter_button| {
            toggle_favourite_filter(
                favourite_filter_button,
                &archive_filter_button,
                Arc::clone(&filter_state),
                &list_items,
            )
        });
    }

    main_box
}

fn toggle_favourite_filter(
    favourite_filter_button: &gtk::Button,
    archive_filter_button: &gtk::Button,
    filter_state: Arc<Mutex<SideBarFilterState>>,
    list_items: &[SideBarListItem],
) {
    let current_state = (*filter_state.lock().unwrap()).clone();
    if let SideBarFilterState::JustFavourite = current_state {
        *filter_state.lock().unwrap() = SideBarFilterState::NoFilter;
        favourite_filter_button.set_label("Show only favourites");
        favourite_filter_button.set_tooltip_text(Some("Show only favourites"));
        list_items.iter().for_each(|sidebar_list_item| {
            match *sidebar_list_item.permanent_state.lock().unwrap() {
                SideBarListItemPermanentState::Normal => sidebar_list_item.main_box.show(),
                SideBarListItemPermanentState::Archived => {}
                SideBarListItemPermanentState::Starred => sidebar_list_item.main_box.show(),
                SideBarListItemPermanentState::Deleted => {}
            }
        });
    } else {
        *filter_state.lock().unwrap() = SideBarFilterState::JustFavourite;
        favourite_filter_button.set_label("Show all");
        favourite_filter_button.set_tooltip_text(Some("Show all"));
        archive_filter_button.set_label("Show archived");
        archive_filter_button.set_tooltip_text(Some("Show archived"));
        list_items.iter().for_each(|sidebar_list_item| {
            match *sidebar_list_item.permanent_state.lock().unwrap() {
                SideBarListItemPermanentState::Normal => sidebar_list_item.main_box.hide(),
                SideBarListItemPermanentState::Archived => sidebar_list_item.main_box.hide(),
                SideBarListItemPermanentState::Starred => sidebar_list_item.main_box.show(),
                SideBarListItemPermanentState::Deleted => {}
            }
        });
    }
}

fn toggle_archive_filter(
    favourite_filter_button: &gtk::Button,
    archive_filter_button: &gtk::Button,
    filter_state: Arc<Mutex<SideBarFilterState>>,
    list_items: &[SideBarListItem],
) {
    let current_state = (*filter_state.lock().unwrap()).clone();
    if let SideBarFilterState::ShowArchive = current_state {
        *filter_state.lock().unwrap() = SideBarFilterState::NoFilter;
        archive_filter_button.set_label("Show archived");
        archive_filter_button.set_tooltip_text(Some("Show archived"));
        list_items.iter().for_each(|sidebar_list_item| {
            match *sidebar_list_item.permanent_state.lock().unwrap() {
                SideBarListItemPermanentState::Normal => sidebar_list_item.main_box.show(),
                SideBarListItemPermanentState::Archived => sidebar_list_item.main_box.hide(),
                SideBarListItemPermanentState::Starred => sidebar_list_item.main_box.show(),
                SideBarListItemPermanentState::Deleted => {}
            }
        });
    } else {
        *filter_state.lock().unwrap() = SideBarFilterState::ShowArchive;
        archive_filter_button.set_label("Hide archived");
        archive_filter_button.set_tooltip_text(Some("Hide archived"));
        favourite_filter_button.set_label("Show only favourites");
        favourite_filter_button.set_tooltip_text(Some("Show only favourites"));
        list_items.iter().for_each(|sidebar_list_item| {
            match *sidebar_list_item.permanent_state.lock().unwrap() {
                SideBarListItemPermanentState::Normal => sidebar_list_item.main_box.show(),
                SideBarListItemPermanentState::Archived => sidebar_list_item.main_box.show(),
                SideBarListItemPermanentState::Starred => sidebar_list_item.main_box.show(),
                SideBarListItemPermanentState::Deleted => {}
            }
        });
    }
}

/*
- Make multiple states enum, normal, editing, deleting, archived, starred
*/
#[derive(Clone, Debug)]
enum SideBarListItemPermanentState {
    Normal,
    Archived,
    Starred,
    Deleted,
}

/*
- Archive/Unarchive widgets
*/
#[derive(Clone, Debug)]
pub struct SideBarListItem {
    main_box: gtk::Box,
    permanent_state: Arc<Mutex<SideBarListItemPermanentState>>,
}

impl SideBarListItem {
    pub fn new(
        file_path: PathBuf,
        conversation_file_option_sender: Sender<Option<PathBuf>>,
    ) -> Self {
        let (permanent_state, conversation_name) = Self::initialise_state_and_name(&file_path);
        let main_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(0)
            .build();
        let open_button = gtk::Button::builder()
            .label(&conversation_name)
            .has_tooltip(true)
            .hexpand(true)
            .tooltip_text("Open Conversation")
            .build();
        let star_button = gtk::Button::builder()
            .icon_name("non-starred-symbolic")
            .has_tooltip(true)
            .tooltip_text("Favourite")
            .build();
        let archive_button = gtk::Button::builder()
            .label("Archive")
            .has_tooltip(true)
            .tooltip_text("Archive")
            .build();
        let menu_button = gtk::Button::builder()
            .icon_name("view-more-symbolic")
            .build();

        let menu_popover = Self::create_menu_popover(
            &main_box,
            &permanent_state,
            &file_path,
            conversation_name,
            &open_button,
        );
        menu_popover.set_parent(&menu_button);
        menu_button.connect_clicked(move |_| {
            menu_popover.popup();
        });

        main_box.append(&open_button);
        main_box.append(&archive_button);
        main_box.append(&star_button);
        main_box.append(&menu_button);

        let file_path_for_button = file_path.clone();
        open_button.connect_clicked(move |_| {
            conversation_file_option_sender
                .send(Some(file_path_for_button.clone()))
                .unwrap();
        });

        {
            let file_path = file_path.clone();
            let archive_button = archive_button.clone();
            let main_box = main_box.clone();
            let permanent_state = Arc::clone(&permanent_state);
            star_button.connect_clicked(move |star_button| {
                Self::toggle_favourite(
                    &file_path,
                    star_button,
                    &archive_button,
                    &main_box,
                    Arc::clone(&permanent_state),
                )
            });
        }
        {
            let file_path = file_path.clone();
            let star_button = star_button.clone();
            let main_box = main_box.clone();
            let permanent_state = Arc::clone(&permanent_state);

            archive_button.connect_clicked(move |archive_button| {
                Self::toggle_archive(
                    &file_path,
                    &star_button,
                    archive_button,
                    &main_box,
                    Arc::clone(&permanent_state),
                )
            });
        }
        match *permanent_state.lock().unwrap() {
            SideBarListItemPermanentState::Normal => {}
            SideBarListItemPermanentState::Archived => {
                archive_button.set_label("Unarchive");
                archive_button.set_tooltip_text(Some("Unarchive"));
                main_box.hide();
            }
            SideBarListItemPermanentState::Starred => {
                star_button.set_icon_name("starred-symbolic");
                star_button.set_tooltip_text(Some("Unfavourite"));
            }
            SideBarListItemPermanentState::Deleted => {
                main_box.hide();
            }
        }
        Self {
            main_box,
            permanent_state,
        }
    }
    fn rename_conversation(file_path: &PathBuf, new_name: &str, open_button: &gtk::Button) {
        if let Some(mut loaded_conversation) = SavedConversation::load(file_path) {
            new_name.clone_into(&mut loaded_conversation.name);
            loaded_conversation.save(file_path);
            open_button.set_label(new_name);
            println!("Renamed {}", new_name);
        }
    }

    fn delete_conversation(
        main_box: &gtk::Box,
        file_path: &PathBuf,
        permanent_state: Arc<Mutex<SideBarListItemPermanentState>>,
    ) {
        main_box.hide();
        *permanent_state.lock().unwrap() = SideBarListItemPermanentState::Deleted;
        let file_absolute =
            utils::get_root_folder().join(PathBuf::from("conversations").join(file_path));
        fs::remove_file(file_absolute).unwrap();
    }

    fn toggle_favourite(
        file_path: &PathBuf,
        star_button: &gtk::Button,
        archive_button: &gtk::Button,
        main_box: &gtk::Box,
        permanent_state: Arc<Mutex<SideBarListItemPermanentState>>,
    ) {
        let current_state = (*permanent_state.lock().unwrap()).clone();
        let archived_val_to_write;
        let starred_val_to_write;
        if let SideBarListItemPermanentState::Starred = current_state {
            *permanent_state.lock().unwrap() = SideBarListItemPermanentState::Normal;
            archived_val_to_write = false;
            starred_val_to_write = false;
            star_button.set_icon_name("non-starred-symbolic");
            star_button.set_tooltip_text(Some("Favourite"));
        } else {
            *permanent_state.lock().unwrap() = SideBarListItemPermanentState::Starred;
            archived_val_to_write = false;
            starred_val_to_write = true;
            star_button.set_icon_name("starred-symbolic");
            star_button.set_tooltip_text(Some("Unfavourite"));
            archive_button.set_label("Archive");
            archive_button.set_tooltip_text(Some("Archive"));
            main_box.show();
        }
        if let Some(mut loaded_conversation) = SavedConversation::load(file_path) {
            loaded_conversation.archived = archived_val_to_write;
            loaded_conversation.starred = starred_val_to_write;
            loaded_conversation.save(file_path);
        }
    }
    fn toggle_archive(
        file_path: &PathBuf,
        star_button: &gtk::Button,
        archive_button: &gtk::Button,
        main_box: &gtk::Box,
        permanent_state: Arc<Mutex<SideBarListItemPermanentState>>,
    ) {
        let current_state = (*permanent_state.lock().unwrap()).clone();
        let archived_val_to_write;
        let starred_val_to_write;
        if let SideBarListItemPermanentState::Archived = current_state {
            *permanent_state.lock().unwrap() = SideBarListItemPermanentState::Normal;
            archived_val_to_write = false;
            starred_val_to_write = false;
            archive_button.set_label("Archive");
            archive_button.set_tooltip_text(Some("Archive"));
            main_box.show();
        } else {
            *permanent_state.lock().unwrap() = SideBarListItemPermanentState::Archived;
            archived_val_to_write = true;
            starred_val_to_write = false;
            archive_button.set_label("Unarchive");
            archive_button.set_tooltip_text(Some("Unarchive"));
            star_button.set_icon_name("non-starred-symbolic");
            star_button.set_tooltip_text(Some("Favourite"));
            main_box.hide();
        }
        if let Some(mut loaded_conversation) = SavedConversation::load(file_path) {
            loaded_conversation.archived = archived_val_to_write;
            loaded_conversation.starred = starred_val_to_write;
            loaded_conversation.save(file_path);
        }
    }

    fn initialise_state_and_name(
        file_path: &PathBuf,
    ) -> (Arc<Mutex<SideBarListItemPermanentState>>, String) {
        let permanent_state;
        let conversation_name: String;
        if let Some(loaded_conversation) = SavedConversation::load(file_path) {
            conversation_name = loaded_conversation.name;
            if loaded_conversation.archived {
                permanent_state = Arc::new(Mutex::new(SideBarListItemPermanentState::Archived));
            } else if loaded_conversation.starred {
                permanent_state = Arc::new(Mutex::new(SideBarListItemPermanentState::Starred));
            } else {
                permanent_state = Arc::new(Mutex::new(SideBarListItemPermanentState::Normal));
            }
        } else {
            conversation_name = file_path
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap_or("Error reading filename")
                .to_owned();
            permanent_state = Arc::new(Mutex::new(SideBarListItemPermanentState::Normal));
        }
        (permanent_state, conversation_name)
    }

    fn create_menu_popover(
        main_box: &gtk::Box,
        permanent_state: &Arc<Mutex<SideBarListItemPermanentState>>,
        file_path: &PathBuf,
        conversation_name: String,
        open_button: &gtk::Button,
    ) -> gtk::Popover {
        let delete_button = gtk::Button::builder()
            .icon_name("edit-delete-symbolic")
            .has_tooltip(true)
            .tooltip_text("Delete")
            .build();
        let rename_button = gtk::Button::builder()
            .icon_name("document-edit-symbolic")
            .has_tooltip(true)
            .tooltip_text("Rename")
            .build();
        let menu_box = gtk::Box::builder()
            .spacing(2)
            .orientation(gtk::Orientation::Horizontal)
            .build();
        menu_box.append(&rename_button);
        menu_box.append(&delete_button);

        let menu_popover = gtk::Popover::builder().autohide(true).build();
        menu_popover.set_position(gtk::PositionType::Right);
        menu_popover.set_child(Some(&menu_box));

        let delete_box = Self::create_delete_box(
            main_box,
            permanent_state,
            file_path,
            &menu_popover,
            &menu_box,
        );

        let rename_box = Self::create_rename_box(
            conversation_name,
            &menu_popover,
            menu_box,
            open_button,
            file_path,
        );
        let menu_popover_for_rename = menu_popover.clone();
        rename_button.connect_clicked(move |_| {
            menu_popover_for_rename.set_child(Some(&rename_box));
        });

        let menu_popover_for_delete = menu_popover.clone();
        delete_button.connect_clicked(move |_| {
            menu_popover_for_delete.set_child(Some(&delete_box));
        });
        menu_popover
    }

    fn create_rename_box(
        conversation_name: String,
        menu_popover: &gtk::Popover,
        menu_box: gtk::Box,
        open_button: &gtk::Button,
        file_path: &PathBuf,
    ) -> gtk::Box {
        let rename_buffer = gtk::EntryBuffer::new(Some(&conversation_name));
        let rename_entry = gtk::Entry::with_buffer(&rename_buffer);
        let confirm_rename_button = gtk::Button::builder()
            .icon_name("emblem-ok-symbolic")
            .build();
        let cancel_rename_button = gtk::Button::builder()
            .icon_name("process-stop-symbolic")
            .build();

        {
            let menu_popover = menu_popover.clone();
            let menu_box = menu_box.clone();
            let open_button = open_button.clone();
            let file_path = file_path.clone();
            let rename_buffer = rename_buffer.clone();
            confirm_rename_button.connect_clicked(move |_| {
                let new_name = rename_buffer.text().to_string();
                SideBarListItem::rename_conversation(&file_path, &new_name, &open_button);
                menu_popover.set_child(Some(&menu_box));
                menu_popover.popdown();
            });
        }
        {
            let menu_popover = menu_popover.clone();
            let menu_box = menu_box.clone();
            let open_button = open_button.clone();
            let file_path = file_path.clone();
            let rename_buffer = rename_buffer.clone();
            rename_entry.connect_activate(move |_| {
                let new_name = rename_buffer.text().to_string();
                SideBarListItem::rename_conversation(&file_path, &new_name, &open_button);
                menu_popover.set_child(Some(&menu_box));
                menu_popover.popdown();
            });
        }
        {
            let menu_popover = menu_popover.clone();
            cancel_rename_button.connect_clicked(move |_| {
                menu_popover.set_child(Some(&menu_box));
                menu_popover.popdown();
            });
        }

        let rename_box = gtk::Box::builder()
            .spacing(2)
            .orientation(gtk::Orientation::Horizontal)
            .build();
        rename_box.append(&rename_entry);
        rename_box.append(&confirm_rename_button);
        rename_box.append(&cancel_rename_button);
        rename_box
    }

    fn create_delete_box(
        main_box: &gtk::Box,
        permanent_state: &Arc<Mutex<SideBarListItemPermanentState>>,
        file_path: &PathBuf,
        menu_popover: &gtk::Popover,
        menu_box: &gtk::Box,
    ) -> gtk::Box {
        let confirm_delete_button = gtk::Button::builder().label("Delete").hexpand(true).build();
        let cancel_delete_button = gtk::Button::builder().label("Cancel").hexpand(true).build();
        {
            let main_box = main_box.clone();
            let permanent_state = Arc::clone(permanent_state);
            let file_path = file_path.clone();
            confirm_delete_button.connect_clicked(move |_| {
                SideBarListItem::delete_conversation(
                    &main_box,
                    &file_path,
                    Arc::clone(&permanent_state),
                )
            });
        }
        {
            let menu_popover = menu_popover.clone();
            let menu_box = menu_box.clone();
            cancel_delete_button.connect_clicked(move |_| {
                menu_popover.set_child(Some(&menu_box));
                menu_popover.popdown();
            });
        }

        let delete_box = gtk::Box::builder()
            .spacing(2)
            .orientation(gtk::Orientation::Horizontal)
            .build();
        delete_box.append(&confirm_delete_button);
        delete_box.append(&cancel_delete_button);
        delete_box
    }
}
