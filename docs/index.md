# npcsh Documentation

`npcsh` is a shell for portable, composable multi-agent teams. Team context, agents, and tools are defined as plain files; the shell compiles them into a live multi-agent system you can chat with, schedule, or extend with custom tools.

## Quick Links

- [Installation Guide](installation.md)
- [Skills](skills.md) — knowledge-content jinxes with progressive section disclosure

## The Team

The bundled team ships with one default agent and several specialists. The default agent is `corca`, who acts as both the coding/shell specialist and the team orchestrator.

| Agent | Role | Key jinxes |
|-------|------|------------|
| `corca` | Orchestrator, coding, shell, files | `read`, `write`, `execute`, `explore`, `delegate`, `convene`, `python`, `shell`, `web_search`, `chat`, `stop` |
| `alicanto` | Deep research, papers, literature review | `python`, `shell`, `load_file` |
| `frederic` | Creative/math, unconventional solutions | `python`, `vixynt`, `roll`, `sample` |
| `kadiefa` | Exploratory thinking, unexpected connections | `python`, `shell`, `web_search` |
| `plonk` | Browser/GUI automation, screenshots | `computer_use`, `screenshot`, `browser_action` |

Switch to an agent inside the shell with `/<agent>` or ask a one-off with `@<agent>`:

```bash
npcsh> @corca refactor the auth module and add tests
npcsh> @alicanto summarize the last three papers on transformers
```

## Common Commands

| Command | What it does |
|---------|--------------|
| `/agent` | Full agent mode: the NPC can call jinxes, run bash, and use the LLM. |
| `/chat` | Chat-only mode: LLM responses without tool use. |
| `/cmd` | Command mode: input is run as bash first; if it fails, fall back to the LLM. |
| `/<agent>` | Switch the current session to the named agent (e.g., `/corca`). |
| `@<agent>` | Ask a one-off question to an agent without switching. |
| `/jinxes` | List the jinxes available to the current team. |
| `/help` | Show the built-in help. |

The CLI also exposes agents via `npc`:

```bash
npc ./npc_team/corca.npc "what is the biggest file on my computer?"
npc chat -n corca
```

## The CAT Data Layer

Everything customizable in `npcsh` lives as simple files across three layers:

| Layer | Files | Purpose |
|-------|-------|---------|
| **Context** | `.ctx` / `team.ctx` / `npc_team/*.ctx` | Shared team context: default model/provider, env vars, MCP servers |
| **Agents** | `.npc`, `agents.md`, `agents/` | Agent definitions: name, persona, directive, model/provider, and jinxes |
| **Tools** | `.jinx`, `skills/` | Reusable tools and workflows that agents invoke by name |

Files can live inside `npc_team/` or at the project root. The agent layer can use `.npc` files, a single `agents.md`, or an `agents/` directory — these are alternatives, not a required combination.

Because these are ordinary files, you can version them in git, share them across projects, and drop in agent definitions from other ecosystems.

## Contributing

Contributions are welcome! Submit issues and pull requests on the [GitHub repository](https://github.com/npc-worldwide/npcsh).

## License

This project is licensed under the MIT License.
