# OpenStudio

OpenStudio is a local-first desktop studio for producing on-brand social visuals with the AI model you already use. It is a Tauri application: the interface is React/TypeScript and the application core is Rust.

There is no OpenStudio backend, account, telemetry, or hosted database. Projects and brand kits are stored in a local SQLite database; BYOK credentials are saved only in the operating system credential store.

## What it does

- Create reusable artistic directions: colours, typography, voice, and constraints.
- Create an editable visual brief and iterate on version history with natural-language feedback.
- Drag reference images, PDFs, or text files into the initial brief or Studio chat; attached files are passed only to the selected model for that generation.
- Connect an already authenticated **Codex CLI** or **Claude CLI** installation.
- Connect any OpenAI-compatible endpoint or Anthropic API with a BYOK key.
- Disable creation until one provider is explicitly active.
- Export each version as SVG or PNG locally.
- Apply the bundled `Poster Art Direction` skill before every generation: it extracts public facts, reads references, sets hierarchy, and rejects generic or technically polluted poster copy.
- Run on macOS, Windows, and Linux through Tauri 2.

OpenStudio does not ship demo projects or simulated AI responses. A connected and active provider is required before a visual can be generated.

## Architecture

The application is deliberately divided by responsibility:

```text
src/                         React presentation + application client
  domain/                    Pure client domain types and visual helpers
  application/               Typed Tauri command client
src-tauri/src/
  domain/                    Visual domain model and SVG rendering
  application/               Tauri use-case commands
  infrastructure/            SQLite repository, credential vault, AI adapters
```

The Rust layer owns persistence, secret access, provider execution, output validation, and exports. The frontend never receives stored API keys. All provider adapters are behind one generation use case, so additional local or hosted providers can be added without changing UI flows.

## Prerequisites

- Node.js 20 or newer
- Rust stable via [rustup](https://rustup.rs)
- Tauri platform prerequisites for your OS: [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)

Optional local provider integrations:

- `codex` installed and authenticated for Codex CLI
- `claude` installed and authenticated for Claude CLI

## Development

```bash
npm install
npm run tauri dev
```

On first launch, open the settings button (or `Cmd/Ctrl + K`) and add a provider. Local CLI providers are detected from `PATH`; remote providers require a model identifier and a key. The key is put in the system credential store, not in SQLite or a project file.

## Checks

```bash
npm run build
npm test
cd src-tauri && cargo check
```

## Release builds

```bash
npm run tauri build
```

Tauri generates native installers for the host platform. Build on each target platform (or use a CI matrix) to publish macOS, Windows, and Linux artifacts.

## Provider protocol

Generation asks the chosen model for a constrained JSON visual specification. OpenStudio validates it, stores the immutable version, and renders the result locally into SVG or PNG. This approach keeps the visual editable and makes exports deterministic.

Reference attachments are limited to eight files per request and 10 MB per file. Image inputs are sent as vision content to compatible OpenAI-style and Anthropic endpoints. For local Codex and Claude CLIs, OpenStudio writes temporary local reference files for the command and removes them once the generation ends.

The bundled production skill is versioned in [`skills/poster-art-direction`](skills/poster-art-direction). It is compiled into the local application and sent to the selected model during both the research and final art-direction stages; it is not a server-side prompt.

For an OpenAI-compatible connection, the endpoint must implement `POST /chat/completions`. The endpoint defaults to `https://api.openai.com/v1` when left empty. For Anthropic, the endpoint defaults to `https://api.anthropic.com/v1/messages`.

## Privacy and security

- No application backend is used.
- Brand kits, briefs, and version history remain in the local app-data SQLite database.
- API keys use the native credential vault through `keyring` (Keychain on macOS, Credential Manager on Windows, the configured secret service on Linux).
- Briefs are transmitted only to the provider you activate. This is inherent to using an external model.

## License

MIT. See [LICENSE](LICENSE).
