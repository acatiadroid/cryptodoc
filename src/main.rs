mod crypto;
mod file;
mod icons;
mod toast;

use std::path::PathBuf;
use std::sync::Arc;

use crypto::{decrypt, encrypt};
use file::{
    get_file_path, get_save_file_path, pathbuf_to_string, pick_file, pick_folder, save_file,
    FileError,
};
use icons::{action, home_icon, new_icon, open_icon, save_icon, settings_icon};
use toast::{Status, Toast};

use iced::keyboard;
use iced::widget::{
    button, column, container, horizontal_space, pick_list, row, text, text_editor, text_input,
};
use iced::window;
use iced::Theme;
use iced::{highlighter, Settings};
use iced::{Command, Element, Length, Subscription};
use image::GenericImageView;

pub fn main() -> iced::Result {
    static ICON: &[u8] = include_bytes!("../assets/app_icon.png");

    let image = image::load_from_memory(ICON).unwrap();
    let (width, height) = image.dimensions();
    let rgba = image.into_rgba8();
    let icon = window::icon::from_rgba(rgba.into_raw(), width, height).unwrap();

    let settings = Settings {
        window: iced::window::Settings {
            icon: Some(icon),
            ..Default::default()
        },
        ..Default::default()
    };

    iced::program("CryptoDoc", CryptoDoc::update, CryptoDoc::view)
        .subscription(CryptoDoc::subscription)
        .theme(CryptoDoc::theme)
        .settings(settings)
        .window_size((900.0, 700.0))
        .font(include_bytes!("../assets/icons.ttf").as_slice())
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
    save_path: String,
    theme: highlighter::Theme,
}

#[derive(Debug, Clone)]
enum Page {
    StartPage,
    NewDocumentPage,
    DocumentViewer,
    AskPassword,
    Settings,
}

#[derive(Debug, Clone)]
enum Message {
    NewDocumentPressed,
    OpenDocumentPressed,
    SaveDocumentPressed,
    SettingsPressed,
    HomePressed,
    NewDocumentSubmitted,
    TryDecrypt,
    SelectFolderPressed,
    CloseToast(usize),
    DocumentInput(String),
    NewDocumentPasswordInput(String),
    PasswordInput(String),
    Edit(text_editor::Action),
    FileOpened(Result<(PathBuf, Arc<String>), FileError>),
    FileSaved(Result<PathBuf, FileError>),
    FolderPathFileSaved(Result<PathBuf, FileError>),
    FolderSelected(Result<PathBuf, FileError>),
    ThemeSelected(highlighter::Theme),
}

impl CryptoDoc {
    fn new() -> Self {
        let save_path =
            std::fs::read_to_string(get_save_file_path()).unwrap_or_else(|_| String::new());

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
            save_path,
            theme: highlighter::Theme::SolarizedDark,
        }
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::ThemeSelected(theme) => {
                self.theme = theme;

                Command::none()
            }

            Message::HomePressed => {
                self.doc_name = String::new();
                self.content = text_editor::Content::new();
                self.password = String::new();
                self.current_page = Page::StartPage;

                Command::none()
            }
            Message::NewDocumentPressed => {
                self.content = text_editor::Content::new();
                self.doc_name = String::new();
                self.password = String::new();

                self.current_page = Page::NewDocumentPage;

                Command::none()
            }

            Message::SelectFolderPressed => {
                Command::perform(pick_folder(), Message::FolderSelected)
            }

            Message::SettingsPressed => {
                self.current_page = Page::Settings;

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

                    let path = get_file_path().unwrap_or_else(|_| PathBuf::new());
                    let mut full_path = path.join(&self.doc_name);
                    full_path.set_extension("cryptodoc");

                    Command::perform(save_file(Some(full_path), res), Message::FileSaved)
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

            Message::FolderSelected(Ok(path)) => {
                self.save_path = pathbuf_to_string(&path);

                Command::perform(
                    save_file(Some(get_save_file_path()), pathbuf_to_string(&path)),
                    Message::FolderPathFileSaved,
                )
            }
            Message::FolderSelected(Err(_)) => {
                self.toasts.push(Toast {
                    title: "Failed".into(),
                    body: "Couldn't select specified folder.".into(),
                    status: Status::Danger,
                });

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

                self.toasts.push(Toast {
                    title: "Success".into(),
                    body: "Document has been saved.".into(),
                    status: Status::Success,
                });

                Command::none()
            }

            Message::FileSaved(Err(error)) => {
                self.error = Some(error);

                self.toasts.push(Toast {
                    title: "Failed".into(),
                    body: format!("Failed to save: {:?}", &self.error).into(),
                    status: Status::Danger,
                });

                Command::none()
            }

            Message::FolderPathFileSaved(Ok(_)) => {
                self.toasts.push(Toast {
                    title: "Success".into(),
                    body: "Document save path has been saved.".into(),
                    status: Status::Success,
                });

                Command::none()
            }

            Message::FolderPathFileSaved(Err(_)) => {
                self.toasts.push(Toast {
                    title: "Failed".into(),
                    body: "Couldn't save document path.".into(),
                    status: Status::Danger,
                });

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
            action(home_icon(), "Home", Some(Message::HomePressed), true),
            action(
                new_icon(),
                "New File",
                Some(Message::NewDocumentPressed),
                false
            ),
            action(
                open_icon(),
                "Open File",
                Some(Message::OpenDocumentPressed),
                false
            ),
            action(
                save_icon(),
                "Save File",
                self.is_dirty.then_some(Message::SaveDocumentPressed),
                false
            ),
            horizontal_space(),
            action(
                settings_icon(),
                "Settings",
                Some(Message::SettingsPressed),
                false
            )
        ]
        .spacing(10);

        match self.current_page {
            Page::Settings => {
                let save_title = text("Directory to save documents into:");

                let save_button = button("Select Path").on_press(Message::SelectFolderPressed);

                let current_path = text(format!("Current Path: {}", &self.save_path));

                let save_row = row![save_button, current_path].spacing(10);

                let theme_title = text("Theme:");

                let theme_list = pick_list(
                    highlighter::Theme::ALL,
                    Some(self.theme),
                    Message::ThemeSelected,
                )
                .text_size(14)
                .padding([5, 10]);

                let content = container(
                    column![controls, save_title, save_row, theme_title, theme_list].spacing(10),
                )
                .padding(10);

                toast::Manager::new(content, &self.toasts, Message::CloseToast).into()
            }

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

    fn theme(&self) -> Theme {
        if self.theme.is_dark() {
            Theme::Dark
        } else {
            Theme::Light
        }
    }
}

impl Default for CryptoDoc {
    fn default() -> Self {
        Self::new()
    }
}
