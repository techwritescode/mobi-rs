use iced::alignment::Horizontal;
use iced::futures::StreamExt;
use iced::widget::image::Handle;
use iced::widget::{
    button, column, container, horizontal_space, row, scrollable, text, text_input, Column,
};
use iced::{widget, Background, Color, Element, Length, Padding, Subscription, Task, Theme};
use reqwest::Client;
use std::sync::Arc;

const IMAGE_WIDTH: f32 = 128.0;
const IMAGE_HEIGHT: f32 = 128.0;

fn main() -> iced::Result {
    iced::application::application("Manga2Kindle", App::update, App::view)
        .subscription(App::subscription)
        .theme(|_| Theme::Dark)
        .run_with(App::new)
}

#[derive(Debug, Clone)]
enum Message {
    Search(String),
    Submit,
    SearchResults(Result<Vec<MangaListItem>, Error>),
    ThumbnailLoaded {
        id: u64,
        index: usize,
        handle: Result<Handle, Error>,
    },
    MangaSelected(MangaListItem),
    Back,
    MangaCoverLoaded(Result<Handle, Error>),
}

#[derive(Debug, Clone)]
enum Error {
    RequestFailed(Arc<reqwest::Error>),
    IOFailed(Arc<std::io::Error>),
    JoinFailed(Arc<tokio::task::JoinError>),
    ImageFailed(Arc<image::ImageError>),
}

#[derive(Debug, Clone)]
struct MangaListItem {
    pub id: String,
    pub title: String,
    pub description: String,
    pub cover_url: String,
    pub cover_image: Option<Handle>,
}

#[derive(Debug, serde::Deserialize)]
pub struct Manga {
    pub id: String,
    pub attributes: serde_json::Value,
    pub relationships: Vec<serde_json::Value>,
}

#[derive(Debug, serde::Deserialize)]
pub struct CollectionResponse<T> {
    result: String,
    response: String,
    data: Vec<T>,
}

pub enum State {
    Search,
    Detail,
}

struct App {
    search: String,
    search_id: u64,
    results: Vec<MangaListItem>,
    state: State,
    selected_manga: Option<MangaListItem>,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                search: String::new(),
                search_id: 0,
                results: Vec::new(),
                state: State::Search,
                selected_manga: None,
            },
            widget::focus_next(),
        )
    }

    fn subscription(&self) -> Subscription<Message> {
        match self.state {
            State::Search => {
                if self.results.is_empty() {
                    Subscription::none()
                } else {
                    let urls = self
                        .results
                        .iter()
                        .map(|result| result.cover_url.clone())
                        .collect::<Vec<_>>();
                    Self::thumbnail_subscription(self.search_id, urls)
                }
            }
            _ => Subscription::none(),
        }
    }

    fn thumbnail_subscription(id: u64, urls: Vec<String>) -> Subscription<Message> {
        use iced::futures::stream;

        let stream = stream::iter(urls.into_iter().enumerate())
            .map(move |(index, url)| {
                let id = id;
                async move {
                    let handle = fetch_image(format!("{url}.256.jpg")).await;
                    Message::ThumbnailLoaded { id, index, handle }
                }
            })
            .buffer_unordered(4);

        Subscription::run_with_id(id, stream)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Search(search) => {
                self.search = search;
                Task::none()
            }
            Message::Submit => {
                self.search_id += 1;
                Task::perform(do_search(self.search.clone()), Message::SearchResults)
            }
            Message::MangaSelected(manga) => {
                let cover_url = manga.cover_url.clone();
                self.selected_manga = Some(manga);
                self.state = State::Detail;
                Task::perform(fetch_image(cover_url), Message::MangaCoverLoaded)
            }
            Message::MangaCoverLoaded(cover) => {
                if let Ok(cover) = cover {
                    self.selected_manga.as_mut().unwrap().cover_image = Some(cover);
                }
                Task::none()
            }
            Message::Back => {
                self.state = State::Search;
                Task::none()
            }
            Message::SearchResults(results) => {
                if let Ok(results) = results {
                    self.results = results;
                } else {
                    eprintln!("{results:#?}");
                }
                Task::none()
            }
            Message::ThumbnailLoaded { id, index, handle } if id == self.search_id => {
                if let (Ok(h), Some(item)) = (handle, self.results.get_mut(index)) {
                    item.cover_image = Some(h);
                }
                Task::none()
            }
            _ => Task::none(),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        match self.state {
            State::Search => self.view_search(),
            State::Detail => self.view_detail(),
        }
    }

    fn view_search(&self) -> Element<'_, Message> {
        let rows: Vec<Element<'_, Message>> = self
            .results
            .iter()
            .map(|list_item| {
                let thumb: Element<'_, Message> = if let Some(handle) = &list_item.cover_image {
                    widget::image(handle.clone())
                        .width(IMAGE_WIDTH)
                        .height(IMAGE_HEIGHT)
                        .into()
                } else {
                    container(text("Loading"))
                        .style(|_| container::Style {
                            background: Some(Background::Color(Color::from_rgb(0.2, 0.2, 0.2))),
                            ..Default::default()
                        })
                        .width(IMAGE_WIDTH)
                        .height(IMAGE_HEIGHT)
                        .into()
                };
                button(row![thumb, column![text(&list_item.title)]])
                    .width(Length::Fill)
                    .on_press(Message::MangaSelected(MangaListItem {
                        id: list_item.id.clone(),
                        title: list_item.title.clone(),
                        cover_url: list_item.cover_url.clone(),
                        description: list_item.description.clone(),
                        cover_image: None, // Leave blank so we can load the full one later
                    }))
                    .into()
            })
            .collect();

        container(column![
            row![
                text_input("Search", &self.search)
                    .on_input(Message::Search)
                    .on_submit(Message::Submit),
                button("Go").on_press(Message::Submit)
            ],
            scrollable(Column::with_children(rows).padding(4))
                .height(Length::Fill)
                .width(Length::Fill)
        ])
        .padding(4)
        .into()
    }

    fn view_detail(&self) -> Element<'_, Message> {
        let selected_manga = self
            .selected_manga
            .as_ref()
            .expect("Selected manga cannot be null in the detail view");
        let cover: Element<'_, Message> = selected_manga
            .cover_image
            .as_ref()
            .and_then(|cover_image| Some(widget::image(cover_image).width(256).into()))
            .unwrap_or(container("").width(256).into());

        let content = scrollable(column![row![
            column![
                text(&selected_manga.title).size(32),
                container(cover)
                    .padding(Padding::ZERO.top(8))
                    .align_x(Horizontal::Center),
            ]
            .padding(20),
            column![text(&selected_manga.description)].padding(Padding::new(20.0).left(0))
        ]]);

        container(column![
            row![
                button("Back").on_press(Message::Back),
                horizontal_space(),
                button("Settings")
            ]
            .padding(8),
            content,
        ])
        .padding(4)
        .into()
    }
}

async fn do_search(search: String) -> Result<Vec<MangaListItem>, Error> {
    let url = format!(
        "https://api.mangadex.org/manga?title={}&includes[]=cover_art",
        search,
    );
    eprintln!("Fetching {}", url);
    let response = get_client()
        .get(url)
        .send()
        .await?
        .json::<CollectionResponse<Manga>>()
        .await?;

    let results = response
        .data
        .iter()
        .map(|item| {
            let cover_image = item
                .relationships
                .iter()
                .find_map(|relationship| get_cover_from_relationship(relationship))
                .unwrap();

            MangaListItem {
                id: item.id.clone(),
                title: get_title_from_attributes(&item.attributes)
                    .unwrap_or("Unknown Title".to_owned()),
                description: get_description_from_attributes(&item.attributes)
                    .unwrap_or("No Description".to_owned()),
                cover_url: format!(
                    "https://uploads.mangadex.org/covers/{}/{}",
                    item.id, cover_image
                ),
                cover_image: None,
            }
        })
        .collect();

    Ok(results)
}

async fn load_manga(id: String) -> Result<Vec<String>, Error> {
    // Load Chapters

    Ok(vec![])
}

fn get_cover_from_relationship(relationship: &serde_json::Value) -> Option<String> {
    if relationship["type"].as_str()? == "cover_art" {
        Some(relationship["attributes"]["fileName"].as_str()?.to_string())
    } else {
        None
    }
}

fn get_title_from_attributes(attributes: &serde_json::Value) -> Option<String> {
    if let Some(title) = attributes["title"]["en"].as_str() {
        Some(title.to_string())
    } else {
        attributes["altTitles"].as_array()?.iter().find_map(|alt| {
            if alt.as_object()?.contains_key("en") {
                Some(alt["en"].as_str()?.to_string())
            } else {
                None
            }
        })
    }
}

fn get_description_from_attributes(attributes: &serde_json::Value) -> Option<String> {
    if let Some(description) = attributes["description"]["en"].as_str() {
        Some(description.to_string())
    } else {
        None
    }
}

fn get_client() -> Client {
    reqwest::ClientBuilder::new()
        .user_agent("Manga2Kindle")
        .build()
        .unwrap()
}

async fn fetch_image(url: String) -> Result<Handle, Error> {
    let bytes = reqwest::ClientBuilder::new()
        .user_agent("manga2kindle")
        .build()
        .unwrap()
        .get(url)
        .send()
        .await?
        .bytes()
        .await?;

    Ok(Handle::from_bytes(bytes))
}

impl From<tokio::task::JoinError> for Error {
    fn from(err: tokio::task::JoinError) -> Error {
        Error::JoinFailed(Arc::new(err))
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Error {
        Error::RequestFailed(Arc::new(err))
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::IOFailed(Arc::new(err))
    }
}

impl From<image::ImageError> for Error {
    fn from(err: image::ImageError) -> Error {
        Error::ImageFailed(Arc::new(err))
    }
}
