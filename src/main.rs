pub mod crypto;
pub mod file;
pub mod toast;

use std::path::PathBuf;
use std::sync::Arc;

use crypto::{decrypt, encrypt};
use file::{get_file_path, pathbuf_to_string, pick_file, save_file, FileError};
use toast::{Status, Toast};

use iced::widget::{button, column, container, row, text, text_editor, text_input, tooltip};
use iced::{Command, Element, Font, Length, Subscription};
use iced::keyboard;

pub fn main() -> iced::Result {
    iced::program("CryptoDoc", CryptoDoc::update, CryptoDoc::view)
        .subscription(CryptoDoc::subscription)
        .font(include_bytes!("../fonts/icons.ttf").as_slice())
        .run()
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
    is_dirty: bool,
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
            is_dirty: false,
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
                if self.doc_name == String::new() {
                    self.toasts.push(Toast {
                        title: "Failed".into(),
                        body: "Open a document first.".into(),
                        status: Status::Danger,
                    });

                    Command::none()
                } else {

                    let text = self.content.text();

                    let res = encrypt(text.as_bytes(), &self.password);

                    self.toasts.push(Toast {
                        title: "Success".into(),
                        body: "Document was saved.".into(),
                        status: Status::Success,
                    });

                    Command::perform(
                        save_file(Some(get_file_path(&self.doc_name)).clone(), res),
                        Message::FileSaved,
                    )
                }
            }

            Message::Edit(action) => {
                self.is_dirty = self.is_dirty || action.is_edit();

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
                self.is_dirty = false;
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
                self.is_dirty = false;

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
            action(new_icon(), "New File", Some(Message::NewDocumentPressed)),
            action(open_icon(), "Open File", Some(Message::OpenDocumentPressed)),
            action(save_icon(), "Save File", self.is_dirty.then_some(Message::SaveDocumentPressed)),
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

    fn subscription(&self) -> Subscription<Message> {
        keyboard::on_key_press(|key, modifiers| match key.as_ref() {
            keyboard::Key::Character("s") if modifiers.command() => {
                Some(Message::SaveDocumentPressed)
            }
            _ => None,
        })
    }
}

impl Default for CryptoDoc {
    fn default() -> Self {
        Self::new()
    }
}

fn action<'a>(
    content: Element<'a, Message>,
    label: &'a str,
    on_press: Option<Message>,
 ) -> Element<'a, Message> {
    let action = button(container(content).width(30).center_x());

    if let Some(on_press) = on_press {
        tooltip(
            action.on_press(on_press),
            label,
            tooltip::Position::FollowCursor,
        )
        .style(container::rounded_box)
        .into()
    } else {
        action.style(button::secondary).into()
    }
}

fn new_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0e800}')
}

fn save_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0e801}')
}

fn open_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0f115}')
}

fn icon<'a, Message>(codepoint: char) -> Element<'a, Message> {
    const ICON_FONT: Font = Font::with_name("editor-icons");

    text(codepoint).font(ICON_FONT).into()
}

