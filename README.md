<p align="center">
  <img src="helixir-rs-logo.jpeg" alt="Helixir Logo" width="400"/>
</p>

<h1 align="center">🧠 Helixir-RS</h1>

<p align="center">
  <strong>The Fastest Memory for LLM Agents</strong><br/>
  <em>Rust implementation of the Helixir ontological memory framework</em>
</p>

<p align="center">
  <a href="#-quick-start">Quick Start</a> •
  <a href="#-features">Features</a> •
  <a href="#-mcp-integration">MCP Integration</a> •
  <a href="#-configuration">Configuration</a> •
  <a href="https://github.com/nikita-rulenko/helixir">Python Version</a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/rust-1.75+-orange.svg" alt="Rust"/>
  <img src="https://img.shields.io/badge/license-AGPL--3.0-blue.svg" alt="License"/>
  <img src="https://img.shields.io/badge/MCP-compatible-green.svg" alt="MCP"/>
</p>

---

## What is Helixir-RS?

**Helixir-RS** is the high-performance Rust version of [Helixir](https://github.com/nikita-rulenko/helixir) — an associative & causal AI memory framework.

It gives your AI agents **persistent, structured, reasoning-capable memory**. Instead of losing context between sessions, your AI remembers facts, learns preferences, tracks goals, and builds knowledge over time.

Built on [HelixDB](https://github.com/HelixDB/helix-db) graph-vector database with native [MCP](https://spec.modelcontextprotocol.io/) support for seamless integration with **Cursor**, **Claude Desktop**, and other AI assistants.

### ⚡ Recommended Stack: Cerebras + OpenRouter

For **maximum speed**, use:
- **[Cerebras](https://cloud.cerebras.ai)** for LLM inference — 70x faster than GPU, free tier available
- **[OpenRouter](https://openrouter.ai)** for embeddings — cheap, reliable, many models

This combination delivers **sub-second memory operations** with the 70B parameter Llama 3.3 model.

### 🦀 Why Rust?

This is the **high-performance Rust implementation** of Helixir. Compared to the [Python version](https://github.com/nikita-rulenko/helixir):

| | Rust | Python |
|---|:---:|:---:|
| **Startup time** | ~50ms | ~2s |
| **Memory usage** | ~15MB | ~150MB |
| **Binary size** | 15MB standalone | Requires Python runtime |
| **Dependencies** | Zero runtime deps | pip/uv + packages |
| **Deployment** | Single binary | Virtual env setup |

Same features, 10x faster, zero dependencies.

---

## ✨ Features

- **🔬 Atomic Fact Extraction** — LLM-powered decomposition into atomic facts
- **🧹 Smart Deduplication** — ADD / UPDATE / SUPERSEDE / NOOP decision engine  
- **🕸️ Graph Memory** — Entities, relations, and reasoning chains
- **🔍 Semantic Search** — Vector similarity + graph traversal (SmartTraversalV2)
- **⏰ Temporal Filtering** — recent (4h), contextual (30d), deep (90d), full
- **🏷️ Ontology Mapping** — skill, preference, goal, fact, opinion, experience, achievement
- **📡 MCP Server** — Native integration with AI assistants
- **🧩 Semantic Chunking** — Automatic splitting of long texts

---

## 🚀 Quick Start

### One-Command Setup (Docker)

```bash
# Clone and start everything
git clone https://github.com/nikita-rulenko/helixir-rs
cd helixir-rs

# Create config
cat > .env << 'EOF'
LLM_API_KEY=your_cerebras_or_openai_key
EMBEDDING_API_KEY=your_openrouter_or_openai_key
EOF

# Start HelixDB + deploy schema
docker-compose up -d
```

**Requirements:**
- Docker & Docker Compose installed
- API keys (see [Configuration](#-configuration))

### Manual Installation

```bash
# 1. Download binary for your platform
# Linux x86_64
curl -L https://github.com/nikita-rulenko/helixir-rs/releases/latest/download/helixir-linux-x86_64.tar.gz | tar xz

# macOS Apple Silicon  
curl -L https://github.com/nikita-rulenko/helixir-rs/releases/latest/download/helixir-macos-arm64.tar.gz | tar xz

# macOS Intel
curl -L https://github.com/nikita-rulenko/helixir-rs/releases/latest/download/helixir-macos-x86_64.tar.gz | tar xz

# 2. Start HelixDB (if not running)
docker run -d -p 6969:6969 helixdb/helixdb:latest

# 3. Deploy schema
./helixir-deploy --host localhost --port 6969

# 4. Run MCP server
export LLM_API_KEY=your_key
export EMBEDDING_API_KEY=your_key
./helixir-mcp
```

### Build from Source

```bash
git clone https://github.com/nikita-rulenko/helixir-rs
cd helixir-rs

# Build
cargo build --release

# Deploy schema & run
./target/release/helixir-deploy --host localhost --port 6969
./target/release/helixir-mcp
```

---

## 🔧 MCP Integration

### Cursor IDE

Edit `~/.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "helixir": {
      "command": "/path/to/helixir-mcp",
      "env": {
        "HELIX_HOST": "localhost",
        "HELIX_PORT": "6969",
        "LLM_PROVIDER": "cerebras",
        "LLM_MODEL": "llama-3.3-70b",
        "LLM_API_KEY": "YOUR_API_KEY",
        "EMBEDDING_PROVIDER": "openai",
        "EMBEDDING_URL": "https://openrouter.ai/api/v1",
        "EMBEDDING_API_KEY": "YOUR_API_KEY"
      }
    }
  }
}
```

### Claude Desktop

**macOS:** `~/Library/Application Support/Claude/claude_desktop_config.json`  
**Windows:** `%APPDATA%\Claude\claude_desktop_config.json`

```json
{
  "mcpServers": {
    "helixir": {
      "command": "/path/to/helixir-mcp",
      "env": {
        "HELIX_HOST": "localhost",
        "HELIX_PORT": "6969",
        "LLM_API_KEY": "YOUR_API_KEY",
        "EMBEDDING_API_KEY": "YOUR_API_KEY"
      }
    }
  }
}
```

### Cursor Rules (Important!)

To make your AI assistant actually USE the memory, add these rules to **Cursor Settings → Rules**:

```
- Always use Helixir MCP to remember important things about the project
- Always use Helixir MCP first to recall context about the current project
- At the start of chat, store the user's prompt to always remember your role and goals
- After reaching context window limit (when Cursor summarizes), read your role and user goals from memory again
- For memory search, use appropriate mode:
  - "recent" for quick context (last 4 hours)
  - "contextual" for balanced search (30 days)
  - "deep" for thorough search (90 days)
  - "full" for complete history
- Use search_by_concept for skill/preference/goal queries
- Use search_reasoning_chain for "why" questions and logical connections
```

---

## 📚 MCP Tools

| Tool | Description |
|------|-------------|
| `add_memory` | Add memory with LLM extraction → `{memories_added, entities, relations, chunks_created}` |
| `search_memory` | Smart search: `recent` (4h), `contextual` (30d), `deep` (90d), `full` |
| `search_by_concept` | Filter by type: `skill`, `goal`, `preference`, `fact`, `opinion`, `experience`, `achievement` |
| `search_reasoning_chain` | Find logical connections: `IMPLIES`, `BECAUSE`, `CONTRADICTS` |
| `get_memory_graph` | Visualize memory as nodes and edges |
| `update_memory` | Update existing memory content |

### Usage Examples

**Store a preference:**
```
"Remember that I prefer dark mode in all applications"
→ add_memory extracts: preference about UI settings
```

**Recall context:**
```
"What do you know about my coding preferences?"
→ search_by_concept(concept_type="preference") 
→ Returns: dark mode preference, editor settings, etc.
```

**Find reasoning chains:**
```
"Why did we decide to use Rust for this project?"
→ search_reasoning_chain(chain_mode="causal")
→ Returns: decision → because → performance requirements
```

**Quick session context:**
```
"What were we working on?"
→ search_memory(mode="recent") 
→ Returns: last 4 hours of activity
```

---

## 📊 Search Modes

| Mode | Time Window | Graph Depth | Use Case |
|------|:-----------:|:-----------:|----------|
| `recent` | 4 hours | 1 | Current session context |
| `contextual` | 30 days | 2 | Balanced (default) |
| `deep` | 90 days | 3 | Thorough historical search |
| `full` | All time | 4 | Complete memory archive |

---

## ⚙️ Configuration

### Environment Variables

| Variable | Required | Default | Description |
|----------|:--------:|---------|-------------|
| `HELIX_HOST` | ✅ | `localhost` | HelixDB server address |
| `HELIX_PORT` | ✅ | `6969` | HelixDB port |
| `LLM_API_KEY` | ✅ | — | API key for LLM provider |
| `EMBEDDING_API_KEY` | ✅ | — | API key for embeddings |
| `LLM_PROVIDER` | | `cerebras` | `cerebras`, `openai`, `ollama` |
| `LLM_MODEL` | | `llama-3.3-70b` | Model name |
| `LLM_BASE_URL` | | — | Custom endpoint (Ollama) |
| `EMBEDDING_PROVIDER` | | `openai` | `openai`, `ollama` |
| `EMBEDDING_URL` | | `https://openrouter.ai/api/v1` | Embedding API URL |
| `EMBEDDING_MODEL` | | `all-mpnet-base-v2` | Embedding model |

### Provider Configurations

#### Option 1: Cerebras + OpenRouter (Recommended)

Ultra-fast inference + cheap embeddings:

```bash
LLM_PROVIDER=cerebras
LLM_MODEL=llama-3.3-70b
LLM_API_KEY=csk-xxx              # https://cloud.cerebras.ai

EMBEDDING_PROVIDER=openai
EMBEDDING_URL=https://openrouter.ai/api/v1
EMBEDDING_MODEL=openai/text-embedding-3-large
EMBEDDING_API_KEY=sk-or-xxx      # https://openrouter.ai/keys
```

#### Option 2: Fully Local (Ollama)

No API keys, fully private:

```bash
# Install Ollama first: curl -fsSL https://ollama.com/install.sh | sh
ollama pull llama3:8b
ollama pull nomic-embed-text

LLM_PROVIDER=ollama
LLM_MODEL=llama3:8b
LLM_BASE_URL=http://localhost:11434

EMBEDDING_PROVIDER=ollama
EMBEDDING_URL=http://localhost:11434
EMBEDDING_MODEL=nomic-embed-text
```

#### Option 3: OpenAI Only

Simple setup, one API key:

```bash
LLM_PROVIDER=openai
LLM_MODEL=gpt-4o-mini
LLM_API_KEY=sk-xxx

EMBEDDING_PROVIDER=openai
EMBEDDING_MODEL=text-embedding-3-small
EMBEDDING_API_KEY=sk-xxx
```

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      MCP Server (stdio)                      │
├─────────────────────────────────────────────────────────────┤
│                      HelixirClient                           │
├─────────────────────────────────────────────────────────────┤
│                     ToolingManager                           │
├──────────┬──────────┬──────────┬──────────┬────────────────┤
│ LLM      │ Decision │ Entity   │ Reasoning│ Search         │
│ Extractor│ Engine   │ Manager  │ Engine   │ Engine         │
├──────────┴──────────┴──────────┴──────────┴────────────────┤
│                      HelixDB Client                          │
├─────────────────────────────────────────────────────────────┤
│                        HelixDB                               │
└─────────────────────────────────────────────────────────────┘
```

---

## 🐳 Docker

### Full Stack (HelixDB + Helixir)

```bash
# Start everything
docker-compose up -d

# Check logs
docker-compose logs -f helixir-mcp
```

### Standalone

```bash
# Build
docker build -t helixir-rs .

# Run with external HelixDB
docker run -e HELIX_HOST=your_helixdb_host \
           -e LLM_API_KEY=xxx \
           -e EMBEDDING_API_KEY=xxx \
           helixir-rs
```

---

## 🧪 Development

```bash
# Run tests
cargo test

# Verbose logging
RUST_LOG=helixir=debug cargo run --bin helixir-mcp

# Lint
cargo clippy
cargo fmt --check
```

---

## 📄 License

[AGPL-3.0-or-later](LICENSE)

⚠️ **This is NOT MIT!** If you modify and deploy Helixir as a service, you must open-source your codebase.

---

## 🔗 Links

- [HelixDB](https://github.com/HelixDB/helix-db) — Graph-vector database
- [Helixir (Python)](https://github.com/nikita-rulenko/helixir) — Python version
- [MCP Specification](https://spec.modelcontextprotocol.io/) — Model Context Protocol
- [Cerebras](https://cloud.cerebras.ai) — Fast LLM inference (free tier)
- [OpenRouter](https://openrouter.ai) — Unified LLM/embedding API
