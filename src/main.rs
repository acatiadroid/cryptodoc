pub mod crypto;

use crypto::{encrypt, decrypt};
use iced::widget::{
    button, column, container, row, text, text_editor, text_input,
};
use iced::{Command, Element, Length};

pub fn main() -> iced::Result {
    iced::run("CryptoDoc", CryptoDoc::update, CryptoDoc::view)
}

struct CryptoDoc {
    current_page: Page,
    content: text_editor::Content,
    password: String,
}

#[derive(Debug, Clone)]
enum Page {
    StartPage,
    NewDocumentPage,
    DocumentViewer,
}

#[derive(Debug, Clone)]
enum Message {
    NewDocumentPressed,
    OpenDocumentPressed,
    SaveDocumentPressed,
    PasswordSubmitted,
    PasswordInput(String),
    Edit(text_editor::Action),
}

impl CryptoDoc {
    fn new() -> Self {
        Self {
            current_page: Page::StartPage,
            content: text_editor::Content::new(),
            password: String::new(),
        }
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::NewDocumentPressed => {
                self.current_page = Page::NewDocumentPage;

                Command::none()
            }

            Message::OpenDocumentPressed => {
                self.current_page = Page::DocumentViewer;
                Command::none()
            }

            Message::SaveDocumentPressed => {
                let text = self.content.text();

                let res = encrypt(text.as_bytes(), &self.password);

                println!("{}", res);

                Command::none()
                // Command::perform(future, f)
            }

            Message::Edit(action) => {
                self.content.perform(action);

                Command::none()
            }

            Message::PasswordInput(content) => {
                self.password = content;

                Command::none()
            },
            Message::PasswordSubmitted => {
                self.current_page = Page::DocumentViewer;
                
                Command::none()
            }

        }
    }

    fn view(&self) -> Element<Message> {
        let controls = row![
            button("New Doc").on_press(Message::NewDocumentPressed),
            button("Open Doc").on_press(Message::OpenDocumentPressed),
            button("Save Doc").on_press(Message::SaveDocumentPressed),
        ]
        .spacing(10);

        match self.current_page {
            Page::StartPage => {
                let placeholder_text = text("Click to get started.");

                container(column![controls, placeholder_text].spacing(10))
                    .padding(10)
                    .center_x()
                    .center_y()
                    .into()
            }

            Page::NewDocumentPage => {
                let title = text("Enter a document password, then press enter.");

                let text_input =
                    text_input("Password", &self.password)   
                        .padding(10)
                        .on_input(Message::PasswordInput)
                        .on_submit(Message::PasswordSubmitted)
                        .secure(true);

                container(column![controls, title, text_input].spacing(10))
                    .padding(10)
                    .center_x()
                    .center_y()
                    .into()
            }
            Page::DocumentViewer => {
                let title = text("Document Editor");
                let editor = text_editor(&self.content)
                    .on_action(Message::Edit)
                    .height(Length::Fill);

                container(column![controls, title, editor].spacing(10))
                    .padding(10)
                    .center_x()
                    .center_y()
                    .into()
            }
        }

        // container(
        //     column![controls].spacing(10)
        // ).padding(10).into()
    }
}

impl Default for CryptoDoc {
    fn default() -> Self {
        CryptoDoc::new()
    }
}
