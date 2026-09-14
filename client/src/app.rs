use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::text::{Ellipsis, Wrapping};
use iced::widget::{column, container, row, rule, scrollable, text};
use iced::{Alignment, Length, Task, padding};
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use tokio::sync::oneshot;

use crate::button::ButtonLabel;
use crate::theme::widget::button::ButtonClass;
use crate::theme::{self, Theme};
use crate::{Element, font};
use crate::{icon, util};

const SERVER_PORT: u16 = 8080;

pub struct App {
    config: common::Config,
    folders: server::FolderHandle,
    app_state: server::AppState,
    server_state: ServerState,
    shutdown_tx: Option<oneshot::Sender<()>>,
    local_ip: Option<IpAddr>,
    error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ServerState {
    Stopped,
    Running,
}

impl Default for App {
    fn default() -> Self {
        let config = common::Config::load();
        let (folders, app_state) = server::FolderHandle::new(&config.folders);
        Self {
            config,
            folders,
            app_state,
            server_state: ServerState::Stopped,
            shutdown_tx: None,
            local_ip: util::detect_local_ip(),
            error: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    AddFolder,
    FolderPicked(Option<PathBuf>),
    RemoveFolder(PathBuf),
    ToggleServer,
    ServerStopped(Result<(), String>),
    CloseError,
}

impl App {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::AddFolder => Task::perform(pick_folder(), Message::FolderPicked),
            Message::FolderPicked(Some(path)) => {
                self.config.add_folder(path);
                self.update_config();
                Task::none()
            }
            Message::FolderPicked(None) => Task::none(),
            Message::RemoveFolder(path) => {
                self.config.remove_folder(&path);
                self.update_config();
                Task::none()
            }
            Message::ToggleServer => self.toggle_server(),
            Message::ServerStopped(result) => {
                self.server_state = ServerState::Stopped;
                self.shutdown_tx = None;
                if let Err(err) = result {
                    self.error = Some(format!("Server stopped with an error: {err}"));
                }
                Task::none()
            }
            Message::CloseError => {
                self.error = None;
                Task::none()
            }
        }
    }

    fn update_config(&mut self) {
        self.folders.update(&self.config.folders);

        if let Err(err) = self.config.save() {
            self.error = Some(format!("Failed to save settings: {err}"));
        }
    }

    fn toggle_server(&mut self) -> Task<Message> {
        match self.server_state {
            ServerState::Stopped => {
                self.error = None;
                let state = self.app_state.clone();
                let addr = SocketAddr::from(([0, 0, 0, 0], SERVER_PORT));
                let (shutdown_tx, shutdown_rx) = oneshot::channel();
                self.shutdown_tx = Some(shutdown_tx);
                self.server_state = ServerState::Running;

                Task::perform(
                    async move {
                        server::serve(state, addr, async {
                            let _ = shutdown_rx.await;
                        })
                        .await
                        .map_err(|err| err.to_string())
                    },
                    Message::ServerStopped,
                )
            }
            ServerState::Running => {
                if let Some(tx) = self.shutdown_tx.take() {
                    let _ = tx.send(());
                }
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let add_button = ButtonLabel::IconWithText(icon::plus(), "ADD")
            .into_button()
            .on_press(Message::AddFolder);

        let title = text("SHARED FOLDERS")
            .wrapping(Wrapping::None)
            .width(Length::Fill)
            .style(theme::widget::text::secondary);

        let header = container(row![title, add_button].align_y(Alignment::Center)).padding(15);

        let folders_section: Element<'_, Message> = if self.config.folders.is_empty() {
            text("No folders selected")
                .size(12)
                .style(theme::widget::text::secondary)
                .into()
        } else {
            column(self.config.folders.iter().map(|folder| {
                let folder_icon = icon::folder().style(theme::widget::text::secondary);

                let folder_path = text(folder.display().to_string())
                    .width(Length::Fill)
                    .wrapping(Wrapping::None)
                    .ellipsis(Ellipsis::End);

                let remove_btn = ButtonLabel::Icon(icon::trash())
                    .into_button()
                    .height(28)
                    .padding([0, 9])
                    .on_press(Message::RemoveFolder(folder.clone()))
                    .class(ButtonClass::TransparentBlue);

                container(
                    row![folder_icon, folder_path, remove_btn]
                        .spacing(10)
                        .align_y(Alignment::Center),
                )
                .height(40)
                .align_y(Alignment::Center)
                .padding(padding::left(15).right(5))
                .style(crate::theme::widget::container::light_rounded)
                .into()
            }))
            .spacing(5)
            .into()
        };

        let direction = Direction::Vertical(Scrollbar::default().width(15).scroller_width(6));
        let folders_scrollable = scrollable(container(folders_section).padding([0, 15]))
            .height(Length::Fill)
            .direction(direction);

        let (toggle_icon, toggle_label, is_running) = match self.server_state {
            ServerState::Stopped => (icon::play().size(10), "START SERVER", false),
            ServerState::Running => (icon::stop().size(10), "STOP SERVER", true),
        };

        let toggle_button = ButtonLabel::IconWithText(toggle_icon, toggle_label)
            .into_button()
            .width(160.0)
            .on_press(Message::ToggleServer)
            .class(ButtonClass::Primary);

        let status: Option<Element<'_, Message>> = if is_running {
            let url = match self.local_ip {
                Some(ip) => format!("http://{ip}:{SERVER_PORT}"),
                None => "?".to_string(),
            };

            let url = row![
                icon::circle().size(8).style(theme::widget::text::green),
                text(url).size(12).style(theme::widget::text::green)
            ]
            .align_y(Alignment::Center)
            .spacing(8);

            let qr_btn = ButtonLabel::Icon(icon::qrcode())
                .into_button()
                .class(ButtonClass::TransparentBlue);

            Some(
                row![url, qr_btn]
                    .align_y(Alignment::Center)
                    .spacing(10)
                    .into(),
            )
        } else {
            None
        };

        let error = self.error.as_ref().map(|error| {
            container(
                row![
                    text(format!("Error: {error}")).size(12).width(Length::Fill),
                    ButtonLabel::Icon(icon::cancel())
                        .into_button()
                        .padding([0, 7])
                        .height(20)
                        .on_press(Message::CloseError)
                        .class(ButtonClass::TransparentWhite)
                ]
                .spacing(10)
                .align_y(Alignment::Center),
            )
            .width(Length::Fill)
            .padding(padding::left(15).right(10).top(5).bottom(5))
            .style(theme::widget::container::red)
        });

        let footer = container(
            row![container(toggle_button).width(Length::Fill), status].align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .padding(15);

        column![
            header,
            folders_scrollable,
            error,
            rule::horizontal(1),
            footer
        ]
        .into()
    }

    pub fn theme(&self) -> Option<Theme> {
        Theme::default().into()
    }

    pub fn window_settings() -> iced::window::Settings {
        let size = iced::Size::new(500.0, 768.0);

        iced::window::Settings {
            size: size,
            min_size: Some(size),
            ..iced::window::Settings::default()
        }
    }

    pub fn settings() -> iced::Settings {
        iced::Settings {
            id: Some(common::APP_ID.into()),
            default_font: iced::Font::new("JetBrains Mono"),
            default_text_size: 14.into(),
            fonts: font::load(),
            ..iced::Settings::default()
        }
    }
}

async fn pick_folder() -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .pick_folder()
        .await
        .map(|handle| handle.path().to_path_buf())
}
