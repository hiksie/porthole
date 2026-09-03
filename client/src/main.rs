mod util;

use iced::widget::{button, column, row, scrollable, text};
use iced::{Alignment, Element, Length, Task};
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use tokio::sync::oneshot;

const SERVER_PORT: u16 = 8080;

fn main() -> iced::Result {
    iced::application(State::default, State::update, State::view)
        .title("Porthole")
        .settings(iced::Settings {
            id: Some(common::APP_ID.into()),
            ..iced::Settings::default()
        })
        .run()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ServerState {
    Stopped,
    Running,
}

struct State {
    config: common::Config,
    folders: server::FolderHandle,
    app_state: server::AppState,
    server_state: ServerState,
    shutdown_tx: Option<oneshot::Sender<()>>,
    local_ip: Option<IpAddr>,
    error: Option<String>,
}

impl Default for State {
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
enum Message {
    AddFolder,
    FolderPicked(Option<PathBuf>),
    RemoveFolder(PathBuf),
    ToggleServer,
    ServerStopped(Result<(), String>),
}

impl State {
    fn update(&mut self, message: Message) -> Task<Message> {
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

    fn view(&self) -> Element<'_, Message> {
        let add_button = button("Add folder").on_press(Message::AddFolder);

        let folders_section: Element<'_, Message> = if self.config.folders.is_empty() {
            text("No folders selected").into()
        } else {
            column(self.config.folders.iter().map(|folder| {
                row![
                    text(folder.display().to_string()).width(Length::Fill),
                    button("Remove").on_press(Message::RemoveFolder(folder.clone())),
                ]
                .spacing(8)
                .align_y(Alignment::Center)
                .into()
            }))
            .spacing(6)
            .into()
        };

        let (toggle_label, is_running) = match self.server_state {
            ServerState::Stopped => ("Start server", false),
            ServerState::Running => ("Stop server", true),
        };
        let toggle_button = button(toggle_label).on_press(Message::ToggleServer);

        let status: Element<'_, Message> = if is_running {
            match self.local_ip {
                Some(ip) => text(format!("Server running: http://{ip}:{SERVER_PORT}")).into(),
                None => text(format!(
                    "Server running on port {SERVER_PORT} (could not determine local IP)"
                ))
                .into(),
            }
        } else {
            text("Server stopped").into()
        };

        let mut content = column![add_button, folders_section, toggle_button, status]
            .spacing(16)
            .padding(20)
            .width(Length::Fill);

        if let Some(err) = &self.error {
            content = content.push(text(format!("Error: {err}")));
        }

        scrollable(content).into()
    }
}

async fn pick_folder() -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .pick_folder()
        .await
        .map(|handle| handle.path().to_path_buf())
}
