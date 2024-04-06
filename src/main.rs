pub mod crypto;
pub mod file;
pub mod toast;

use std::path::PathBuf;
use std::sync::Arc;

use crypto::{decrypt, encrypt};
use file::{get_file_path, pathbuf_to_string, pick_file, save_file, FileError};
use toast::{Status, Toast};

use iced::widget::{button, column, container, row, text, text_editor, text_input};
use iced::{Command, Element, Length};

pub fn main() -> iced::Result {
    iced::run("CryptoDoc", CryptoDoc::update, CryptoDoc::view)
}

struct CryptoDoc {
    current_page: Page,
    content: text_editor::Content,
    encrypted_content: String,
    doc_name: String,
    password: String,
    error: Option<FileError>,
    path: Option<PathBuf>,
    toasts: Vec<Toast>,
}

#[derive(Debug, Clone)]
enum Page {
    StartPage,
    NewDocumentPage,
    DocumentViewer,
    AskPassword,
}

#[derive(Debug, Clone)]
enum Message {
    NewDocumentPressed,
    OpenDocumentPressed,
    SaveDocumentPressed,
    NewDocumentSubmitted,
    TryDecrypt,
    CloseToast(usize),
    DocumentInput(String),
    NewDocumentPasswordInput(String),
    PasswordInput(String),
    Edit(text_editor::Action),
    FileOpened(Result<(PathBuf, Arc<String>), FileError>),
    FileSaved(Result<PathBuf, FileError>),
}

impl CryptoDoc {
    fn new() -> Self {
        Self {
            toasts: vec![],
            current_page: Page::StartPage,
            content: text_editor::Content::new(),
            encrypted_content: String::new(),
            doc_name: String::new(),
            password: String::new(),
            error: None,
            path: None,
        }
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::NewDocumentPressed => {
                self.content = text_editor::Content::new();
                self.doc_name = String::new();
                self.password = String::new();
                
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
                self.password = String::new();

                self.path = Some(path.clone());

                self.encrypted_content = (&content.as_str()).to_string();

                self.doc_name = pathbuf_to_string(&path);

                self.current_page = Page::AskPassword;

                Command::none()
            }

            Message::FileOpened(Err(error)) => {
                self.error = Some(error);

                Command::none()
            }

            Message::NewDocumentPasswordInput(password) => {
                self.password = password;

                Command::none()
            }

            Message::TryDecrypt => {
                let decrypted_result = decrypt(&self.encrypted_content.as_str(), &self.password);

                match decrypted_result {
                    Ok((result, decrypted_vec)) => {
                        if !result {
                            self.toasts.push(Toast {
                                title: "Failed".into(),
                                body: "Password is incorrect.".into(),
                                status: Status::Danger,
                            })
                        } else {
                            let decrypted_text =
                                String::from_utf8(decrypted_vec).expect("Failed to convert to vec");
                            self.content = text_editor::Content::with_text(&decrypted_text);
                            self.current_page = Page::DocumentViewer;
                        }
                    }
                    Err(_) => {
                        println!("Failed to decrypt");
                    }
                }

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

            Message::CloseToast(index) => {
                self.toasts.remove(index);

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

                let content = container(column![controls, placeholder_text].spacing(10))
                    .padding(10)
                    .center_x()
                    .center_y();

                toast::Manager::new(content, &self.toasts, Message::CloseToast).into()
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

                let content = container(
                    column![controls, name_title, name_input, pass_title, pass_input, submit_btn]
                        .spacing(10),
                )
                .padding(10)
                .center_x()
                .center_y();

                toast::Manager::new(content, &self.toasts, Message::CloseToast).into()
            }
            Page::DocumentViewer => {
                let title = text(format!("Current Document: {}", self.doc_name));
                let editor = text_editor(&self.content)
                    .on_action(Message::Edit)
                    .height(Length::Fill);

                let content = container(column![controls, title, editor].spacing(10))
                    .padding(10)
                    .center_x()
                    .center_y();

                toast::Manager::new(content, &self.toasts, Message::CloseToast).into()
            }
            Page::AskPassword => {
                let title = text(format!(
                    "Enter the password for: {}",
                    self.path
                        .as_ref()
                        .map_or(String::from(""), |p| pathbuf_to_string(p))
                ));

                let pass_input = text_input("Password", &self.password)
                    .padding(10)
                    .on_input(Message::NewDocumentPasswordInput)
                    .secure(true);

                let submit_btn = button("Submit").on_press(Message::TryDecrypt);

                let content =
                    container(column![controls, title, pass_input, submit_btn].spacing(10))
                        .padding(10)
                        .center_x()
                        .center_y();

                toast::Manager::new(content, &self.toasts, Message::CloseToast).into()
            }
        }
    }
}

impl Default for CryptoDoc {
    fn default() -> Self {
        CryptoDoc::new()
    }
}
