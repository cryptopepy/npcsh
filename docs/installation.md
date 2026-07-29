## Installation

`npcsh` is distributed as pre-built Rust binaries and as a Rust crate. Pick the option that fits your workflow.

## Install script (recommended)

```bash
curl -fsSL https://enpisi.com/install-npcsh.sh | sh
```

The script downloads the latest `npcsh` and `npc` binaries for your platform into `~/.npcsh/bin`. Make sure that directory is on your PATH:

```bash
export PATH="$HOME/.npcsh/bin:$PATH"
```

Then run:

```bash
npcsh
```

## Cargo

```bash
cargo install npcsh
```

This installs the `npcsh` and `npc` binaries via crates.io.

## System dependencies

### Models

`npcsh` works with any model provider that LiteLLM supports, including local and hosted options.

Local options:
- [Ollama](https://ollama.com) — `ollama pull qwen3.5:2b`
- [LM Studio](https://lmstudio.ai) — start the local server and use `openai-like` provider
- [MLX](https://github.com/ml-explore/mlx) / `npcpy` MLX provider — local Apple Silicon models

Hosted providers:
- OpenRouter, OpenAI, Anthropic, Gemini, DeepSeek, Moonshot, Minimax, and others

Set the model and provider in `~/.npcshrc` or per-agent:

```bash
export NPCSH_CHAT_MODEL=qwen3.5:2b
export NPCSH_CHAT_PROVIDER=ollama
```

### Model recommendations by available VRAM

These are rough Ollama q4-equivalent fits. Actual usage depends on context length, quantization, and whether the model is MoE/dense. If you do not have a local GPU, use a hosted provider for the best scores.

| Available VRAM | Try this first | Score | Notes |
|------------------|----------------|-------|-------|
| CPU / no GPU, or < 4 GB | `qwen3.5:2b` | 66/100 | Fast enough for basic tasks; `0.8b` works but scores 23/100 |
| 8 GB | `qwen3.5:4b` | 85/100 | Sweet spot for small GPUs; `granite4.1:3b` is an alternative at 68/100 |
| 12 GB | `qwen3.5:9b` | 95/100 | Strong performance without needing a large card |
| 16 GB | `gemma4:26b` | 96/100 | Tight fit at 16 GB; `qwen3.5:9b` is the safer fallback |
| 24 GB | `qwen3.5:35b` | 97/100 | Dense 35B model; also fits `granite4.1:30b` (94/100) or `gemma4:31b` (92/100) |
| 32 GB+ | `qwen3.5:397b` | 96/100 | Largest tested MoE; only if your setup can load it |

For the best results without worrying about VRAM, use a hosted provider such as OpenRouter, OpenAI, Anthropic, or Gemini.

### Windows

WSL is recommended for running `npcsh` on Windows. You can also install the binaries via the install script or cargo inside WSL.

## Rust build (development / latest)

To build the Rust binaries from source:

```bash
cd npcsh/rust
cargo build --release
cp target/release/npcsh ~/.npcsh/bin/npcsh
cp target/release/npc ~/.npcsh/bin/npc
```

For normal use, install the pre-built release via the install script or cargo. The source build is for development only.

## Startup and configuration

Start the shell by typing:

```bash
npcsh
```

When initialized, `npcsh` generates a `.npcshrc` file in your home directory that stores your settings — default chat model/provider, image generation model/provider, embedding model/provider, database path, etc.

```bash
export NPCSH_CHAT_MODEL=qwen3.5:2b
export NPCSH_CHAT_PROVIDER=ollama
export NPCSH_DEFAULT_MODE=agent
export NPCSH_EMBEDDING_MODEL=nomic-embed-text
export NPCSH_EMBEDDING_PROVIDER=ollama
export NPCSH_STREAM_OUTPUT=1
```

The installer tries to source this file from your shell config automatically. If it does not (for example, you use an alternative rc file), add this to `.bashrc` or `.zshrc`:

```bash
if [ -f ~/.npcshrc ]; then
    . ~/.npcshrc
fi
```

`npcsh` supports inference via all major providers through LiteLLM, including but not limited to `openai`, `anthropic`, `ollama`, `gemini`, `deepseek`, and `openai-like` APIs. The `openai-like` provider is intended for custom or locally hosted servers (LM Studio, Llama CPP, etc.).

API keys can be placed in a project `.env` file, in `~/.npcshrc`, or in your existing shell config. `npcsh` always checks the current folder's `.env` first, so you can use per-project keys without manually switching them.

```bash
export OPENAI_API_KEY="your_openai_key"
export ANTHROPIC_API_KEY="your_anthropic_key"
export GEMINI_API_KEY="your_gemini_key"
export DEEPSEEK_API_KEY="your_deepseek_key"
```

Individual NPCs can override the default model/provider by setting `model` and `provider` in their `.npc` files.

## Project structure

A project has three layers: team context, agents, and tools. You can keep them under an `npc_team/` directory or at the project root. The agent layer can be `.npc` files, a single `agents.md`, or an `agents/` directory — these are alternatives, not layers to combine.

```
./npc_team/
├── team.ctx            # team-level context
├── example1.npc        # agent definition
├── example2.npc
└── jinxes/             # tools
    └── example.jinx
```

Or use a flat layout:

```
./
├── team.ctx            # team-level context
├── agents.md           # many agents in one file
└── jinxes/             # tools
    └── example.jinx
```

If both `npc_team/*.npc` and `agents.md`/`agents/` are present, `npcsh` asks which agent layout to use on first run and saves the choice in `.NPCSH_PREFERRED_TEAM_NAME`. Later runs use the preferred layout automatically.

State such as conversation history, images, screenshots, jobs, and triggers is stored under `~/.npcsh/` (and in `~/npcsh_history.db` by default).
