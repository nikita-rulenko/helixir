

use rmcp::{
    handler::server::{
        router::tool::ToolRouter,
        router::prompt::PromptRouter,
        wrapper::Parameters,
    },
    model::*,
    tool, tool_handler, tool_router,
    prompt, prompt_handler, prompt_router,
    transport::stdio,
    service::RequestContext,
    ErrorData as McpError, RoleServer, ServerHandler, ServiceExt,
};
use rmcp::schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::core::config::HelixirConfig;
use crate::core::helixir_client::{HelixirClient, HelixirClientError};


#[derive(Debug, Deserialize, rmcp::schemars::JsonSchema)]
pub struct AddMemoryParams {
    #[schemars(description = "Text to remember (will be extracted into atomic facts)")]
    pub message: String,
    #[schemars(description = "User identifier (e.g., 'claude', 'developer')")]
    pub user_id: String,
    #[schemars(description = "Optional agent identifier")]
    pub agent_id: Option<String>,
}

#[derive(Debug, Deserialize, rmcp::schemars::JsonSchema)]
pub struct SearchMemoryParams {
    #[schemars(description = "Search query")]
    pub query: String,
    #[schemars(description = "User identifier")]
    pub user_id: String,
    #[schemars(description = "Max results (default: mode-based)")]
    pub limit: Option<i32>,
    #[schemars(
        description = "Search mode: 'recent' (4h), 'contextual' (30d), 'deep' (90d), 'full'"
    )]
    pub mode: Option<String>,
    #[schemars(description = "Override time window in days")]
    pub temporal_days: Option<f64>,
    #[schemars(description = "Override graph depth")]
    pub graph_depth: Option<i32>,
}

#[derive(Debug, Deserialize, rmcp::schemars::JsonSchema)]
pub struct UpdateMemoryParams {
    #[schemars(description = "Memory ID to update")]
    pub memory_id: String,
    #[schemars(description = "New content")]
    pub new_content: String,
    #[schemars(description = "User identifier")]
    pub user_id: String,
}

#[derive(Debug, Deserialize, rmcp::schemars::JsonSchema)]
pub struct GetMemoryGraphParams {
    #[schemars(description = "User identifier")]
    pub user_id: String,
    #[schemars(description = "Optional starting point memory ID")]
    pub memory_id: Option<String>,
    #[schemars(description = "Traversal depth (default: 2)")]
    pub depth: Option<i32>,
}

#[derive(Debug, Deserialize, rmcp::schemars::JsonSchema)]
pub struct SearchByConceptParams {
    #[schemars(description = "Search query (semantic matching)")]
    pub query: String,
    #[schemars(description = "User identifier")]
    pub user_id: String,
    #[schemars(
        description = "Concept type: 'skill', 'preference', 'goal', 'fact', 'opinion', 'experience', 'achievement'"
    )]
    pub concept_type: Option<String>,
    #[schemars(description = "Comma-separated tags to filter by")]
    pub tags: Option<String>,
    #[schemars(description = "Search mode")]
    pub mode: Option<String>,
    #[schemars(description = "Max results (default: 10)")]
    pub limit: Option<i32>,
}

#[derive(Debug, Deserialize, rmcp::schemars::JsonSchema)]
pub struct SearchReasoningChainParams {
    #[schemars(description = "Search query")]
    pub query: String,
    #[schemars(description = "User identifier")]
    pub user_id: String,
    #[schemars(
        description = "Chain mode: 'causal' (BECAUSE), 'forward' (IMPLIES), 'both', 'deep'"
    )]
    pub chain_mode: Option<String>,
    #[schemars(description = "Maximum chain depth (default: 5)")]
    pub max_depth: Option<i32>,
    #[schemars(description = "Number of seed memories")]
    pub limit: Option<i32>,
}


#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct MemorySummaryArgs {
    #[schemars(description = "User identifier")]
    pub user_id: String,
    #[schemars(description = "Optional topic to focus on")]
    pub topic: Option<String>,
}


#[derive(Clone)]
pub struct HelixirMcpServer {
    client: Arc<RwLock<HelixirClient>>,
    tool_router: ToolRouter<Self>,
    prompt_router: PromptRouter<Self>,
}

impl HelixirMcpServer {
    
    pub fn new(client: HelixirClient) -> Self {
        Self {
            client: Arc::new(RwLock::new(client)),
            tool_router: Self::tool_router(),
            prompt_router: Self::prompt_router(),
        }
    }

    
    fn convert_error(err: HelixirClientError) -> McpError {
        match err {
            HelixirClientError::Config(msg) => McpError::invalid_params(msg, None),
            HelixirClientError::Database(msg) => McpError::internal_error(msg, None),
            HelixirClientError::Llm(msg) => McpError::internal_error(msg, None),
            HelixirClientError::Embedding(msg) => McpError::internal_error(msg, None),
            HelixirClientError::Tooling(msg) => McpError::internal_error(msg, None),
            HelixirClientError::NotInitialized => {
                McpError::internal_error("Client not initialized", None)
            }
            HelixirClientError::Operation(msg) => McpError::internal_error(msg, None),
        }
    }

    
    fn result_to_json<T: Serialize>(result: T) -> Result<String, McpError> {
        serde_json::to_string_pretty(&result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }
}

#[tool_router]
impl HelixirMcpServer {
    
    #[tool(description = "Add memory with LLM-powered extraction. Extracts atomic facts, generates embeddings, creates graph relations. Returns: {memories_added, entities, relations, memory_ids, chunks_created}")]
    async fn add_memory(
        &self,
        Parameters(params): Parameters<AddMemoryParams>,
    ) -> Result<CallToolResult, McpError> {
        info!("🧠 Adding memory for user={}", params.user_id);

        let client = self.client.read().await;
        let result = client
            .add(&params.message, &params.user_id, params.agent_id.as_deref(), None)
            .await
            .map_err(Self::convert_error)?;

        info!(
            "✅ Added {} memories ({} chunks)",
            result.memories_added,
            result.chunks_created
        );

        let json = Self::result_to_json(&result)?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    
    #[tool(description = "Smart memory search with automatic strategy selection. Modes: 'recent' (4h, fast), 'contextual' (30d, balanced), 'deep' (90d), 'full' (all). Returns: [{memory_id, content, score, metadata}]")]
    async fn search_memory(
        &self,
        Parameters(params): Parameters<SearchMemoryParams>,
    ) -> Result<CallToolResult, McpError> {
        let mode = params.mode.unwrap_or_else(|| "recent".to_string());
        let limit = params.limit.map(|l| l as usize);

        let query_preview: String = params.query.chars().take(50).collect();
        info!(
            "🔍 Searching: '{}' [mode={}, limit={:?}]",
            query_preview,
            mode,
            limit
        );

        let client = self.client.read().await;
        let results = client
            .search(
                &params.query,
                &params.user_id,
                limit,
                Some(&mode),
                params.temporal_days,
                params.graph_depth.map(|d| d as usize),
            )
            .await
            .map_err(Self::convert_error)?;

        info!("✅ Found {} memories", results.len());

        let json = Self::result_to_json(&results)?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    
    #[tool(description = "Update memory content (regenerates embedding & relations). Returns: {updated: bool, memory_id}")]
    async fn update_memory(
        &self,
        Parameters(params): Parameters<UpdateMemoryParams>,
    ) -> Result<CallToolResult, McpError> {
        let id_preview: String = params.memory_id.chars().take(12).collect();
        info!("✏️ Updating memory: {}...", id_preview);

        let client = self.client.read().await;
        let result = client
            .update(&params.memory_id, &params.new_content, &params.user_id)
            .await
            .map_err(Self::convert_error)?;

        if result.updated {
            info!("✅ Memory updated");
        } else {
            warn!("⚠️ Memory update failed");
        }

        let json = Self::result_to_json(&result)?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    
    #[tool(description = "Get memory graph visualization. Returns: {nodes: [...], edges: [...]}")]
    async fn get_memory_graph(
        &self,
        Parameters(params): Parameters<GetMemoryGraphParams>,
    ) -> Result<CallToolResult, McpError> {
        info!("📊 Getting memory graph for user={}", params.user_id);

        let client = self.client.read().await;
        let result = client
            .get_graph(&params.user_id, params.memory_id.as_deref(), params.depth.map(|d| d as usize))
            .await
            .map_err(Self::convert_error)?;

        let json = Self::result_to_json(&result)?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    
    #[tool(description = "Search memories by ontology concepts. Concept types: 'skill', 'preference', 'goal', 'fact', 'opinion', 'experience', 'achievement'. Returns: [{memory_id, content, concept_score}]")]
    async fn search_by_concept(
        &self,
        Parameters(params): Parameters<SearchByConceptParams>,
    ) -> Result<CallToolResult, McpError> {
        let query_preview: String = params.query.chars().take(30).collect();
        info!(
            "🎯 Concept search: '{}' type={:?}",
            query_preview,
            params.concept_type
        );

        let client = self.client.read().await;
        let results = client
            .search_by_concept(
                &params.query,
                &params.user_id,
                params.concept_type.as_deref(),
                params.tags.as_deref(),
                params.mode.as_deref(),
                params.limit.map(|l| l as usize),
            )
            .await
            .map_err(Self::convert_error)?;

        info!("✅ Found {} memories", results.len());

        let json = Self::result_to_json(&results)?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    
    #[tool(description = "Search with logical reasoning chains (IMPLIES/BECAUSE/CONTRADICTS). Chain modes: 'causal' (why?), 'forward' (effects), 'both', 'deep'. Returns: {chains: [...], deepest_chain}")]
    async fn search_reasoning_chain(
        &self,
        Parameters(params): Parameters<SearchReasoningChainParams>,
    ) -> Result<CallToolResult, McpError> {
        let chain_mode = params.chain_mode.unwrap_or_else(|| "both".to_string());

        let query_preview: String = params.query.chars().take(30).collect();
        info!(
            "🔗 Reasoning chain: '{}' mode={}",
            query_preview,
            chain_mode
        );

        let client = self.client.read().await;
        let result = client
            .search_reasoning_chain(
                &params.query,
                &params.user_id,
                Some(&chain_mode),
                params.max_depth.map(|d| d as usize),
                params.limit.map(|l| l as usize),
            )
            .await
            .map_err(Self::convert_error)?;

        info!("✅ Found {} chains", result.chains.len());

        let json = Self::result_to_json(&result)?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }
}


#[prompt_router]
impl HelixirMcpServer {
    
    #[prompt(
        name = "memory_summary",
        description = "Generate prompt to summarize user's memories on a topic"
    )]
    async fn memory_summary(
        &self,
        Parameters(args): Parameters<MemorySummaryArgs>,
    ) -> Result<GetPromptResult, McpError> {
        let topic_filter = args.topic
            .map(|t| format!(" about {}", t))
            .unwrap_or_default();

        let messages = vec![
            PromptMessage::new_text(
                PromptMessageRole::User,
                format!(
                    "Analyze memories for user_id={}{}.

Use search_memory tool to find relevant memories.
Provide a summary with:
1. Key patterns and themes
2. Important facts and preferences  
3. Connections between memories
4. Timeline of events",
                    args.user_id,
                    topic_filter
                ),
            ),
        ];

        Ok(GetPromptResult {
            description: Some(format!("Memory summary for {}", args.user_id)),
            messages,
        })
    }

    
    #[prompt(
        name = "tool_selection_guide",
        description = "Guide for AI to select the right memory tool for each task"
    )]
    async fn tool_selection_guide(&self) -> Result<GetPromptResult, McpError> {
        let guide = r#"# 🧠 Helixir Memory Tools - Selection Guide

You have access to powerful memory tools. Choose the RIGHT tool for each task:

## 📝 add_memory
**When to use:** Storing new information, facts, decisions, events
**Examples:**
- "Remember that we decided to use Rust for performance"
- "Save this: the API endpoint is /v1/memories"

## 🔍 search_memory (default search)
**When to use:** General queries, finding relevant context, quick lookups
**Modes:**
- recent: Quick search in last 4 hours (default, fastest)
- contextual: Balanced search, last 30 days
- deep: Thorough search, last 90 days
- full: Complete history search

## 🎯 search_by_concept (OntoSearch)
**When to use:** Searching by TYPE of memory or specific tags
**Concept types:** skill, preference, goal, fact, opinion, experience, achievement

## 🔗 search_reasoning_chain (ChainSearch)
**When to use:** Understanding WHY or tracing logical connections
**Chain modes:** causal (why?), forward (what follows?), both, deep

## 🕸️ get_memory_graph
**When to use:** Visualizing connections, exploring memory structure

## ✏️ update_memory
**When to use:** Correcting outdated information

---

## 🎯 Quick Decision Tree:

1. **Storing info?** → add_memory
2. **General "what do I know"?** → search_memory
3. **Asking about skills/preferences/goals?** → search_by_concept
4. **Asking "why" or "what follows"?** → search_reasoning_chain
5. **Want to see connections?** → get_memory_graph"#;

        let messages = vec![
            PromptMessage::new_text(PromptMessageRole::Assistant, guide.to_string()),
        ];

        Ok(GetPromptResult {
            description: Some("Tool selection guide for Helixir memory operations".to_string()),
            messages,
        })
    }
}


#[tool_handler]
#[prompt_handler]
impl ServerHandler for HelixirMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_prompts()
                .enable_resources()
                .build(),
            server_info: Implementation {
                name: "helixir".into(),
                version: "2.0.0".into(),
                ..Default::default()
            },
            instructions: Some(
                "Helixir Memory Management for AI - Ontological memory with LLM-powered extraction, \
                 graph reasoning, and multi-strategy search. Use add_memory to store, search_memory \
                 for retrieval, and search_reasoning_chain for logical inference."
                    .to_string(),
            ),
        }
    }

    
    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParam>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        Ok(ListResourcesResult {
            resources: vec![
                RawResource::new("config://helixir", "helixir-config".to_string())
                    .no_annotation(),
                RawResource::new("status://helixdb", "helixdb-status".to_string())
                    .no_annotation(),
            ],
            next_cursor: None,
        })
    }

    async fn read_resource(
        &self,
        ReadResourceRequestParam { uri }: ReadResourceRequestParam,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        match uri.as_str() {
            "config://helixir" => {
                let client = self.client.read().await;
                let config = client.config();
                
                let content = serde_json::to_string_pretty(&json!({
                    "version": "2.0.0",
                    "helixdb": {
                        "host": config.host,
                        "port": config.port,
                        "instance": config.instance,
                    },
                    "llm": {
                        "provider": config.llm_provider,
                        "model": config.llm_model,
                    },
                    "capabilities": {
                        "memory_management": true,
                        "vector_search": true,
                        "graph_traversal": true,
                        "llm_extraction": true,
                        "entity_linking": true,
                        "ontology_mapping": true,
                        "onto_search": true,
                        "reasoning_chains": true,
                    },
                    "tools": [
                        "add_memory",
                        "search_memory",
                        "search_by_concept",
                        "search_reasoning_chain",
                        "get_memory_graph",
                        "update_memory",
                    ],
                })).unwrap_or_default();

                Ok(ReadResourceResult {
                    contents: vec![ResourceContents::text(content, uri)],
                })
            }
            "status://helixdb" => {
                let client = self.client.read().await;
                let config = client.config();
                
                let content = serde_json::to_string_pretty(&json!({
                    "status": "connected",
                    "host": config.host,
                    "port": config.port,
                    "instance": config.instance,
                })).unwrap_or_default();

                Ok(ReadResourceResult {
                    contents: vec![ResourceContents::text(content, uri)],
                })
            }
            _ => Err(McpError::resource_not_found(
                format!("Unknown resource: {}", uri),
                Some(json!({ "uri": uri })),
            )),
        }
    }
}


pub async fn run_server() -> anyhow::Result<()> {
    info!("🚀 Initializing Helixir MCP Server...");

    let config = HelixirConfig::from_env();
    let client = HelixirClient::new(config)?;
    client.initialize().await?;

    info!("✅ Helixir MCP Server ready");
    info!(
        "   📍 HelixDB: {}:{}",
        client.config().host,
        client.config().port
    );
    info!(
        "   🤖 LLM: {}/{}",
        client.config().llm_provider,
        client.config().llm_model
    );
    info!("   📊 Instance: {}", client.config().instance);

    let server = HelixirMcpServer::new(client);
    let service = server.serve(stdio()).await?;
    service.waiting().await?;

    Ok(())
}
