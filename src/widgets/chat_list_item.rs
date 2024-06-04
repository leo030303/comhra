use adw::prelude::*;

use crate::models::Message;

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
    pub content_label: gtk::Label,
    role_label: gtk::Label,
}

impl ChatMessageListItem {
    pub fn new(chat_message_option: Option<Message>) -> Self {
        let chat_message_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(10)
            .build();

        let chat_role_label = gtk::Label::builder()
            .label("")
            .halign(gtk::Align::Center)
            .width_chars(12)
            .build();
        let chat_content_label = gtk::Label::builder()
            .wrap(true)
            .label("")
            .halign(gtk::Align::Start)
            .hexpand(true)
            .build();
        chat_message_box.append(&chat_role_label);
        chat_message_box.append(&chat_content_label);
        let mut chat_message_list_item = Self {
            main_box: chat_message_box,
            content_label: chat_content_label,
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
                self.content_label.set_text(&chat_message.content);
            }
            crate::models::Role::Assistant => {
                self.role_label.add_css_class("assistant-label");
                self.role_label.set_text("Assistant");
                self.content_label.set_text(&chat_message.content);
            }
            crate::models::Role::System => {
                self.role_label.add_css_class("system-label");
                self.role_label.set_text("System");
                self.content_label.set_text(&chat_message.content);
            }
        }
    }
}
