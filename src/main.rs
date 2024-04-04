pub mod crypto;
pub mod file;

use std::path::PathBuf;
use std::sync::Arc;

use crypto::{decrypt, encrypt};
use file::{get_file_path, load_file, pick_file, save_file, FileError};
use iced::widget::{button, column, container, row, text, text_editor, text_input};
use iced::{Command, Element, Length};

pub fn main() -> iced::Result {
    iced::run("CryptoDoc", CryptoDoc::update, CryptoDoc::view)
}

struct CryptoDoc {
    current_page: Page,
    content: text_editor::Content,
    doc_name: String,
    password: String,
    error: Option<FileError>,
    path: Option<PathBuf>,
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
    NewDocumentSubmitted,
    DocumentInput(String),
    PasswordInput(String),
    Edit(text_editor::Action),
    FileOpened(Result<(PathBuf, Arc<String>), FileError>),
    FileSaved(Result<PathBuf, FileError>),
}

impl CryptoDoc {
    fn new() -> Self {
        Self {
            current_page: Page::StartPage,
            content: text_editor::Content::new(),
            doc_name: String::new(),
            password: String::new(),
            error: None,
            path: None,
        }
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::NewDocumentPressed => {
                self.current_page = Page::NewDocumentPage;

                Command::none()
            }

            Message::OpenDocumentPressed => Command::perform(pick_file(), Message::FileOpened),

            Message::SaveDocumentPressed => {
                let text = self.content.text();

                let res = encrypt(text.as_bytes(), &self.password);

                Command::perform(
                    save_file(Some(get_file_path(&self.doc_name)).clone(), res),
                    Message::FileSaved,
                )
            }

            Message::Edit(action) => {
                self.content.perform(action);

                Command::none()
            }

            Message::DocumentInput(content) => {
                self.doc_name = content;

                Command::none()
            }

            Message::PasswordInput(content) => {
                self.password = content;

                Command::none()
            }

            Message::NewDocumentSubmitted => {
                self.current_page = Page::DocumentViewer;

                Command::none()
            }

            Message::FileOpened(Ok((path, content))) => {
                self.path = Some(path);
                self.content = text_editor::Content::with_text(&content);
                Command::none()
            }

            Message::FileOpened(Err(error)) => {
                self.error = Some(error);

                Command::none()
            }

            Message::FileSaved(Ok(path)) => {
                self.path = Some(path);

                Command::none()
            }

            Message::FileSaved(Err(error)) => {
                self.error = Some(error);

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
                let name_title = text("Enter the new document name:");

                let name_input = text_input("Document Name", &self.doc_name)
                    .padding(10)
                    .on_input(Message::DocumentInput);

                let pass_title = text("Enter a document password:");

                let pass_input = text_input("Password", &self.password)
                    .padding(10)
                    .on_input(Message::PasswordInput)
                    .secure(true);

                let submit_btn = button("Create").on_press(Message::NewDocumentSubmitted);

                container(
                    column![controls, name_title, name_input, pass_title, pass_input, submit_btn].spacing(10),
                )
                .padding(10)
                .center_x()
                .center_y()
                .into()
            }
            Page::DocumentViewer => {
                let title = text(format!("Current Document: {}", self.doc_name));
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
