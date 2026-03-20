use std::sync::Arc;

use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router,
};

use crate::types::{DownloadRequest, Md5, SearchOptions};
use crate::Client;

// ── Parameter types ──────────────────────────────────────────────

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SearchParams {
    /// Search query for books, papers, magazines, comics, etc.
    pub query: String,
    /// Page number (starts at 1)
    #[serde(default)]
    pub page: Option<u32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct DetailsParams {
    /// MD5 hash of the item to get details for
    pub md5: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct DownloadParams {
    /// MD5 hash of the item to download
    pub md5: String,
    /// Path index for download source selection
    #[serde(default)]
    pub path_index: Option<u32>,
    /// Domain index for download source selection
    #[serde(default)]
    pub domain_index: Option<u32>,
}

// ── Server struct ────────────────────────────────────────────────

#[derive(Clone)]
pub struct Server {
    client: Arc<Client>,
    tool_router: ToolRouter<Self>,
}

impl Server {
    pub fn new(client: Arc<Client>) -> Self {
        Self {
            client,
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for Server {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Anna's Archive — search and retrieve books, papers, \
                 magazines, comics, and other documents from the world's \
                 largest open library index. Use search without an API key. \
                 Details and downloads require an API key."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

#[tool_router]
impl Server {
    #[tool(
        description = "Search Anna's Archive for books, papers, magazines, comics, and other documents. No API key required."
    )]
    async fn search(
        &self,
        Parameters(params): Parameters<SearchParams>,
    ) -> String {
        let options = SearchOptions::new(&params.query);
        let options = if let Some(page) = params.page {
            options.with_page(page)
        } else {
            options
        };

        match self.client.search(options).await {
            Ok(response) => {
                serde_json::to_string_pretty(&response).unwrap_or_else(
                    |e| format!("{{\"error\": \"serialization: {e}\"}}"),
                )
            }
            Err(e) => format!("{{\"error\": \"{e}\"}}"),
        }
    }

    #[tool(
        description = "Get detailed metadata for an item by its MD5 hash. Requires API key."
    )]
    async fn details(
        &self,
        Parameters(params): Parameters<DetailsParams>,
    ) -> String {
        let md5 = Md5::from(params.md5.as_str());
        match self.client.details(&md5).await {
            Ok(details) => {
                serde_json::to_string_pretty(&details).unwrap_or_else(
                    |e| format!("{{\"error\": \"serialization: {e}\"}}"),
                )
            }
            Err(e) => format!("{{\"error\": \"{e}\"}}"),
        }
    }

    #[tool(
        description = "Get a fast download URL for an item. Requires API key."
    )]
    async fn download_url(
        &self,
        Parameters(params): Parameters<DownloadParams>,
    ) -> String {
        let request = DownloadRequest {
            md5: Md5::from(params.md5.as_str()),
            path_index: params.path_index,
            domain_index: params.domain_index,
        };

        match self.client.download_url(request).await {
            Ok(info) => {
                serde_json::to_string_pretty(&info).unwrap_or_else(|e| {
                    format!("{{\"error\": \"serialization: {e}\"}}")
                })
            }
            Err(e) => format!("{{\"error\": \"{e}\"}}"),
        }
    }
}
