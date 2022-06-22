use std::sync::Arc;

use anyhow::Result;
use reqwest::{
    header::{self, HeaderMap},
    Client as HttpClient,
};
use serde::Deserialize;
use time::{Duration, OffsetDateTime};
use tokio::sync::Mutex;
use tracing::info;
use url::Url;

use crate::settings;

mod deser;

#[derive(Debug, Deserialize)]
pub struct SearchResponse {
    pub game_id: String,
    pub display_name: String,
    pub broadcaster_language: String,
    pub title: String,
    pub thumbnail_url: String,
    #[serde(deserialize_with = "deser::opt_datetime")]
    pub started_at: Option<OffsetDateTime>,
}

#[derive(Debug, Deserialize)]
pub struct Stream {
    pub id: String,
    pub user_id: String,
    pub user_name: String,
    pub game_id: String,
    #[serde(rename = "type")]
    pub ty: Option<String>,
    pub title: String,
    pub viewer_count: u64,
    #[serde(deserialize_with = "deser::opt_datetime")]
    pub started_at: Option<OffsetDateTime>,
    pub language: String,
    pub thumbnail_url: String,
}

#[derive(Deserialize)]
pub struct AuthResponse {
    access_token: String,
    #[serde(deserialize_with = "deser::duration")]
    expires_in: Duration,
}

#[derive(Deserialize)]
pub struct Response<T> {
    pub data: Vec<T>,
    pub pagination: Option<Pagination>,
}

#[derive(Deserialize)]
pub struct Pagination {
    pub cursor: Option<String>,
}

#[derive(Copy, Clone, Deserialize)]
pub enum Category {
    Art,
    BeautyAndBodyArt,
    FoodAndDrink,
    JustChatting,
    MakersAndCrafting,
    Music,
    Retro,
    ScienceAndTechnology,
    SoftwareAndGameDevelopment,
    TalkShowsAndPodcasts,
}

impl Category {
    pub const fn game_id(self) -> &'static str {
        match self {
            Self::Art => "509660",
            Self::BeautyAndBodyArt => "509669",
            Self::FoodAndDrink => "509667",
            Self::JustChatting => "509658",
            Self::MakersAndCrafting => "509673",
            Self::Music => "26936",
            Self::Retro => "27284",
            Self::ScienceAndTechnology => "509670",
            Self::SoftwareAndGameDevelopment => "1469308723",
            Self::TalkShowsAndPodcasts => "417752",
        }
    }

    #[allow(dead_code)]
    const fn name(self) -> &'static str {
        match self {
            Self::Art => "Art",
            Self::BeautyAndBodyArt => "Beauty & Body Art",
            Self::FoodAndDrink => "Food & Drink",
            Self::JustChatting => "Just Chatting",
            Self::MakersAndCrafting => "Makers & Crafting",
            Self::Music => "Music",
            Self::Retro => "Retro",
            Self::ScienceAndTechnology => "Science & Technology",
            Self::SoftwareAndGameDevelopment => "Software and Game Development",
            Self::TalkShowsAndPodcasts => "Talk Shows & Podcasts",
        }
    }
}

pub type AsyncClient = Arc<Mutex<Client>>;

pub struct Client {
    client: HttpClient,
    client_id: String,
    client_secret: String,
    token: String,
    exires_at: OffsetDateTime,
}

impl Client {
    pub async fn get_token(client_id: &str, client_secret: &str) -> Result<AuthResponse> {
        let mut url = Url::parse("https://id.twitch.tv/oauth2/token").unwrap();
        url.query_pairs_mut()
            .append_pair("client_id", client_id)
            .append_pair("client_secret", client_secret)
            .append_pair("grant_type", "client_credentials")
            .append_pair("scope", "");

        let resp = HttpClient::new()
            .post(url)
            .send()
            .await?
            .error_for_status()?
            .json::<AuthResponse>()
            .await?;

        Ok(resp)
    }

    pub async fn new(
        settings::Twitch {
            client_id,
            client_secret,
        }: settings::Twitch,
    ) -> Result<Self> {
        info!("getting initial token");

        let resp = Self::get_token(&client_id, &client_secret).await?;
        let token = resp.access_token;

        let mut headers = HeaderMap::new();
        headers.insert(header::AUTHORIZATION, format!("Bearer {}", token).parse()?);
        headers.insert("Client-Id", client_id.parse()?);

        Ok(Self {
            client: HttpClient::builder().default_headers(headers).build()?,
            client_id,
            client_secret,
            token,
            exires_at: OffsetDateTime::now_utc() + resp.expires_in - Duration::days(1),
        })
    }

    async fn get_streams(&self, game_id: &str, after: Option<&str>) -> Result<Response<Stream>> {
        let mut url = Url::parse("https://api.twitch.tv/helix/streams").unwrap();
        url.query_pairs_mut()
            .append_pair("game_id", game_id)
            .append_pair("first", "100");

        if let Some(after) = after {
            url.query_pairs_mut().append_pair("after", after);
        }

        self.client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
            .map_err(Into::into)
    }

    pub async fn get_streams_all(&mut self, game_id: &str) -> Result<Vec<Stream>> {
        if self.exires_at <= OffsetDateTime::now_utc() {
            info!("refreshing token");
            self.token = Self::get_token(&self.client_id, &self.client_secret)
                .await?
                .access_token;
        }

        let Response {
            mut data,
            mut pagination,
        } = self.get_streams(game_id, None).await?;

        while let Some(cursor) = pagination.and_then(|p| p.cursor) {
            let resp = self.get_streams(game_id, Some(&cursor)).await?;

            data.extend(resp.data);
            pagination = resp.pagination;
        }

        Ok(data)
    }
}
