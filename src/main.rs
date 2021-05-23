#![windows_subsystem = "windows"]
#![feature(with_options)]

use iced::{Align, Application, Button, Clipboard, Column, Command, Container, Element, Error, HorizontalAlignment, Length, Settings, Text, button, window};
use std::path::Path;
use std::fs;
use std::fs::File;
use std::io::{
    ErrorKind, BufWriter, prelude::*
};
use std::env;
use std::process;

#[cfg(target_family = "unix")]
static HOST_FILE: &str = "/etc/hosts";
#[cfg(target_family = "windows")]
static HOST_FILE: &str = "C:\\Windows\\System32\\drivers\\etc\\hosts";

pub fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 && args[1] == "clean" {
        let result = clean();
        if result.is_err() {
            eprintln!("error: {:?}", result.err());
        }
        return Ok(());
    } else {
        return MojangAuth::run(Settings {
            window: window::Settings {
                size: (350, 250),
                resizable: true,
                decorations: true,
                always_on_top: false,
                transparent: false,
                icon: None,
                min_size: None,
                max_size: None
            },
            ..Settings::default()
        });
    }
}

#[derive(Debug)]
enum MojangAuth {
    Loading,
    Loaded(State),
}

#[derive(Debug, Default)]
struct State {
    clean_button: button::State,
    modified: Option<Vec<String>>,
    text: String,
}

#[derive(Debug, Clone)]
enum Message {
    Loaded(Result<HostFile, LoadError>),
    CleanPressed,
}

impl Application for MojangAuth {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (MojangAuth, Command<Message>) {
        (
            MojangAuth::Loading,
            Command::perform(HostFile::check(), Message::Loaded),
        )
    }

    fn title(&self) -> String {
        String::from("Mojang Auth Checker")
    }

    fn update(&mut self, message: Message, _clipboard: &mut Clipboard) -> Command<Message> {
        match self {
            MojangAuth::Loading => {
                match message {
                    Message::Loaded(Ok(state)) => {
                        if state.modified.len() > 0 {
                            *self = MojangAuth::Loaded(State {
                                clean_button: button::State::new(),
                                modified: Some(state.modified),
                                text: String::from("Le fichier hosts a été modifié ! Modifications trouvés :"),
                            });
                        }
                        else {
                            *self = MojangAuth::Loaded(State {
                                clean_button: button::State::new(),
                                modified: None,
                                text: String::from("Tout semble bon !"),
                            });
                        }
                    }
                    Message::Loaded(Err(_)) => {
                        *self = MojangAuth::Loaded(State {
                            clean_button: button::State::new(),
                            modified: None,
                            text: String::from("Impossible de lire le fichier hosts!"),
                        });
                    }
                    _ => {}
                }

                Command::none()
            }
            MojangAuth::Loaded(_) => {
                match message {
                    Message::CleanPressed => {
                        let result = match clean() {
                            Ok(_) => String::from("OK ! Lancez Minecraft pour vérifier"),
                            Err(err) => match err.kind() {
                                ErrorKind::PermissionDenied => self_run_admin(),
                                _ => String::from(format!("Échec du nettoyage: {}", err)),
                            },
                        };
                        *self = MojangAuth::Loaded(State {
                            clean_button: button::State::new(),
                            modified: None,
                            text: result,
                        });
                    }
                    _ => {}
                };
                Command::none()
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        match self {
            MojangAuth::Loading => loading_message(),
            MojangAuth::Loaded(State {
                clean_button,
                modified,
                text
            }) => {
                let title = Text::new("Mojang auth checker")
                    .width(Length::Fill)
                    .size(30)
                    .color([0.5, 0.5, 0.5])
                    .horizontal_alignment(HorizontalAlignment::Center);

                let button = match modified {
                    Some(_line) =>  Button::new(clean_button, Text::new("Nettoyer")).on_press(Message::CleanPressed),
                    None =>  Button::new(clean_button, Text::new("Nettoyer"))
                };

                let modified_lines: Element<_> = match modified {
                    None => Column::new().into(),
                    Some(lines) => {
                        lines.iter_mut()
                        .fold(Column::new().spacing(0), |column, line| {
                            column.push(Text::new(line.as_str()))
                        })
                        .into()
                    }
                };

                let content = Column::new()
                    .padding(10)
                    .spacing(20)
                    .align_items(Align::Center)
                    .push(title)
                    .push(Text::new(text.as_str())
                        .width(Length::Fill)
                        .size(20)
                        .horizontal_alignment(HorizontalAlignment::Center))
                    .push(modified_lines)
                    .push(button);
                
                Container::new(content)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x()
                    .into()
            }
        }
    }
}

fn loading_message() -> Element<'static, Message> {
    Container::new(
        Text::new("Loading...")
            .horizontal_alignment(HorizontalAlignment::Center)
            .size(50),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_y()
    .into()
}

// Persistence
#[derive(Debug, Clone)]
struct HostFile {
    modified: Vec<String>,
}

#[derive(Debug, Clone)]
enum LoadError {
    FileError,
}

impl HostFile {
    async fn check() -> Result<HostFile, LoadError> {
        use async_std::prelude::*;

        let mut contents = String::new();

        let mut file = async_std::fs::File::open(HOST_FILE)
            .await
            .map_err(|_| LoadError::FileError)?;

        file.read_to_string(&mut contents)
            .await
            .map_err(|_| LoadError::FileError)?;

        Ok(HostFile {
            modified: contents.lines().filter(|line| line.contains("mojang.com")).map(|line| String::from(line)).collect()
        })
    }
}

fn clean() -> Result<(), std::io::Error> {
    let path = Path::new(HOST_FILE);
    let contents = fs::read_to_string(&path)?;

    let file = File::with_options().write(true).truncate(true).open(&path)?;
    let mut writer = BufWriter::new(&file);
    for line in contents.lines().filter(|line| !line.contains("mojang.com")) {
        writer.write(line.as_bytes())?;
        writer.write(b"\r\n")?;
    }
    writer.flush()?;
    Ok(())
}

fn self_run_admin() -> String {
    let args: Vec<String> = env::args().collect();
    let command = format!("'{}' clean", args[0]);
    match process::Command::new("powershell")
    .args(&["start", "-verb", "runas", command.as_str()])
    .status() {
        Ok(status) => if status.success() {
                String::from("OK ! Lancez Minecraft pour vérifier")
            } else {
                String::from("Échec du nettoyage")
            },
        Err(_) => String::from("Échec du nettoyage")
    }
}
