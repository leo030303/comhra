use adw::prelude::*;

use crate::models::Message;
use arboard::Clipboard;

/*
- Editable field/label for text
- Label for user/assistant
- Button to show rag content
- Collapsible label for rag content
- Button for copy
- Button for edit
- Button for regenerate
*/
pub struct ChatMessageListItem {
    pub main_box: gtk::Box,
    pub content_textbox: gtk::TextView,
    role_label: gtk::Label,
}

impl ChatMessageListItem {
    pub fn new(chat_message_option: Option<Message>) -> Self {
        let chat_message_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(10)
            .build();

        let copy_button = gtk::Button::builder()
            .icon_name("edit-copy-symbolic")
            .tooltip_text("Copy message")
            .build();
        let edit_button = gtk::Button::builder()
            .icon_name("document-edit-symbolic")
            .tooltip_text("Edit message")
            .build();
        let chat_role_label = gtk::Label::builder()
            .label("")
            .halign(gtk::Align::Center)
            .width_chars(12)
            .build();
        let chat_message_side_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(5)
            .build();
        chat_message_side_box.append(&chat_role_label);
        chat_message_side_box.append(&edit_button);
        chat_message_side_box.append(&copy_button);

        let chat_content_buffer = gtk::TextBuffer::builder()
            .enable_undo(true)
            .text("")
            .build();
        let chat_content_textbox = gtk::TextView::builder()
            .editable(false)
            .cursor_visible(false)
            .buffer(&chat_content_buffer)
            .hexpand(true)
            .vexpand(true)
            .build();

        copy_button.connect_clicked(move |_| {
            let mut clipboard = Clipboard::new().unwrap();
            clipboard
                .set_text(
                    chat_content_buffer
                        .text(
                            &chat_content_buffer.start_iter(),
                            &chat_content_buffer.end_iter(),
                            true,
                        )
                        .as_str(),
                )
                .unwrap();
        });

        chat_message_box.append(&chat_message_side_box);
        chat_message_box.append(&chat_content_textbox);
        let mut chat_message_list_item = Self {
            main_box: chat_message_box,
            content_textbox: chat_content_textbox,
            role_label: chat_role_label,
        };
        if let Some(chat_message) = chat_message_option {
            chat_message_list_item.update_message(chat_message);
        }
        chat_message_list_item
    }

    pub fn update_message(&mut self, chat_message: Message) {
        match chat_message.role {
            crate::models::Role::User => {
                self.role_label.add_css_class("user-label");
                self.role_label.set_text("User");
                self.content_textbox
                    .buffer()
                    .set_text(&chat_message.content);
            }
            crate::models::Role::Assistant => {
                self.role_label.add_css_class("assistant-label");
                self.role_label.set_text("Assistant");
                self.content_textbox
                    .buffer()
                    .set_text(&chat_message.content);
            }
            crate::models::Role::System => {
                self.role_label.add_css_class("system-label");
                self.role_label.set_text("System");
                self.content_textbox
                    .buffer()
                    .set_text(&chat_message.content);
            }
        }
    }
}
