use iced::alignment::Horizontal;
use iced::futures::{StreamExt, stream};
use iced::widget::button::Status;
use iced::widget::image::Handle;
use iced::widget::{
    Column, button, center, column, container, horizontal_space, mouse_area, opaque, progress_bar,
    row, scrollable, stack, text, text_input,
};
use iced::{Background, Color, Element, Length, Padding, Subscription, Task, Theme, widget};
use iced_aw::card;
use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, ImageFormat};
use mobi::mobi_writer::MobiWriter;
use reqwest::Client;
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;

const IMAGE_WIDTH: f32 = 128.0;
const IMAGE_HEIGHT: f32 = 128.0;

fn main() -> iced::Result {
    // TODO: Someday
    // let disks = sysinfo::Disks::new_with_refreshed_list();
    //
    // for disk in disks.iter() {
    //     if disk.name() == "Kindle" && disk.is_removable() {
    //         eprintln!("Disk: {:?}", disk.mount_point());
    //     }
    // }

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
    MangaChaptersLoaded(Result<Vec<Volume>, Error>),
    ChapterClicked {
        volume: String,
        chapter: String,
        id: String,
    },
    MangaDownloaded(Result<(MangaChapterResponse, (String, String)), Error>),
    ImageDownloaded(Result<DownloadImage, Error>),
    DownloadError(Error),
}

#[derive(Debug, Clone)]
enum Error {
    RequestFailed(Arc<reqwest::Error>),
    IOFailed(Arc<std::io::Error>),
    JoinFailed(Arc<tokio::task::JoinError>),
    ImageFailed(Arc<image::ImageError>),
    GenericError(String),
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

#[derive(Debug, Clone)]
pub struct Chapter {
    pub id: String,
    pub volume: String,
    pub chapter: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Volume {
    pub name: String,
    pub chapters: Vec<Chapter>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct MangaChapterImages {
    hash: String,
    data: Vec<String>,
    // data_saver: Vec<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct MangaChapterResponse {
    #[serde(rename = "baseUrl")]
    base_url: String,
    chapter: MangaChapterImages,
}

pub enum State {
    Search,
    Detail,
}

struct Download {
    base_url: String,
    hash: String,

    volume: String,
    chapter: String,
    images: Vec<String>,

    downloaded: usize,
    total_to_download: usize,
    downloaded_images: HashMap<String, DownloadImage>,
}

#[derive(Debug, Clone)]
struct DownloadImage {
    name: String,
    width: u32,
    height: u32,
    bytes: Vec<u8>,
}

struct App {
    search: String,
    search_id: u64,
    results: Vec<MangaListItem>,
    state: State,
    selected_manga: Option<MangaListItem>,
    volumes: Vec<Volume>,

    active_download: Option<Download>,
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
                volumes: vec![],
                active_download: None,
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
            State::Detail => {
                if let Some(response) = &self.active_download {
                    download_manga_images(response)
                } else {
                    Subscription::none()
                }
            }
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
                let id = manga.id.clone();
                let cover_url = manga.cover_url.clone();
                self.selected_manga = Some(manga);
                self.state = State::Detail;

                Task::batch(vec![
                    Task::perform(load_manga(id), Message::MangaChaptersLoaded),
                    Task::perform(fetch_image(cover_url), Message::MangaCoverLoaded),
                ])
            }
            Message::MangaCoverLoaded(cover) => {
                if let Ok(cover) = cover {
                    self.selected_manga.as_mut().unwrap().cover_image = Some(cover);
                }
                Task::perform(
                    load_manga(self.selected_manga.as_ref().unwrap().id.clone()),
                    Message::MangaChaptersLoaded,
                )
            }
            Message::MangaChaptersLoaded(result) => {
                if let Ok(result) = result {
                    self.volumes = result;
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
                    eprintln!("Search Error: {results:#?}");
                }
                Task::none()
            }
            Message::ThumbnailLoaded { id, index, handle } if id == self.search_id => {
                if let (Ok(h), Some(item)) = (handle, self.results.get_mut(index)) {
                    item.cover_image = Some(h);
                }
                Task::none()
            }
            Message::ChapterClicked {
                id,
                volume,
                chapter,
            } => Task::perform(
                download_manga(id, volume, chapter),
                Message::MangaDownloaded,
            ),
            Message::MangaDownloaded(result) => {
                match result {
                    Ok((manga, (volume, chapter))) => {
                        let total_to_download = manga.chapter.data.len();
                        self.active_download = Some(Download {
                            base_url: manga.base_url,
                            hash: manga.chapter.hash,
                            volume,
                            chapter,
                            images: manga.chapter.data,
                            downloaded: 0,
                            total_to_download,
                            downloaded_images: HashMap::new(),
                        });
                    }
                    Err(err) => {
                        eprintln!("{err:?}")
                    }
                }

                Task::none()
            }
            Message::ImageDownloaded(result) => {
                let img = result.expect("Failed to download image");
                if let Some(download) = self.active_download.as_mut() {
                    download.downloaded_images.insert(img.name.to_owned(), img);
                    download.downloaded += 1;
                    if download.downloaded == download.total_to_download {
                        self.write_manga().expect("Failed to save manga");

                        self.active_download = None;
                    }
                } else {
                    unreachable!();
                }

                Task::none()
            }
            Message::DownloadError(error) => {
                eprintln!("{error:?}");
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
                    container(widget::image(handle.clone()))
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
                    .style(|a, b| button::Style {
                        background: match b {
                            Status::Hovered => {
                                Some(Background::Color(Theme::Dark.palette().primary))
                            }
                            _ => None,
                        },
                        text_color: Color::WHITE,
                        ..Default::default()
                    })
                    .width(Length::Fill)
                    .on_press(Message::MangaSelected(list_item.clone()))
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
            .and_then(|cover_image| Some(widget::image(cover_image).width(256).height(360).into()))
            .unwrap_or(
                container(center(text("Loading")))
                    .width(256)
                    .height(360)
                    .style(|_| container::Style {
                        background: Some(Color::from_rgb(0.2, 0.2, 0.2).into()),
                        ..Default::default()
                    })
                    .into(),
            );

        let volumes: Vec<Element<'_, Message>> = self
            .volumes
            .iter()
            .map(|volume| {
                let header: Element<'_, Message> =
                    container(text(format!("Volume {}", volume.name)))
                        .padding(4)
                        .width(Length::Fill)
                        .style(|_| container::Style {
                            background: Some(Background::Color(Theme::Dark.palette().success)),
                            ..Default::default()
                        })
                        .into();

                let chapters: Vec<Element<'_, Message>> = volume
                    .chapters
                    .iter()
                    .map(|chapter| {
                        button(text(format!("Chapter {}", chapter.name)))
                            .width(Length::Fill)
                            .on_press_with(|| Message::ChapterClicked {
                                id: chapter.id.clone(),
                                volume: chapter.volume.clone(),
                                chapter: chapter.chapter.clone(),
                            })
                            .style(|a, b| button::Style {
                                background: match b {
                                    Status::Hovered => {
                                        Some(Background::Color(Theme::Dark.palette().primary))
                                    }
                                    _ => None,
                                },
                                text_color: Color::WHITE,
                                ..Default::default()
                            })
                            .into()
                    })
                    .collect();

                let mut elements: Vec<Element<'_, Message>> = vec![header];
                elements.extend(chapters);

                Column::with_children(elements).into()
            })
            .collect();

        let content = scrollable(column![row![
            column![
                text(&selected_manga.title).size(32),
                container(cover)
                    .padding(Padding::ZERO.top(8))
                    .align_x(Horizontal::Center),
            ]
            .padding(20),
            column![
                text(&selected_manga.description),
                Column::with_children(volumes),
            ]
            .padding(Padding::new(20.0).left(0)),
        ]]);

        let modal: Element<'_, Message> = if let Some(download) = &self.active_download {
            container(opaque(mouse_area(
                center(opaque(
                    card(
                        text("Downloading"),
                        progress_bar(
                            0.0..=download.total_to_download as f32,
                            download.downloaded as f32,
                        ),
                    )
                    .width(300),
                ))
                .style(|_| container::Style {
                    background: Some(
                        Color {
                            a: 0.8,
                            ..Color::BLACK
                        }
                        .into(),
                    ),
                    ..Default::default()
                }),
            )))
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
        } else {
            column![].into()
        };

        stack![
            container(column![
                row![
                    button("Back").on_press(Message::Back),
                    horizontal_space(),
                    button("Settings")
                ]
                .padding(8),
                content,
            ])
            .padding(4),
            modal
        ]
        .into()
    }

    fn write_manga(&mut self) -> Result<(), Error> {
        if let Some(active_download) = self.active_download.as_mut() {
            let selected_manga = self.selected_manga.as_ref().unwrap();
            let title = selected_manga.title.clone();
            let cover_image = selected_manga
                .cover_image
                .clone()
                .ok_or(Error::GenericError("No cover image".to_owned()))?;

            let mut html = "<html><head></head><body>".to_owned();
            let mut writer = MobiWriter::new(title.clone());
            writer.add_image(make_cover(cover_image)?);
            for (i, k) in active_download.images.drain(..).enumerate() {
                let download_image = active_download
                    .downloaded_images
                    .remove(&k)
                    .expect("Failed to find image");
                writer.add_image(download_image.bytes);
                html += format!("<p height=\"0pt\" width=\"0pt\" align=\"center\"><img recindex=\"{:05}\" align=\"baseline\" width=\"{}\" height=\"{}\"></img></p><mbp:pagebreak/>", i+2, download_image.width, download_image.height).as_str();
            }

            html += "</body></html>";
            writer.set_content(html);
            std::fs::write(
                format!(
                    "{}.{}.{}.mobi",
                    title, active_download.volume, active_download.chapter
                ),
                writer.to_bytes()?,
            )?;

            Ok(())
        } else {
            unreachable!()
        }
    }
}

fn make_cover(image: Handle) -> Result<Vec<u8>, Error> {
    if let Handle::Bytes(_, img_bytes) = image {
        let mut bytes = Cursor::new(Vec::new());
        image::load_from_memory(&img_bytes)?
            .grayscale()
            .write_to(&mut bytes, ImageFormat::Jpeg)?;

        Ok(bytes.into_inner())
    } else {
        unreachable!()
    }
}

async fn do_search(search: String) -> Result<Vec<MangaListItem>, Error> {
    let url = format!(
        "https://api.mangadex.org/manga?title={}&includes[]=cover_art",
        search,
    );
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

async fn load_manga(id: String) -> Result<Vec<Volume>, Error> {
    // Load Chapters
    let url = format!(
        "https://api.mangadex.org/manga/{}/aggregate?translatedLanguage[]=en",
        id,
    );
    let response = get_client()
        .get(url)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    let volumes = response
        .get("volumes")
        .unwrap()
        .as_object()
        .ok_or(Error::GenericError("Failed to fetch volumes".to_owned()))?
        .iter()
        .rev()
        .map(|(vk, v)| Volume {
            name: vk.to_owned(),
            chapters: v
                .get("chapters")
                .unwrap()
                .as_object()
                .unwrap()
                .iter()
                .rev()
                .map(|(ck, v)| Chapter {
                    id: v.get("id").unwrap().as_str().unwrap().to_owned(),
                    volume: vk.to_owned(),
                    chapter: ck.to_owned(),
                    name: ck.to_owned(),
                })
                .collect::<Vec<_>>(),
        })
        .collect::<Vec<_>>();

    Ok(volumes)
}

async fn download_manga(
    id: String,
    volume: String,
    chapter: String,
) -> Result<(MangaChapterResponse, (String, String)), Error> {
    let client = get_client();

    let response = client
        .get(format!("https://api.mangadex.org/at-home/server/{id}"))
        .send()
        .await?
        .json::<MangaChapterResponse>()
        .await?;

    Ok((response, (volume, chapter)))
}

fn download_manga_images(download: &Download) -> Subscription<Message> {
    let client = get_client();

    let base_url = download.base_url.to_owned();
    let hash = download.hash.to_owned();
    let data = download.images.to_owned();

    let downloads = stream::iter(data.clone())
        .map(move |file| {
            let base_url = base_url.clone();
            let hash = hash.clone();
            let file = file.clone();
            let client = client.clone();

            async move {
                match client
                    .get(format!("{}/data/{}/{}", base_url, hash, file))
                    .send()
                    .await
                {
                    Ok(response) => {
                        let result: Result<DownloadImage, Error> = response
                            .bytes()
                            .await
                            .map_err(Into::into)
                            .and_then(|bytes| image::load_from_memory(&bytes).map_err(Into::into))
                            .and_then(|image| {
                                let mut bytes = Cursor::new(Vec::new());
                                let (width, height) = get_adjusted_size(&image);
                                match image
                                    .grayscale()
                                    .resize(width, height, FilterType::Lanczos3)
                                    .write_to(&mut bytes, ImageFormat::Jpeg)
                                {
                                    Ok(()) => Ok(DownloadImage {
                                        name: file,
                                        width,
                                        height,
                                        bytes: bytes.into_inner(),
                                    }),
                                    Err(e) => Err(e.into()),
                                }
                            });

                        Message::ImageDownloaded(result)
                        // std::fs::write(download_dir.join(file), response.bytes().await.unwrap())
                        //     .expect("Failed to write downloaded file");
                    }
                    Err(err) => Message::DownloadError(Error::from(err)),
                }
            }
        })
        .buffer_unordered(4);

    Subscription::run_with_id(download.hash.to_owned(), downloads)
}

fn get_adjusted_size(img: &DynamicImage) -> (u32, u32) {
    let (width, height) = img.dimensions();
    let max_width = 700;
    let max_height = 900;

    let scale_w = max_width as f32 / width as f32;
    let scale_h = max_height as f32 / height as f32;

    let scale = scale_w.min(scale_h);

    let new_width = (width as f32 * scale).round() as u32;
    let new_height = (height as f32 * scale).round() as u32;

    (new_width, new_height)
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
    let bytes = get_client().get(url).send().await?.bytes().await?;

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

impl From<anyhow::Error> for Error {
    fn from(value: anyhow::Error) -> Self {
        Error::GenericError(value.to_string())
    }
}
