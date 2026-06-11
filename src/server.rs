use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::sync::{Arc, Mutex};
use rmcp::{
    handler::server::{
        router::prompt::PromptRouter,
        wrapper::Parameters, 
        ServerHandler,
    },
    model::*,
    service::RequestContext,
    RoleServer,
    ErrorData as McpError,
    prompt,
};
use crate::{search, reader, research, models};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchWebParams {
    pub query: String,
    #[serde(default = "default_search_type")]
    pub search_type: String,
    pub max_results: Option<usize>,
}

fn default_search_type() -> String {
    "text".to_string()
}


#[derive(Debug, Deserialize, JsonSchema)]
pub struct FetchPageParams {
    pub url: String,
    #[serde(default = "default_output_format")]
    pub output_format: String,
    #[serde(default)]
    pub include_metadata: bool,
    #[serde(default = "default_max_length")]
    pub max_length: usize,
    #[serde(default = "default_timeout")]
    pub timeout: usize,
}

fn default_output_format() -> String {
    "txt".to_string()
}

fn default_max_length() -> usize {
    15000
}

fn default_timeout() -> usize {
    30
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchDomainParams {
    pub query: String,
    #[serde(default = "default_domain")]
    pub domain: String,
}

fn default_domain() -> String {
    "docs.python.org".to_string()
}



#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetVersionParams {}

#[derive(Clone)]
pub struct SearchServer {
    pub client: Client,
    pub state: Arc<Mutex<models::SearchState>>,
    pub user_agent: Arc<crate::user_agent::UserAgent>,
    pub prompt_router: PromptRouter<Self>,
    pub tool_router: rmcp::handler::server::router::tool::ToolRouter<Self>,
}

impl SearchServer {
    pub fn new(client: Client, state: Arc<Mutex<models::SearchState>>, user_agent: Arc<crate::user_agent::UserAgent>) -> Self {
        Self {
            client,
            state,
            user_agent,
            prompt_router: Self::prompt_router(),
            tool_router: Self::tool_router(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ResearchParams {
    pub topic: String,
    pub depth: Option<String>,
}

#[rmcp::prompt_router]
impl SearchServer {
    #[prompt(name = "research_topic", description = "Create a plan to research a specific topic")]
    async fn research_topic(&self, Parameters(params): Parameters<ResearchParams>) -> Result<GetPromptResult, McpError> {
        let depth = params.depth.as_deref().unwrap_or("comprehensive");
        Ok(GetPromptResult::new(vec![
            PromptMessage::new_text(
                PromptMessageRole::User,
                format!("I want to conduct a {} research on the topic: {}. Please help me find the most relevant information and synthesize it.", depth, params.topic),
            ),
        ]))
    }
}

#[rmcp::tool_router]
impl SearchServer {
    #[rmcp::tool(description = "Unified search tool for web content and news.")]
    async fn search_web(&self, Parameters(params): Parameters<SearchWebParams>) -> Result<CallToolResult, McpError> {
        let req = models::SearchRequest {
            query: params.query,
            search_type: params.search_type,
            max_results: params.max_results,
            time_range: None,
            region: None,
            safesearch: None,
            page: None,
            backend: None,
            filters: None,
        };
        
        let base_url = std::env::var("DDG_BASE_URL").unwrap_or_else(|_| "https://lite.duckduckgo.com/lite/".to_string());
        let res = search::ddg_search(&self.client, &self.user_agent, &self.state, req, &base_url).await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
            
        Ok(CallToolResult::success(vec![Annotated::new(RawContent::text(serde_json::to_string_pretty(&res).unwrap()), None)]))
    }


    #[rmcp::tool(description = "Extracts the full text content from a web page URL.")]
    async fn fetch_page(&self, Parameters(params): Parameters<FetchPageParams>) -> Result<CallToolResult, McpError> {
        let res = reader::fetch_page(
            &self.client, 
            &params.url, 
            &params.output_format, 
            params.include_metadata, 
            true, true, true, true, 
            params.max_length, 
            params.timeout as u64
        ).await.map_err(|e| McpError::internal_error(e.to_string(), None))?;
        
        Ok(CallToolResult::success(vec![Annotated::new(RawContent::text(serde_json::to_string_pretty(&res).unwrap()), None)]))
    }

    #[rmcp::tool(description = "Searches specifically for technical documentation or content on a specific domain.")]
    async fn search_domain(&self, Parameters(params): Parameters<SearchDomainParams>) -> Result<CallToolResult, McpError> {
        let res = research::search_domain(&self.client, &self.user_agent, &self.state, &params.query, &params.domain).await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
            
        Ok(CallToolResult::success(vec![Annotated::new(RawContent::text(serde_json::to_string_pretty(&res).unwrap()), None)]))
    }


    #[rmcp::tool(description = "Get the current version of the MCP server.")]
    async fn get_version(&self, Parameters(_params): Parameters<GetVersionParams>) -> Result<CallToolResult, McpError> {
        Ok(CallToolResult::success(vec![Annotated::new(RawContent::text(env!("CARGO_PKG_VERSION")), None)]))
    }
}

#[rmcp::tool_handler(name = "web-search-mcp-rust", instructions = "A web search and information retrieval MCP server")]
impl ServerHandler for SearchServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder()
            .enable_tools()
            .enable_resources()
            .enable_prompts()
            .build())
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        Ok(ListToolsResult::with_all_items(self.tool_router.list_all()))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let context = rmcp::handler::server::tool::ToolCallContext::new(self, request, _context);
        self.tool_router.call(context).await
    }

    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        Ok(ListPromptsResult::with_all_items(self.prompt_router.list_all()))
    }

    async fn get_prompt(
        &self,
        request: GetPromptRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        let context = rmcp::handler::server::prompt::PromptContext::new(
            self, 
            request.name, 
            request.arguments, 
            _context
        );
        self.prompt_router.get_prompt(context).await
    }

    async fn list_resources(

        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        Ok(ListResourcesResult::with_all_items(vec![
            RawResource::new("web://search/docs", "Search Documentation").no_annotation(),
        ]))
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        if request.uri == "web://search/docs" {
            Ok(ReadResourceResult::new(vec![ResourceContents::text("Web Search MCP Server Documentation\n\nThis server provides tools for web search and page extraction.", &request.uri)]))
        } else {
            Err(McpError::resource_not_found("resource_not_found", Some(serde_json::json!({ "uri": request.uri })),))
        }
    }
}
