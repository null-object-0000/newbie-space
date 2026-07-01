# LLM Gateway Agent Compatibility Roadmap

This document tracks what the desktop LLM API gateway needs before it can be used reliably by mainstream AI agents such as Claude Code, Codex, OpenClaw, Hermes, and similar local or self-hosted agent runtimes.

## Current State

The gateway currently provides a local OpenAI-compatible Chat Completions proxy:

- `GET /health`
- `GET /v1/models`
- `POST /v1/chat/completions`
- `POST /v1/responses`
- `POST /v1/messages`
- `POST /v1/messages/count_tokens`

It already supports:

- Multiple upstream channels.
- API key pools with round-robin or random selection.
- Public model to upstream model mapping.
- Route priority based fallback.
- Non-streaming and SSE streaming Chat Completions forwarding.
- Minimal OpenAI Responses API compatibility for text requests by translating Responses requests to Chat Completions upstreams and wrapping results back into Responses-shaped output.
- Minimal Anthropic Messages compatibility for text requests by translating `/v1/messages` requests to Chat Completions upstreams and wrapping results back into Anthropic-shaped messages.
- Basic tool-call history bridging between Responses function calls, Anthropic `tool_use` / `tool_result` blocks, and OpenAI Chat Completions `tool_calls` / `tool` messages.
- Local estimated Anthropic token counting for `/v1/messages/count_tokens`.
- Basic request logs, token usage, estimated cost, and usage breakdowns by public model, channel, and API key.
- Detailed response output recording for non-streaming responses and accumulated text for streaming responses.
- Persistent local JSONL request logs that survive desktop app restarts.
- Per-request fallback attempt logs with channel, key, upstream model, status, latency, error, and upstream request id when the provider returns one.
- Local gateway request ids are propagated to upstream providers through `x-request-id` and `x-gateway-request-id`.
- Config profiles and S3 sync.
- Loopback-only listen host validation.
- Optional local inbound gateway token validation.
- UI-generated connection snippets for Claude Code, Codex, and OpenAI-compatible clients based on the current listen address, public model, and local gateway token setting.

This is enough for simple OpenAI-compatible clients, but not enough for agent-grade compatibility.

## Target Compatibility

The gateway should become a local provider surface that agents can point at without custom patches:

- Claude Code via `ANTHROPIC_BASE_URL`.
- Codex via a custom `model_provider` with `base_url` and `wire_api = "responses"`.
- OpenAI SDKs, OpenAI Agents SDK, and compatible runtimes via `/v1/responses` or `/v1/chat/completions`.
- OpenClaw, Hermes, and other agent frameworks via OpenAI-compatible and Anthropic-compatible endpoints.

## Missing Capabilities

### 1. OpenAI Responses API

Codex and newer OpenAI agent stacks are centered on the Responses API. The gateway now has a minimal `POST /v1/responses` bridge for text requests, but full agent-grade support still needs:

- `GET /v1/responses/{response_id}`
- `DELETE /v1/responses/{response_id}`
- `GET /v1/responses/{response_id}/input_items`
- `POST /v1/responses/{response_id}/cancel`

The `POST /v1/responses` bridge now handles basic function-call history and function tool schema normalization, but still needs broader coverage for hosted tools, previous response state, richer multimodal content, and full Responses streaming event parity.

### 2. Anthropic Messages API

Claude Code can route through an LLM gateway with `ANTHROPIC_BASE_URL`, but it expects Anthropic-shaped requests, responses, headers, and streaming events. The gateway now has a minimal `POST /v1/messages` bridge for text requests and local estimated token counting, but full support still needs:

- Anthropic request headers such as `x-api-key`, `anthropic-version`, and `anthropic-beta`.
- Stronger Anthropic SSE event parity, including tool use deltas, accurate streaming usage, and edge-case event ordering.
- Optional exact token counting via upstream tokenizer/API instead of local character estimation.

The current bridge translates text, simple tool schemas, `tool_use`, and `tool_result` blocks to the internal chat route, then translates OpenAI-style responses back to Anthropic Messages. Full Claude Code behavior still needs compatibility testing.

### 3. Tool Calling Conversion

The gateway now has a first-pass conversion among:

- OpenAI Chat Completions `tools`, `tool_choice`, and `tool_calls`.
- OpenAI Responses input/output items.
- Anthropic `tools`, `tool_use`, and `tool_result` content blocks.

This still needs compatibility hardening for streaming tool deltas, malformed arguments, parallel tool-call edge cases, hosted tools, and exact client behavior in Claude Code, Codex, OpenClaw, and Hermes.

### 4. Model Capability Metadata

The route model currently stores names, channel, priority, and pricing only. Agents need to know or configure whether a model supports:

- Tool calls.
- Parallel tool calls.
- Structured output.
- Vision and file input.
- Reasoning or thinking parameters.
- Prompt caching.
- Streaming.
- Context window and output token limits.

This should become explicit route or model metadata instead of implicit knowledge in the user.

### 5. Inbound Gateway Authentication

The server is loopback-only and now has optional local token validation. Agent traffic may include source code, terminal output, credentials, and private files, so this should be expanded beyond the first baseline:

- Per-client labels and optional per-client keys.
- Request body size limits.
- Optional allowlist checks for origin, user agent, or client name.

### 6. Agent-Grade Observability

The gateway now writes local JSONL request logs and records a basic fallback chain per request. Long-running agent operations still need:

- Per-client filtering and deeper per-model, per-channel, and per-key drill-down.
- Stream interruption and retry diagnostics.
- Redaction controls for prompts, tool arguments, and file paths.

### 7. Smarter Routing And Fallback

Current fallback is mostly priority order. Add:

- Retry and fallback decisions by status code and error type.
- Channel and key circuit breakers.
- Per-key rate limits and cooldowns.
- Real weighted round-robin using the existing `weight` field.
- Sticky routing by conversation/session where useful.
- Policy options such as fastest, cheapest, primary-first, or capability-first.

## Priority Plan

### P0: Make Mainstream Agents Connect

- Harden `/v1/responses` beyond the current text bridge.
- Harden `/v1/messages` beyond the current text bridge.
- Add request/response conversion for basic text, system messages, multimodal placeholders, and streaming text.
- Keep local inbound gateway token support enabled in Agent presets.
- Expand UI snippets into import/export presets for Claude Code, Codex, OpenClaw, Hermes, and common SDK conventions.

### P1: Make Agent Runs Reliable

- Add tool call conversion across OpenAI Chat, OpenAI Responses, and Anthropic Messages.
- Harden persistent logs with indexing, filtering, and retention settings.
- Expand fallback attempt, upstream request ID, and stream failure diagnostics into searchable UI views.
- Add key and channel health state.
- Add real weighted key selection.

### P2: Make It A Real Agent Gateway

- Add model capability metadata and validation.
- Add per-client quota and cost views.
- Add prompt cache, reasoning token, and tool usage accounting.
- Add import/export presets for Claude Code, Codex, OpenClaw, Hermes, Ollama, LM Studio, and common OpenAI-compatible providers.
- Add a compatibility test suite with mock upstreams.
- Keep protocol conversion unit tests for Responses function calls, Anthropic tool use/results, and Chat tool-call output shapes.

## Suggested Code Shape

Split the current large gateway module into protocol and routing components:

```text
src-tauri/src/llm_gateway/
  mod.rs
  config.rs
  routing.rs
  logs.rs
  http.rs
  protocol/
    openai_chat.rs
    openai_responses.rs
    anthropic_messages.rs
```

The internal routing layer should operate on a normalized request/response shape. Protocol modules should only handle wire-format conversion.

## Acceptance Checks

P0 is done when these checks pass against a local mock upstream:

- `GET /v1/models` returns configured public models.
- `POST /v1/chat/completions` works for non-streaming and streaming text.
- `POST /v1/responses` works for non-streaming and streaming text.
- `POST /v1/messages` works for non-streaming and streaming text.
- `POST /v1/messages/count_tokens` returns `input_tokens`.
- Request logs include prompt, completion, and total token counts when upstream usage is available.
- Usage summaries show token and cost breakdowns by model, channel, and key.
- Request details preserve the model response output text for both non-streaming and streaming requests.
- Claude Code can be configured with `ANTHROPIC_BASE_URL=http://127.0.0.1:<port>` and complete a simple prompt.
- Codex can be configured with a custom provider using `base_url = "http://127.0.0.1:<port>/v1"` and `wire_api = "responses"` and complete a simple prompt.
- Requests without a valid local gateway token are rejected when inbound auth is enabled.
- Unit tests cover basic Responses and Anthropic tool-call history conversion.
- The UI shows copyable connection snippets for Claude Code, Codex, and OpenAI-compatible clients.

P1 is done when:

- Tool calls round-trip through all supported protocol surfaces.
- Route fallback behavior is visible in logs.
- Upstream request IDs are captured in request details when providers return them.
- Persistent JSONL logs survive desktop app restart.
- A failed key or channel can be cooled down without taking down the whole gateway.
