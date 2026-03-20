use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use reqwest::cookie::Jar;

use crate::error::Error;
use crate::types::{
    Config, DownloadInfo, DownloadRequest, ItemDetails, Md5, SearchOptions,
    SearchResponse,
};

/// Client for the Anna's Archive API.
///
/// Provides search, detail lookup, and download URL resolution with
/// automatic domain failover across configured mirrors.
pub struct Client {
    http: reqwest::Client,
    config: Config,
    // Held to keep the cookie jar alive for reqwest's cookie_provider.
    _cookie_jar: Arc<Jar>,
    authenticated: AtomicBool,
}

impl Default for Client {
    fn default() -> Self {
        Self::from_config(Config::default())
    }
}

impl From<Config> for Client {
    fn from(config: Config) -> Self {
        Self::from_config(config)
    }
}

impl Client {
    /// Create a client with default configuration (no API key).
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a client with an API key and otherwise default config.
    pub fn with_api_key(api_key: impl Into<String>) -> Self {
        Self::from_config(Config {
            api_key: Some(api_key.into()),
            ..Config::default()
        })
    }

    /// Create a client from a full configuration.
    pub fn from_config(config: Config) -> Self {
        let cookie_jar = Arc::new(Jar::default());

        let http = reqwest::Client::builder()
            .user_agent(&config.user_agent)
            .cookie_provider(cookie_jar.clone())
            .build()
            .expect("failed to create HTTP client");

        Self {
            http,
            config,
            _cookie_jar: cookie_jar,
            authenticated: AtomicBool::new(false),
        }
    }

    /// Search Anna's Archive. Does not require an API key.
    pub async fn search(
        &self,
        options: SearchOptions,
    ) -> Result<SearchResponse, Error> {
        let page = options.page.unwrap_or(1);
        let query = urlencoding::encode(&options.query);
        let path = format!("/search?q={query}&page={page}");

        let html = self.fetch_with_failover(&path).await?;
        SearchResponse::from_html(&html, page)
    }

    /// Get detailed metadata for an item by MD5 hash.
    /// Requires an API key.
    pub async fn details(
        &self,
        md5: &Md5,
    ) -> Result<ItemDetails, Error> {
        self.ensure_authenticated().await?;

        let path =
            format!("/db/aarecord_elasticsearch/md5:{md5}.json");

        let mut last_error = None;

        for domain in &self.config.domains {
            let url = format!("https://{domain}{path}");

            let response = match self.http.get(&url).send().await {
                Ok(r) => r,
                Err(e) => {
                    last_error = Some(Error::Network(e));
                    continue;
                }
            };

            if response.status().is_success() {
                let body =
                    response.text().await.map_err(Error::Network)?;
                return ItemDetails::from_json(&body, md5);
            }

            let status = response.status().as_u16();

            if status == 403 {
                // Re-authenticate and retry once.
                self.authenticated.store(false, Ordering::SeqCst);
                self.authenticate().await?;

                if let Ok(resp) = self.http.get(&url).send().await
                    && resp.status().is_success()
                {
                    let body = resp
                        .text()
                        .await
                        .map_err(Error::Network)?;
                    return ItemDetails::from_json(&body, md5);
                }
            }

            if response.status().is_client_error() {
                return Err(Error::Http { status });
            }

            last_error = Some(Error::Http { status });
        }

        Err(last_error.unwrap_or(Error::DomainsExhausted))
    }

    /// Get a fast download URL for an item. Requires an API key.
    pub async fn download_url(
        &self,
        request: DownloadRequest,
    ) -> Result<DownloadInfo, Error> {
        let api_key = self
            .config
            .api_key
            .as_ref()
            .ok_or(Error::KeyRequired)?;

        let path_idx = request.path_index.unwrap_or(0);
        let domain_idx = request.domain_index.unwrap_or(0);
        let md5 = &request.md5;

        let mut last_error = None;

        for domain in &self.config.domains {
            let url = format!(
                "https://{domain}/dyn/api/fast_download.json\
                 ?md5={md5}\
                 &path_index={path_idx}\
                 &domain_index={domain_idx}\
                 &key={api_key}"
            );

            let response = match self.http.get(&url).send().await {
                Ok(r) => r,
                Err(e) => {
                    last_error = Some(Error::Network(e));
                    continue;
                }
            };

            if !response.status().is_success() {
                let status = response.status().as_u16();
                let body = response.text().await.unwrap_or_default();

                if body.contains("no_membership")
                    || body.contains("invalid")
                {
                    return Err(Error::KeyRejected);
                }

                last_error = Some(Error::Http { status });
                continue;
            }

            #[derive(serde::Deserialize)]
            struct ApiResponse {
                download_url: Option<String>,
                error: Option<String>,
            }

            let api_resp: ApiResponse =
                response.json().await.map_err(Error::Network)?;

            if let Some(error) = api_resp.error {
                return Err(Error::Remote { message: error });
            }

            let download_url =
                api_resp.download_url.ok_or(Error::MissingField {
                    field: "download_url",
                })?;

            return Ok(DownloadInfo { download_url });
        }

        Err(last_error.unwrap_or(Error::DomainsExhausted))
    }

    // ── Internal ─────────────────────────────────────────────────

    async fn authenticate(&self) -> Result<(), Error> {
        let api_key = self
            .config
            .api_key
            .as_ref()
            .ok_or(Error::KeyRequired)?;

        for domain in &self.config.domains {
            let url = format!("https://{domain}/account/");

            let response = self
                .http
                .post(&url)
                .form(&[("key", api_key.as_str())])
                .send()
                .await;

            match response {
                Ok(resp)
                    if resp.status().is_success()
                        || resp.status().is_redirection() =>
                {
                    self.authenticated.store(true, Ordering::SeqCst);
                    return Ok(());
                }
                Ok(resp) if resp.status().is_client_error() => {
                    return Err(Error::KeyRejected);
                }
                _ => continue,
            }
        }

        Err(Error::DomainsExhausted)
    }

    async fn ensure_authenticated(&self) -> Result<(), Error> {
        if !self.authenticated.load(Ordering::SeqCst) {
            self.authenticate().await?;
        }
        Ok(())
    }

    async fn fetch_with_failover(
        &self,
        path: &str,
    ) -> Result<String, Error> {
        let mut last_error = None;

        for domain in &self.config.domains {
            let url = format!("https://{domain}{path}");

            match self.http.get(&url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        return response
                            .text()
                            .await
                            .map_err(Error::Network);
                    }

                    let status = response.status().as_u16();

                    if response.status().is_client_error() {
                        return Err(Error::Http { status });
                    }

                    // Server error — try next domain.
                    last_error = Some(Error::Http { status });
                }
                Err(e) => {
                    last_error = Some(Error::Network(e));
                }
            }
        }

        Err(last_error.unwrap_or(Error::DomainsExhausted))
    }
}
