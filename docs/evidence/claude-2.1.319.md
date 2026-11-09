custody: ci
run-id: 42654070717
run-url: https://github.com/ckir/aiprofiles/actions/runs/42654070717
harness-commit: 96c66274332c4054bab8794f8715ccc2e33604e1
---
install:
  npm install --global @anthropic-ai/claude-code
exit-codes:
  install 0
  version 0
  help 0
  behaviour-env-CLAUDE_CONFIG_DIR 0
probe-exit: 0
version:
  2.1.319 (Claude Code)
version-extracted: 2.1.319
help:
  Usage: claude [options] [command] [prompt]
  
  Claude Code - starts an interactive session by default, use -p/--print for
  non-interactive output
  
  Arguments:
    prompt                                Your prompt
  
  Options:
    --add-dir <directories...>            Additional directories to allow tool
                                          access to
    --agent <agent>                       Agent for the current session. Overrides
                                          the 'agent' setting.
    --agents <json-or-file>               JSON object defining custom agents, or
                                          with --print the path to a file that
                                          holds one (e.g. '{"reviewer":
                                          {"description": "Reviews code",
                                          "prompt": "You are a code reviewer"}}')
    --allow-dangerously-skip-permissions  Enable bypassing all permission checks
                                          as an option, without it being enabled
                                          by default. Recommended only for
                                          sandboxes with no internet access.
    --allowedTools, --allowed-tools <tools...>
        Comma or space-separated list of tool names to allow (e.g. "Bash(git *)
        Edit")
    --append-system-prompt <prompt>       Append a system prompt to the default
                                          system prompt
    --autocompact <auto|tokens>           Auto-compact window size (auto, or
                                          100k–1M tokens)
    --ax-screen-reader                    Render screen-reader friendly output
                                          (flat text, no decorative borders or
                                          animations).
    --bg, --background                    Start the session in the background and
                                          return immediately. Prints the id that
                                          `claude attach`, `logs`, `stop` and `rm`
                                          take; `claude agents` lists them. With
                                          --resume <session-id>, continues that
                                          session in the background under the same
                                          ID, or starts a copy and says so when
                                          the session is already running
    --bare                                Minimal mode: skip hooks (those defined
                                          in settings and by installed plugins;
                                          features built into Claude Code are
                                          unaffected), LSP, plugin sync,
                                          attribution, auto-memory, background
                                          prefetches, keychain reads, and
                                          CLAUDE.md auto-discovery. Sets
                                          CLAUDE_CODE_SIMPLE=1. Anthropic auth is
                                          strictly ANTHROPIC_API_KEY or
                                          apiKeyHelper via --settings (OAuth and
                                          keychain are never read). 3P providers
                                          (Bedrock/Vertex/Foundry) use their own
                                          credentials. Skills still resolve via
                                          /skill-name. Explicitly provide context
                                          via: --system-prompt[-file],
                                          --append-system-prompt[-file], --add-dir
                                          (CLAUDE.md dirs), --mcp-config,
                                          --settings, --agents, --plugin-dir.
    --betas <betas...>                    Beta headers to include in API requests
                                          (API key users only)
    --brief                               Enable SendUserMessage tool for
                                          agent-to-user communication
    --chrome                              Enable Claude in Chrome integration
    --cloud [description|session_id|url]  Create a cloud session with the given
                                          description, or attach to an existing
                                          one by session ID or claude.ai/code URL
    -c, --continue                        Continue the most recent conversation in
                                          the current directory
    --dangerously-skip-permissions        Bypass all permission checks.
                                          Recommended only for sandboxes with no
                                          internet access.
    -d, --debug [filter]                  Enable debug mode with optional category
                                          filtering (e.g., --debug=hooks,mcp or
                                          --debug='!hooks,!mcp')
    --debug-file <path>                   Write debug logs to a specific file path
                                          (implicitly enables debug mode)
    --desktop                             Open in the Claude Desktop app instead
                                          of the terminal (with --continue or
                                          --resume <id> to pick the session)
    --disable-slash-commands              Disable all skills and commands
    --disallowedTools, --disallowed-tools <tools...>
        Comma or space-separated list of tool names to deny (e.g. "Bash(git *)
        Edit")
    --effort <level>                      Effort level for the current session
                                          (low, medium, high, xhigh, max)
    --environment <environment_id>        Create a new cloud session that runs on
                                          the given self-hosted environment
                                          (ccpool_...).
    --exclude-dynamic-system-prompt-sections
        Move per-machine sections (cwd, env info, memory paths, git status) from
        the system prompt into the first user message. Improves cross-user
        prompt-cache reuse. Only applies with the default system prompt (ignored
        with --system-prompt). (default: false)
    --fallback-model <model>              Enable automatic fallback to specified
                                          model(s) when the default model is
                                          overloaded or not available. Accepts a
                                          comma-separated list to try each in
                                          order. Re-tries the primary at the start
                                          of each user turn.
    --file <specs...>                     File resources to download at startup.
                                          Format: file_id:relative_path (e.g.,
                                          --file file_abc:doc.txt
                                          file_def:img.png)
    --fork-session                        When resuming, create a new session ID
                                          instead of reusing the original (use
                                          with --resume or --continue)
    --forward-subagent-text               Forward subagent text and thinking
                                          blocks as assistant/user messages with
                                          parent_tool_use_id set (only works with
                                          --print and --output-format=stream-json)
    --from-pr [value]                     Resume a session linked to a PR by PR
                                          number/URL, or open interactive picker
                                          with optional search term
    -h, --help                            Display help for command
    --ide                                 Automatically connect to IDE on startup
                                          if exactly one valid IDE is available
    --include-hook-events                 Include all hook lifecycle events in the
                                          output stream (only works with
                                          --output-format=stream-json)
    --include-partial-messages            Include partial message chunks as they
                                          arrive (only works with --print and
                                          --output-format=stream-json)
    --input-format <format>               Input format (only works with --print):
                                          "text" (default), or "stream-json"
                                          (realtime streaming input) (choices:
                                          "text", "stream-json")
    --json-schema <schema>                JSON Schema for structured output
                                          validation (only works with --print).
                                          Example:
                                          {"type":"object","properties":{"name":{"type":"string"}},"required":["name"]}
    --max-budget-usd <amount>             Maximum dollar amount to spend on API
                                          calls (only works with --print)
    --mcp-config <configs...>             Load MCP servers from JSON files or
                                          strings (space-separated)
    --model <model>                       Model for the current session. Provide
                                          an alias for the latest model (e.g.
                                          'fable', 'opus', or 'sonnet') or a
                                          model's full name.
    -n, --name <name>                     Set a display name for this session
                                          (shown in the prompt box, /resume
                                          picker, and terminal title)
    --no-chrome                           Disable Claude in Chrome integration
    --no-session-persistence              Disable session persistence - sessions
                                          will not be saved to disk and cannot be
                                          resumed (only works with --print)
    --output-format <format>              Output format (only works with --print):
                                          "text" (default), "json" (single
                                          result), or "stream-json" (realtime
                                          streaming) (choices: "text", "json",
                                          "stream-json")
    --permission-mode <mode>              Permission mode to use for the session
                                          (choices: "acceptEdits", "auto",
                                          "bypassPermissions", "manual",
                                          "dontAsk", "plan")
    --permission-prompts <target>         Who answers permission prompts with
                                          --print: "host" (the SDK host or
                                          --permission-prompt-tool) or "none"
                                          (nobody: anything that would prompt is
                                          denied automatically; the permission
                                          mode still decides everything else)
                                          (choices: "host", "none", default:
                                          "host")
    --plugin-dir <path>                   Load a plugin from a directory or .zip
                                          for this session only; a folder of
                                          plugins loads each child (repeatable:
                                          --plugin-dir A --plugin-dir B.zip)
                                          (default: [])
    --plugin-url <url>                    Fetch a plugin .zip from a URL for this
                                          session only (repeatable: --plugin-url A
                                          --plugin-url B) (default: [])
    -p, --print                           Print response and exit (useful for
                                          pipes). Note: The workspace trust dialog
                                          is skipped when Claude is run in
                                          non-interactive mode (via -p, or when
                                          stdout is not a TTY, e.g. piped or
                                          redirected output). Only use this in
                                          directories you trust. Settings files
                                          that fail validation are silently
                                          ignored in this mode (no error dialog is
                                          shown).
    --prompt-suggestions [value]          Enable prompt suggestions. In print/SDK
                                          mode, emits a prompt_suggestion message
                                          after each turn with a predicted next
                                          user prompt (choices: "true", "false",
                                          "1", "0", "yes", "no", "on", "off",
                                          preset: "true")
    --remote-control [name]               Start an interactive session with Remote
                                          Control enabled (optionally named)
    --remote-control-session-name-prefix <prefix>
        Prefix for auto-generated Remote Control session names (default: hostname)
    --replay-user-messages                Re-emit user messages from stdin back on
                                          stdout for acknowledgment (only works
                                          with --input-format=stream-json and
                                          --output-format=stream-json)
    --restricted                          Restricted mode: removes the built-in
                                          tools that run commands or code (Bash,
                                          PowerShell and the other code-running
                                          tools) and WebFetch unless --tools names
                                          them, and ignores user, project and
                                          local settings files (managed settings
    [truncated: more than 200 lines]
strings:
  # /home/agent/.local/bin/claude -> /home/agent/.local/lib/node_modules/@anthropic-ai/claude-code/bin/claude.exe
  # scanned 11 file(s), 680655625 byte(s) under /home/agent/.local/lib/node_modules/@anthropic-ai/claude-code
  documented ANTHROPIC_API_KEY: present
  documented ANTHROPIC_AUTH_TOKEN: present
  documented ANTHROPIC_PROFILE: present
  documented CLAUDE_CODE_OAUTH_TOKEN: present
  documented CLAUDE_CODE_OAUTH_REFRESH_TOKEN: present
  documented CLAUDE_CODE_USE_BEDROCK: present
  documented CLAUDE_CODE_USE_VERTEX: present
  documented CLAUDE_CODE_USE_FOUNDRY: present
  #
  # variable-shaped tokens that name a credential or a location, documented or not
  ABORT_ERRERR_ACCESS_DENIEDERR_AMBIGUOUS_ARGUMENTERR_ARG_NOT_ITERABLEERR_ASSERTIONERR_ASYNC_CALLBACKERR_ASYNC_TYPEERR_BODY_ALREADY_USEDERR_BORINGSSLERR_BROTLI_INVALID_PARAMERR_BUFFER_OUT_OF_BOUNDSERR_BUFFER_TOO_LARGEERR_CHILD_PROCESS_IPC_REQUIREDERR_CHILD_PROCESS_STDIO_MAXBUFFERERR_CLOSED_MESSAGE_PORTERR_CONSOLE_WRITABLE_STREAMERR_CONSTRUCT_CALL_INVALIDERR_CONSTRUCT_CALL_REQUIREDERR_CRYPTO_CUSTOM_ENGINE_NOT_SUPPORTEDERR_CRYPTO_ECDH_INVALID_FORMATERR_CRYPTO_ECDH_INVALID_PUBLIC_KEYERR_CRYPTO_HASH_F  [line truncated]
  ACCESSKEY
  ACCESS_KEY_ID
  ACCESS_TOKEN
  ACCESS_TOKEN_WITH_AUTH_SCHEME
  ACCOUNTKEY
  ACTIONS_ID_TOKEN_REQUEST_TOKEN
  ACTIONS_ID_TOKEN_REQUEST_URL
  ACTIONS_RUNTIME_TOKEN
  ADDRCONFIG
  AES_KEY_SETUP_FAILED
  AFTER_DOCTYPE_PUBLIC_KEYWORD
  AFTER_DOCTYPE_SYSTEM_KEYWORD
  AGENT_PROXY_AUTH_TOKEN
  AGENT_SPAWN_KEPT_KEYS
  AGENT_SPAWN_RESTORED_KEYS
  AGENT_TASK_META_KEY
  AGENT_VIEW_RELAUNCH_ENV_KEY
  ALLOWED_OAUTH_BASE_URLS
  ALLOW_LARGE_UNCOUNTED_BYTES_PER_TOKEN
  ALLUSERSPROFILE
  ANACONDA_API_TOKEN
  ANDROID_HOME
  ANSIBLE_CONFIG
  ANSIBLE_VAULT_PASSWORD_FILE
  ANTHROPIC_API_KEY
  ANTHROPIC_AUTH_TOKEN
  ANTHROPIC_AWS_API_KEY
  ANTHROPIC_CONFIG_DIR
  ANTHROPIC_ENVIRONMENT_KEY
  ANTHROPIC_FOUNDRY_API_KEY
  ANTHROPIC_FOUNDRY_AUTH_TOKEN
  ANTHROPIC_IDENTITY_TOKEN
  ANTHROPIC_IDENTITY_TOKEN_FILE
  ANTHROPIC_PROFILE
  ANTHROPIC_SECRET_PLACEHOLDER_
  ANTHROPIC_WEBHOOK_SIGNING_KEY
  ANTHROPIC_WORK_SECRET
  ANY_KEY__
  APIKEY
  API_KEY
  API_KEY_URL
  API_KEY_WITH_CREDENTIALS
  APP_SERVICE_SECRET_HEADER_NAME
  ARTIFACTS_CREDENTIALPROVIDER_ACCESSTOKEN
  ARTIFACTS_CREDENTIALPROVIDER_EXTERNAL_FEED_ENDPOINTS
  ASDF_DATA_DIR
  ATTR_ASPNETCORE_USER_IS_AUTHENTICATED
  AUTH
  AUTH0_
  AUTHCODE_EXPIRED
  AUTHKEY
  AUTHORED
  AUTHORITY
  AUTHORITY1
  AUTHORITY_INFO_ACCESS
  AUTHORITY_KEYID
  AUTHORIZATION
  AUTHORIZATION_CODE_GRANT
  AUTHORIZATION_HEADER_NAME
  AUTHORIZATION_PENDING
  AUTHORIZED
  AUTHORS
  AUTHOR_AGENT_TYPE
  AUTH_HEADER
  AUTH_HEADER_REJECTED
  AUTH_METHOD
  AUTO_REPLIES_ARMED_TOKEN
  AWS_ACCESS_KEY_ID
  AWS_AUTH_SCHEME_PREFERENCE
  AWS_BEARER_TOKEN_
  AWS_BEARER_TOKEN_BEDROCK
  AWS_CONFIG_FILE
  AWS_CONTAINER_AUTHORIZATION_TOKEN
  AWS_CONTAINER_AUTHORIZATION_TOKEN_FILE
  AWS_CONTAINER_CREDENTIALS_FULL_URI
  AWS_CONTAINER_CREDENTIALS_RELATIVE_URI
  AWS_CREDENTIAL_EXPIRATION
  AWS_CREDENTIAL_SCOPE
  AWS_DEFAULT_PROFILE
  AWS_PROFILE
  AWS_SECRET_ACCESS_KEY
  AWS_SESSION_TOKEN
  AWS_SHARED_CREDENTIALS_FILE
  AWS_WEB_IDENTITY_TOKEN_FILE
  AZURE_AUTHORITY_HOST
  AZURE_AUTH_LOCATION
  AZURE_CLIENT_CERTIFICATE_PASSWORD
  AZURE_CLIENT_SECRET
  AZURE_CONFIG_DIR
  AZURE_FEDERATED_TOKEN_FILE
  AZURE_IDENTITY_DISABLE_MULTITENANTAUTH
  AZURE_PASSWORD
  AZURE_POD_IDENTITY_AUTHORITY_HOST
  AZURE_REGIONAL_AUTHORITY_NAME
  AZURE_TOKEN_CREDENTIALS
  BACKGROUND_SETTINGS_WORK_WAIT_MS
  BAD_KEY_LENGTH
  BAD_PASSWORD_READ
  BAD_SRTP_PROTECTION_PROFILE_LIST
  BAGGAGE_KEY_PAIR_SEPARATOR
  BCONFIGURATION
  BINSTAR_API_TOKEN
  BOTO_CONFIG
  BRIPGREP_CONFIG_PATH
  BUNDLE_APP_CONFIG
  BUNDLE_CONFIG
  BUNDLE_USER_CONFIG
  BUNDLE_USER_HOME
  BUN_CONFIG_DISABLE_
  BUN_CONFIG_DISABLE_COPY_FILE_RANGE
  BUN_CONFIG_DNS_TIME_TO_LIVE_SECONDS
  BUN_CONFIG_FILE
  BUN_CONFIG_HTTP_IDLE_TIMEOUT
  BUN_CONFIG_HTTP_RETRY_BACKOFF
  BUN_CONFIG_MAX_HTTP_REQUESTS
  BUN_CONFIG_MAX_HTTP_REQUESTSBUN_CONFIG_MAX_HTTP_REQUESTS
  BUN_CONFIG_REGISTRY
  BUN_CONFIG_SKIP_INSTALL_PACKAGES
  BUN_CONFIG_TOKENNPM_CONFIG_TOKEN
  BUN_CONFIG_VERBOSE_FETCH
  BUN_CONFIG_WS_CLOSE_TIMEOUT
  BUN_CONFIG_WS_HANDSHAKE_TIMEOUT
  BUN_CONFIG_YARN_LOCKFILEBUN_CONFIG_HTTP_RETRY_COUNTBUN_CONFIG_SKIP_SAVE_LOCKFILEBUN_CONFIG_SKIP_LOAD_LOCKFILEBUN_CONFIG_NO_VERIFY
  BUN_FEATURE_FLAG_DISABLE_ADDRCONFIG
  BUN_MANIFEST_CACHEBUN_INSTALL_GLOBAL_STOREBUN_INSTALL_PROGRESSBUN_CONFIG_REGISTRYNPM_CONFIG_REGISTRY
  BUN_TLS_EXTRA_DISTRUSTED_KEYS
  BUN_TLS_REFUSE_DISTRUSTED_KEYS
  BUN_WORKER_MESSAGING_KEY
  BUN_WORKER_PARENT_PORT_KEY
  BUN_WORKER_STDIO_KEY
  BZMPOPCONFIGF
  CACHED_ACCESS_TOKEN_EXPIRED
  CACHE_KEY
  CACHE_KEY_SEPARATOR
  CALLER_ASKS_KEY
  CANNOT_HAVE_BOTH_PRIVKEY_AND_METHOD
  CANNOT_RECOVER_MULTI_PRIME_KEY
  CANT_CHECK_DH_KEY
  CARGO_HOME
  CARGO_REGISTRY_TOKEN
  CATALOG_ID_TO_KEY
  CCR_AGENT_PROXY_TOKEN_FILE_DESCRIPTOR
  CCR_CREDENTIAL_REFUSED_PREFIX
  CCR_OAUTH_TOKEN_FILE
  CCR_SESSION_PROFILE
  CDNSKEY
  CERTIFICATE_AND_PRIVATE_KEY_MISMATCH
  CERTIFICATE_CONFIGURATION_ENV_VARIABLE
  CERTIFICATE_HAS_NO_KEYID
  CHANNEL_ARGS_CONFIG_SELECTOR_KEY
  CHOICE_TOKEN
  CIAM_AUTH_URL
  CLASSIC_ENVELOPE_KEYS
  CLAUDE_AI_AUTHORIZE_URL
  CLAUDE_AI_PROFILE_SCOPE
  CLAUDE_API_KEY
  CLAUDE_BG_AUTH_SNAPSHOT_PATH
  CLAUDE_BG_CLAIM_AUTH
  CLAUDE_BG_PTY_AUTH
  CLAUDE_BG_RV_AUTH
  CLAUDE_BG_SOCKET_TOKENS_PATH
  CLAUDE_BRIDGE_OAUTH_TOKEN
  CLAUDE_CHROME_TAB_GROUP_KEY
  CLAUDE_CODE_AGENT_PROXY_GIT_CONFIG
  CLAUDE_CODE_API_KEY_FILE_DESCRIPTOR
  CLAUDE_CODE_API_KEY_HELPER_TTL_MS
  CLAUDE_CODE_ARG_KEY_SHAPE
  CLAUDE_CODE_ARTIFACTS_API_TOKEN
  CLAUDE_CODE_AUTH_FAIL_EXIT_MS
  CLAUDE_CODE_BRIDGE_CHILD_MACHINE_SETTINGS
  CLAUDE_CODE_CCR_EARLY_SYNC_AFTER_SETTINGS_REFUSAL
  CLAUDE_CODE_CLIENT_KEY
  CLAUDE_CODE_CLIENT_KEY_PASSPHRASE
  CLAUDE_CODE_CONFIG_LOCK_WAIT_MS
  CLAUDE_CODE_CONFIG_PROBE
  CLAUDE_CODE_CONFIG_WATCH_EVENTS
  CLAUDE_CODE_CUSTOM_OAUTH_URL
  CLAUDE_CODE_DESIGN_OAUTH_CLIENT_ID
  CLAUDE_CODE_DISABLE_AUTH_REFRESH_LOCK
  CLAUDE_CODE_DISABLE_HOME_SETTINGS_SEED
  CLAUDE_CODE_ENABLE_PROXY_AUTH_HELPER
  CLAUDE_CODE_ENABLE_TOKEN_USAGE_ATTACHMENT
  CLAUDE_CODE_FILE_READ_MAX_OUTPUT_TOKENS
  CLAUDE_CODE_GATEWAY_TOKEN
  CLAUDE_CODE_GATEWAY_TOKEN_FILE_DESCRIPTOR
  CLAUDE_CODE_HFI_BEARER_TOKEN
  CLAUDE_CODE_HIDE_SETTINGS_HINT
    [truncated: more than 200 lines]
baseline-env-CLAUDE_CONFIG_DIR:
  # /home/agent/probe-target
  d /home/agent/probe-target
  # /home/agent/.claude
  (absent)
delta-env-CLAUDE_CONFIG_DIR:
  + d /home/agent/probe-target/backups
  + f /home/agent/probe-target/.claude.json
  + f /home/agent/probe-target/backups/.claude.json.backup.1794241423206
container-delta:
  C /etc
  C /home/agent
  C /home
  A /home/agent/.local
  A /home/agent/.local/bin
  A /home/agent/.local/bin/claude
  A /home/agent/.local/lib
  A /home/agent/.local/lib/node_modules
  A /home/agent/.local/lib/node_modules/@anthropic-ai
  A /home/agent/.local/lib/node_modules/@anthropic-ai/claude-code
  A /home/agent/.local/lib/node_modules/@anthropic-ai/claude-code/LICENSE.md
  A /home/agent/.local/lib/node_modules/@anthropic-ai/claude-code/README.md
  A /home/agent/.local/lib/node_modules/@anthropic-ai/claude-code/bin
  A /home/agent/.local/lib/node_modules/@anthropic-ai/claude-code/bin/claude.exe
  A /home/agent/.local/lib/node_modules/@anthropic-ai/claude-code/cli-wrapper.cjs
  A /home/agent/.local/lib/node_modules/@anthropic-ai/claude-code/install.cjs
  A /home/agent/.local/lib/node_modules/@anthropic-ai/claude-code/node_modules
  A /home/agent/.local/lib/node_modules/@anthropic-ai/claude-code/node_modules/@anthropic-ai
  A /home/agent/.local/lib/node_modules/@anthropic-ai/claude-code/node_modules/@anthropic-ai/claude-code-linux-x64
  A /home/agent/.local/lib/node_modules/@anthropic-ai/claude-code/node_modules/@anthropic-ai/claude-code-linux-x64/LICENSE.md
  A /home/agent/.local/lib/node_modules/@anthropic-ai/claude-code/node_modules/@anthropic-ai/claude-code-linux-x64/README.md
  A /home/agent/.local/lib/node_modules/@anthropic-ai/claude-code/node_modules/@anthropic-ai/claude-code-linux-x64/claude
  A /home/agent/.local/lib/node_modules/@anthropic-ai/claude-code/node_modules/@anthropic-ai/claude-code-linux-x64/package.json
  A /home/agent/.local/lib/node_modules/@anthropic-ai/claude-code/package.json
  A /home/agent/.local/lib/node_modules/@anthropic-ai/claude-code/sdk-tools.d.ts
  A /home/agent/.npm
  A /home/agent/.npm/_cacache
  A /home/agent/.npm/_cacache/content-v2
  A /home/agent/.npm/_cacache/content-v2/sha512
  A /home/agent/.npm/_cacache/content-v2/sha512/0c
  A /home/agent/.npm/_cacache/content-v2/sha512/0c/d4
  A /home/agent/.npm/_cacache/content-v2/sha512/0c/d4/63f0b9ffc8000b34ea8e1765696d8d7fa67665e1beeff204ad35f7155deda962ebee5b4864762ac011463d2b1921117f8854db8f447272f7e2027b701408
  A /home/agent/.npm/_cacache/content-v2/sha512/14
  A /home/agent/.npm/_cacache/content-v2/sha512/14/ee
  A /home/agent/.npm/_cacache/content-v2/sha512/14/ee/12776dc13b554240f8897b9bc6ddb367995e7e6224dabbba3596a2b716be55944e132dd2a2c98a70411c045cef75904da5d76129da2ec9d7d4fc31dda839
  A /home/agent/.npm/_cacache/content-v2/sha512/36
  A /home/agent/.npm/_cacache/content-v2/sha512/36/d4
  A /home/agent/.npm/_cacache/content-v2/sha512/36/d4/d5aa565a4ccdec118dace04c5333af033a5bd31917399ec3deaafb5fafe8666ddc28e988c986fc604a4c3970b6f3b3a47bdf5275419bd912160cb56b0e74
  A /home/agent/.npm/_cacache/content-v2/sha512/3d
  A /home/agent/.npm/_cacache/content-v2/sha512/3d/b6
  A /home/agent/.npm/_cacache/content-v2/sha512/3d/b6/276d275b4002cff85a3f0376036a86e481ce7d22b7ace2cfaa44771b54928399643146223828b86ad127efdac3dc4adef45c2f501274504b6ace53c8aab4
  A /home/agent/.npm/_cacache/content-v2/sha512/42
  A /home/agent/.npm/_cacache/content-v2/sha512/42/1d
  A /home/agent/.npm/_cacache/content-v2/sha512/42/1d/cc38d2cbf3fd2c3decc10e8b7ab07fdb3553d580559f8f26bdeec71b7b78feb8e128e6a34e83829f3ccdc473da7dcd36609f5313960f63fa7a9b4d399a3a
  A /home/agent/.npm/_cacache/content-v2/sha512/51
  A /home/agent/.npm/_cacache/content-v2/sha512/51/f8
  A /home/agent/.npm/_cacache/content-v2/sha512/51/f8/c6f5be788458d92036cdc2ba2bd74677b8f16c1fa4753eab3bb3fae5e5faad37d46d8a2dd3fd8bbc57178f8664d06b54e637010f199c3749836bbe0d1ebb
  A /home/agent/.npm/_cacache/content-v2/sha512/5e
  A /home/agent/.npm/_cacache/content-v2/sha512/5e/8c
  A /home/agent/.npm/_cacache/content-v2/sha512/5e/8c/80c465c198db1bf5a873d312c0b72cff23a6f58fee25c8ba6e7324d022ae0570a531acc0ccd80cc0dac1e2a47a0dd4374c24835042b2d9c51651abb16a04
  A /home/agent/.npm/_cacache/content-v2/sha512/6b
  A /home/agent/.npm/_cacache/content-v2/sha512/6b/f2
  A /home/agent/.npm/_cacache/content-v2/sha512/6b/f2/cc0d4c1539b254644783bf954bcba6a9c420a06c68faba6d067ad258a06ebcd94f2caeb7955c37e82bf235f4f32b628160a6f3d134187950b5da9ee124b8
  A /home/agent/.npm/_cacache/content-v2/sha512/99
  A /home/agent/.npm/_cacache/content-v2/sha512/99/d2
  A /home/agent/.npm/_cacache/content-v2/sha512/99/d2/43b903baa8fb15d0ed6d6ac9bc564b929ee8cd85ee6edf3cd07885c0d4d0206c6857c17d4bfb02773f46faecd808b9b4f98f53bdcd44d481637fda1d47fc
  A /home/agent/.npm/_cacache/content-v2/sha512/aa
  A /home/agent/.npm/_cacache/content-v2/sha512/aa/54
  A /home/agent/.npm/_cacache/content-v2/sha512/aa/54/11829ccbe0141abd5f6d354dd66cb5f50eb745675d676d0a7b4e06fef5202b5f79bfa7c52197a34a644cc4016a0477f0500d82152dd3ddf0ebcdf2e1e13c
  A /home/agent/.npm/_cacache/content-v2/sha512/d3
  A /home/agent/.npm/_cacache/content-v2/sha512/d3/9f
  A /home/agent/.npm/_cacache/content-v2/sha512/d3/9f/51f859b4291aa538ab6491f69d0ba9da08a49997d464d4c65bc9c4f83a8573f9a325da23f082918fe475ee84b7a7804e6b6d5af933b0ee26af9a53daafdd
  A /home/agent/.npm/_cacache/index-v5
  A /home/agent/.npm/_cacache/index-v5/04
  A /home/agent/.npm/_cacache/index-v5/04/cf
  A /home/agent/.npm/_cacache/index-v5/04/cf/ab6be81b75178b1499eaf137ef7e335a8571ad69a4c018a5195353bb2216
  A /home/agent/.npm/_cacache/index-v5/23
  A /home/agent/.npm/_cacache/index-v5/23/c2
  A /home/agent/.npm/_cacache/index-v5/23/c2/0358876565a484d45a113827b0878c26f9e8a2569e81cee788b21977aef8
  A /home/agent/.npm/_cacache/index-v5/36
  A /home/agent/.npm/_cacache/index-v5/36/9e
  A /home/agent/.npm/_cacache/index-v5/36/9e/d27f0d4326a432a7f70ab880544a3b1f4a3d1412ed67ae05058ae14b6b60
  A /home/agent/.npm/_cacache/index-v5/38
  A /home/agent/.npm/_cacache/index-v5/38/13
  A /home/agent/.npm/_cacache/index-v5/38/13/18cb5bd6e40d8f3c82c478c7bfa1b55a623eada6df35945b88ce43fe042b
  A /home/agent/.npm/_cacache/index-v5/39
  A /home/agent/.npm/_cacache/index-v5/39/64
  A /home/agent/.npm/_cacache/index-v5/39/64/9759f919932497d2ba4bb7ef94e1f3d0a7b7745146e365bba5cbb761b17b
  A /home/agent/.npm/_cacache/index-v5/41
  A /home/agent/.npm/_cacache/index-v5/41/c5
  A /home/agent/.npm/_cacache/index-v5/41/c5/4270bf1cd1aae004ed6fee83989ac428601f4c060987660e9a1aef9d53b6
  A /home/agent/.npm/_cacache/index-v5/69
  A /home/agent/.npm/_cacache/index-v5/69/f3
  A /home/agent/.npm/_cacache/index-v5/69/f3/473ffeb668b6b174ab242dc308bc99769064ab93586dc5a0cd14d92bd8ae
  A /home/agent/.npm/_cacache/index-v5/82
  A /home/agent/.npm/_cacache/index-v5/82/9a
  A /home/agent/.npm/_cacache/index-v5/82/9a/9dcc9bf7a0e36536bb95629820ae36b62e6711735332256da3a94795cd29
  A /home/agent/.npm/_cacache/index-v5/d3
  A /home/agent/.npm/_cacache/index-v5/d3/a4
  A /home/agent/.npm/_cacache/index-v5/d3/a4/217d21c59484f4edc85af5f0549d026c15388bad78d9ad85ac9a729297bc
  A /home/agent/.npm/_cacache/index-v5/da
  A /home/agent/.npm/_cacache/index-v5/da/df
  A /home/agent/.npm/_cacache/index-v5/da/df/8711f609ada78ac76e1101bc69f44309aa1e518485327ac90d5ed829ab89
  A /home/agent/.npm/_cacache/index-v5/e2
  A /home/agent/.npm/_cacache/index-v5/e2/64
  A /home/agent/.npm/_cacache/index-v5/e2/64/7a1d3a9a8e90ecace65e69e3b1423f117fc2325bf365c21a7d2e90f053f9
  A /home/agent/.npm/_cacache/tmp
  A /home/agent/.npm/_logs
  A /home/agent/.npm/_logs/2026-11-09T16_23_37_209Z-debug-0.log
  A /home/agent/.npm/_update-notifier-last-checked
  A /home/agent/probe-target
  A /home/agent/probe-target/.claude.json
  A /home/agent/probe-target/backups
  A /home/agent/probe-target/backups/.claude.json.backup.1794241423206
  C /home/probe
  A /home/probe/.probe-out
  A /home/probe/.probe-out/after-env-CLAUDE_CONFIG_DIR.txt
  A /home/probe/.probe-out/baseline-env-CLAUDE_CONFIG_DIR.txt
  A /home/probe/.probe-out/behaviour-env-CLAUDE_CONFIG_DIR.cmd
  A /home/probe/.probe-out/behaviour-env-CLAUDE_CONFIG_DIR.exit-code
  A /home/probe/.probe-out/behaviour-env-CLAUDE_CONFIG_DIR.txt
  A /home/probe/.probe-out/delta-env-CLAUDE_CONFIG_DIR.txt
  A /home/probe/.probe-out/help.cmd
  A /home/probe/.probe-out/help.exit-code
  A /home/probe/.probe-out/help.txt
  A /home/probe/.probe-out/install.cmd
  A /home/probe/.probe-out/install.exit-code
  A /home/probe/.probe-out/install.txt
  A /home/probe/.probe-out/strings.txt
  A /home/probe/.probe-out/version.cmd
  A /home/probe/.probe-out/version.exit-code
  A /home/probe/.probe-out/version.extracted
  A /home/probe/.probe-out/version.txt
  A /home/probe/.probe-state
  A /home/probe/.probe-state/failures
  A /home/probe/.probe-state/pristine
  A /home/probe/.probe-state/pristine/1.absent
  A /home/probe/.probe-state/pristine/index
  A /home/probe/.probe-state/steps
  A /src
  C /tmp
  A /tmp/node-compile-cache
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/00bf0630
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/00e7fb07
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/01c08d40
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/0275a71a
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/027fd99b
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/029c3683
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/03bfe4fd
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/04e65cde
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/052c232d
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/0535d717
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/053fed39
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/0555bc32
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/077049fd
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/09587c8c
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/09a7ebaf
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/09cc6bf3
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/0abe949a
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/0b00cc65
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/0bb0939e
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/0c15d042
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/0c92995d
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/0cb643b3
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/0d738cfc
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/0da616be
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/0e32632a
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/0eba5d40
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/0f16a34a
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/0f96f96d
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/1002e6f0
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/107ab572
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/108868d6
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/10f112e9
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/1134cedf
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/121174e4
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/1215b633
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/129c9ca9
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/12eb0ec0
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/13581cd4
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/138d8b8a
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/14003ba0
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/1400e130
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/14321a91
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/1464282f
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/161e91c9
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/165e65c4
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/16c0288e
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/16d89fbf
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/17ec4ef0
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/193e2733
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/19c64358
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/1a11b592
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/1a717684
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/1ae914af
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/1b711ba2
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/1c4e1590
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/1d0a3031
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/1e2f7071
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/1e6ddd85
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/1eb6ec67
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/1f2f0e9a
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/20b81e57
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/20cadba9
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/21c61b6a
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/23971331
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/23976ea8
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/2476a3c1
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/2512a231
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/2548310c
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/262de1a0
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/2669a8be
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/26851a55
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/27893977
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/279cf8d9
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/28f6ca10
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/29564910
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/2afb5617
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/2b47d793
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/2b72d99e
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/2b80aaeb
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/2bb97a42
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/2c1bd91e
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/2d189c75
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/2d333828
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/2d8bd46c
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/2ec2cb17
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/2ee203d7
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/2f8c9e26
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/2fc52dda
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/30838e72
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/30909de6
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/312d1339
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/314a1792
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/31503801
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/31720429
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/31af2baa
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/31fcef26
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/32d1c5bc
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/33573f62
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/34d66bd2
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/35a067fc
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/36354dfa
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/36de03eb
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/36e97a38
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/36f68b81
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/37bac5d5
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/37c36750
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/37e3f297
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/39ce3f5c
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/39e8eb68
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/3ade6c40
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/3bf3c906
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/3c96d784
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/3ca02643
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/3f8c2a21
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/40194e3b
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/41707149
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/42a36d27
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/42a5d001
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/42bbb12f
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/43ce6b52
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/442c1750
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/45615b69
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/4564e2da
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/4565d742
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/46d40b27
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/471d2ae5
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/473c8e1f
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/47ad38b1
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/47b4b1f8
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/47bfaea9
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/47f4c8eb
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/481d9a18
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/49486cf1
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/49631807
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/49b91afd
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/49d0b900
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/4a909031
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/4b6a72dd
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/4bb798db
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/4bda5c6a
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/4cf262e6
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/4d348b0d
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/4d5538af
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/4d653578
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/4ddaf35d
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/4ddbfffe
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/4e2640f8
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/4e34a80b
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/4e3a9b9a
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/4f18cd52
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/4f26fa49
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/502c76a8
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/504a17c6
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/5092240e
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/50be787f
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/515ea996
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/5167ddd4
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/517aeb01
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/52299478
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/52380b07
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/5244a2f6
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/527f5bd3
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/52839e32
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/52f4092a
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/5301bd2c
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/557c0551
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/55cf3abc
    [truncated: more than 300 lines]
