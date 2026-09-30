use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::text::{Ellipsis, Wrapping};
use iced::widget::{column, container, qr_code, row, rule, scrollable, stack, text, void};
use iced::{Alignment, Length, Task, padding};
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use tokio::net::TcpListener;
use tokio::sync::oneshot;

use crate::button::ButtonLabel;
use crate::dialog::dialog;
use crate::settings::{self, Data};
use crate::theme::button::ButtonClass;
use crate::theme::{self, Theme};
use crate::{Element, font};
use crate::{icon, util};

pub struct App {
    config: common::Config,
    folders: server::FolderHandle,
    app_state: server::AppState,
    server_state: ServerState,
    server_addr: Option<SocketAddr>,
    shutdown_tx: Option<oneshot::Sender<()>>,
    restart_pending: bool,
    local_ips: Option<Vec<IpAddr>>,
    error: Option<String>,
    qr_modal: Option<qr_code::Data>,
    add_btn_enabled: bool,
    settings: settings::Settings,
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
            server_addr: None,
            shutdown_tx: None,
            restart_pending: false,
            local_ips: util::detect_local_ips(),
            error: None,
            qr_modal: None,
            add_btn_enabled: true,
            settings: settings::Settings::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    AddFolder,
    StartFolderPicking,
    FoldersPicked(Option<Vec<PathBuf>>),
    RemoveFolder(PathBuf),
    ToggleServer,
    ServerStopped(Result<(), String>),
    CloseError,
    OpenQrModal(String),
    CloseQrModal,
    Settings(settings::Message),
}

impl Message {
    pub fn open_settings(data: settings::Data) -> Message {
        Message::Settings(settings::Message::Open(data))
    }

    pub fn close_settings() -> Message {
        Message::Settings(settings::Message::Close)
    }
}

impl App {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::AddFolder => Task::batch([
                Task::done(Message::StartFolderPicking),
                Task::perform(
                    pick_folders(self.config.folders.clone()),
                    Message::FoldersPicked,
                ),
            ]),
            Message::StartFolderPicking => {
                self.add_btn_enabled = false;
                Task::none()
            }
            Message::FoldersPicked(paths) => {
                if let Some(paths) = paths {
                    for path in paths {
                        self.config.add_folder(path);
                    }
                    self.update_config();
                }

                self.add_btn_enabled = true;
                Task::none()
            }
            Message::RemoveFolder(path) => {
                self.config.remove_folder(&path);
                self.update_config();
                Task::none()
            }
            Message::ToggleServer => self.toggle_server(),
            Message::ServerStopped(result) => {
                self.server_state = ServerState::Stopped;
                self.server_addr = None;
                self.shutdown_tx = None;

                if let Err(err) = result {
                    self.error = Some(format!("Server stopped with an error: {err}"));
                }

                if std::mem::take(&mut self.restart_pending) {
                    self.toggle_server()
                } else {
                    Task::none()
                }
            }
            Message::CloseError => {
                self.error = None;
                Task::none()
            }
            Message::OpenQrModal(url) => {
                match qr_code::Data::new(url.as_bytes()) {
                    Ok(data) => {
                        self.qr_modal = Some(data);
                    }
                    Err(err) => {
                        self.error = Some(err.to_string());
                    }
                }
                Task::none()
            }
            Message::CloseQrModal => {
                self.qr_modal = None;
                Task::none()
            }
            Message::Settings(message) => match self.settings.update(message) {
                settings::Action::None => Task::none(),
                settings::Action::Save(ip, port) => {
                    let ip_changed = self.config.set_selected_ip(ip);
                    let port_changed = self.config.set_selected_port(port);

                    if ip_changed || port_changed {
                        self.update_config();
                    }

                    if port_changed && self.server_state == ServerState::Running {
                        self.restart_server();
                    }

                    Task::done(Message::close_settings())
                }
            },
        }
    }

    fn update_config(&mut self) {
        self.folders.update(&self.config.folders);

        if let Err(err) = self.config.save() {
            self.error = Some(format!("Failed to save settings: {err}"));
        }
    }

    fn restart_server(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            self.restart_pending = true;
            let _ = tx.send(());
        }
    }

    fn toggle_server(&mut self) -> Task<Message> {
        match self.server_state {
            ServerState::Stopped => {
                self.error = None;
                let addr = SocketAddr::from(([0, 0, 0, 0], self.config.selected_port.unwrap_or(0)));

                let std_listener = match std::net::TcpListener::bind(addr) {
                    Ok(listener) => listener,
                    Err(err) => {
                        self.error = Some(err.to_string());
                        return Task::none();
                    }
                };

                if let Err(err) = std_listener.set_nonblocking(true) {
                    self.error = Some(err.to_string());
                    return Task::none();
                }

                let listener = match TcpListener::from_std(std_listener) {
                    Ok(listener) => listener,
                    Err(err) => {
                        self.error = Some(err.to_string());
                        return Task::none();
                    }
                };

                self.server_addr = listener.local_addr().ok();

                let state = self.app_state.clone();
                let (shutdown_tx, shutdown_rx) = oneshot::channel();
                self.shutdown_tx = Some(shutdown_tx);
                self.server_state = ServerState::Running;

                Task::perform(
                    async move {
                        server::serve(state, listener, async {
                            let _ = shutdown_rx.await;
                        })
                        .await
                        .map_err(|err| err.to_string())
                    },
                    Message::ServerStopped,
                )
            }
            ServerState::Running => {
                self.restart_pending = false;
                if let Some(tx) = self.shutdown_tx.take() {
                    let _ = tx.send(());
                }
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let add_button = ButtonLabel::IconWithText {
            icon: icon::plus(),
            text: text("ADD"),
        }
        .into_button()
        .on_press_maybe(self.add_btn_enabled.then_some(Message::AddFolder));

        let title = text("SHARED FOLDERS")
            .wrapping(Wrapping::None)
            .width(Length::Fill)
            .style(theme::text::secondary);

        let header = container(row![title, add_button].align_y(Alignment::Center)).padding(15);

        let folders_section: Element<'_, Message> = if self.config.folders.is_empty() {
            text("No folders selected")
                .size(12)
                .style(theme::text::secondary)
                .into()
        } else {
            column(self.config.folders.iter().map(|folder| {
                let folder_icon = icon::folder().style(theme::text::secondary);

                let path = util::display_path(folder);

                let folder_path = text(format!("{}{}", path.prefix, path.name))
                    .width(Length::Fill)
                    .wrapping(Wrapping::None)
                    .ellipsis(Ellipsis::Start);

                let remove_btn = ButtonLabel::Icon(icon::trash())
                    .into_button()
                    .height(28)
                    .padding([0, 9])
                    .on_press(Message::RemoveFolder(folder.clone()))
                    .class(ButtonClass::Transparent(
                        theme::button::BackgroundColor::Blue,
                        theme::button::TextColor::Blue,
                    ));

                container(
                    row![folder_icon, folder_path, remove_btn]
                        .spacing(10)
                        .align_y(Alignment::Center),
                )
                .height(40)
                .align_y(Alignment::Center)
                .padding(padding::left(15).right(5))
                .style(theme::container::light_rounded)
                .into()
            }))
            .spacing(5)
            .into()
        };

        let direction = Direction::Vertical(Scrollbar::default().width(15).scroller_width(6));
        let folders_scrollable =
            scrollable(container(folders_section).padding(padding::horizontal(15).bottom(5)))
                .height(Length::Fill)
                .direction(direction);

        let (toggle_icon, toggle_label, is_running) = match self.server_state {
            ServerState::Stopped => (icon::play().size(10), text("START SERVER"), false),
            ServerState::Running => (icon::stop().size(10), text("STOP SERVER"), true),
        };

        let toggle_button = ButtonLabel::IconWithText {
            icon: toggle_icon,
            text: toggle_label,
        }
        .into_button()
        .width(160.0)
        .on_press(Message::ToggleServer)
        .class(ButtonClass::Primary);

        let status: Option<Element<'_, Message>> = if is_running {
            let local_ip = self
                .config
                .selected_ip
                .as_ref()
                .or_else(|| self.local_ips.as_ref().and_then(|ips| ips.first()));

            let port = self.server_addr.map(|addr| addr.port());

            let url = local_ip
                .zip(port)
                .map(|(ip, port)| format!("http://{ip}:{}", port));

            let qr_btn = ButtonLabel::Icon(icon::qrcode())
                .into_button()
                .height(28)
                .padding([0, 9])
                .class(ButtonClass::Transparent(
                    theme::button::BackgroundColor::Blue,
                    theme::button::TextColor::Blue,
                ))
                .on_press_maybe(url.as_ref().map(|url| Message::OpenQrModal(url.clone())));

            let url_btn: Element<'_, Message> = match (port, local_ip, &self.local_ips) {
                (Some(port), Some(local_ip), Some(local_ips)) => ButtonLabel::IconWithText {
                    icon: icon::circle().size(8),
                    text: text(url.unwrap_or_else(|| "?".to_string())).size(12),
                }
                .into_button()
                .height(28)
                .padding([0, 12])
                .on_press(Message::open_settings(Data {
                    port: Some(port),
                    ip: *local_ip,
                    ips: local_ips.clone(),
                    port_error: Default::default(),
                }))
                .class(ButtonClass::Transparent(
                    theme::button::BackgroundColor::Blue,
                    theme::button::TextColor::Green,
                ))
                .into(),
                (_, _, _) => void().into(),
            };

            Some(
                row![url_btn, qr_btn]
                    .spacing(4)
                    .align_y(Alignment::Center)
                    .into(),
            )
        } else {
            None
        };

        let error = self.error.as_ref().map(|error| {
            container(
                row![
                    text(format!("Error: {error}")).size(12).width(Length::Fill),
                    ButtonLabel::Icon(icon::cancel().size(10))
                        .into_button()
                        .padding([0, 7])
                        .height(20)
                        .on_press(Message::CloseError)
                        .class(ButtonClass::Transparent(
                            theme::button::BackgroundColor::White,
                            theme::button::TextColor::White
                        ))
                ]
                .spacing(10)
                .align_y(Alignment::Center),
            )
            .width(Length::Fill)
            .padding(padding::left(15).right(10).top(5).bottom(5))
            .style(theme::container::red)
        });

        let footer = container(
            row![container(toggle_button).width(Length::Fill), status].align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .padding(15);

        let qr_modal: Option<Element<'_, Message>> = self
            .qr_modal
            .as_ref()
            .map(|qr_data| dialog(qr_code(qr_data).cell_size(6).into(), Message::CloseQrModal));

        let settings = self.settings.view().map(|s| s.map(Message::Settings));

        let screen = column![
            header,
            folders_scrollable,
            error,
            rule::horizontal(1),
            footer
        ];

        stack![screen].push(qr_modal).push(settings).into()
    }

    pub fn theme(&self) -> Option<Theme> {
        Theme.into()
    }

    pub fn window_settings() -> iced::window::Settings {
        let size = iced::Size::new(500.0, 768.0);

        iced::window::Settings {
            size,
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

async fn pick_folders(existing_folders: Vec<PathBuf>) -> Option<Vec<PathBuf>> {
    let mut dialog = rfd::AsyncFileDialog::new();

    if let Some(start_dir) = existing_folders.into_iter().rev().find(|f| f.exists()) {
        dialog = dialog.set_directory(start_dir);
    } else if let Some(home) = dirs::home_dir() {
        dialog = dialog.set_directory(home);
    }

    dialog.pick_folders().await.map(|handles| {
        handles
            .into_iter()
            .map(|handle| handle.path().to_path_buf())
            .collect()
    })
}
