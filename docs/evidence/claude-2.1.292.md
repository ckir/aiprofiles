custody: ci
run-id: 37542704292
run-url: https://github.com/ckir/aiprofiles/actions/runs/37542704292
harness-commit: 37a4c54476f3b00ada2d061500c4d7896835cb1b
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
  2.1.292 (Claude Code)
version-extracted: 2.1.292
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
                                          filtering (e.g., "api,hooks" or
                                          "!1p,!file")
    --debug-file <path>                   Write debug logs to a specific file path
                                          (implicitly enables debug mode)
    --desktop                             Open in the Claude Desktop app instead
                                          of the terminal (with --continue or
                                          --resume <id> to pick the session)
    --disable-slash-commands              Disable all skills
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
                                          validation. Example:
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
                                          PowerShell, REPL and the other
                                          code-running tools) and WebFetch unless
                                          --tools names them, and ignores user,
                                          project and local settings files
                                          (managed settings and --settings still
    [truncated: more than 200 lines]
strings:
  # /home/agent/.local/bin/claude -> /home/agent/.local/lib/node_modules/@anthropic-ai/claude-code/bin/claude.exe
  # scanned 11 file(s), 503102003 byte(s) under /home/agent/.local/lib/node_modules/@anthropic-ai/claude-code
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
  ACCOUNT_CONSENT_KEYS
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
  ALLUSERSPROFILE
  AMPERSAND_TOKEN
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
  AUTH_HEADER
  AUTH_HEADER_REJECTED
  AUTH_METHOD
  AUTO_REPLIES_ARMED_TOKEN
  AWS_ACCESS_KEY_ID
  AWS_ACCESS_KEY_IDS3_SECRET_ACCESS_KEYAWS_SECRET_ACCESS_KEYS3_REGIONAWS_REGIONS3_ENDPOINTAWS_ENDPOINTS3_BUCKETAWS_BUCKETAWS_SESSION_TOKEN
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
  BUN_CONFIG_MAX_HTTP_REQUESTS
  BUN_CONFIG_MAX_HTTP_REQUESTSBUN_CONFIG_MAX_HTTP_REQUESTS
  BUN_CONFIG_REGISTRY
  BUN_CONFIG_SKIP_INSTALL_PACKAGESERR_MYSQL_INVALID_ENCODED_LENGTH
  BUN_CONFIG_TOKEN
  BUN_CONFIG_VERBOSE_FETCH
  BUN_CONFIG_WS_HANDSHAKE_TIMEOUT
  BUN_CONFIG_YARN_LOCKFILEBUN_CONFIG_HTTP_RETRY_COUNTBUN_CONFIG_SKIP_SAVE_LOCKFILEBUN_CONFIG_SKIP_LOAD_LOCKFILEBUN_CONFIG_NO_VERIFYI
  BUN_FEATURE_FLAG_DISABLE_ADDRCONFIG
  BUN_GC_TIMER_INTERVALBUN_CONFIG_HTTP_IDLE_TIMEOUTBUN_INOTIFY_COALESCE_INTERVALBUN_INSTALL_STREAMING_MIN_SIZEBUN_CONFIG_WS_HANDSHAKE_TIMEOUTBUN_CONFIG_DNS_TIME_TO_LIVE_SECONDSBUN_GC_RUNS_UNTIL_SKIP_RELEASE_ACCESSBUN_INSTALL_STREAMING_DRAIN_THRESHOLDCOLUMNS
  BUN_INSTALL_GLOBAL_STOREBUN_INSTALL_PROGRESSBUN_CONFIG_REGISTRYNPM_CONFIG_REGISTRY
  BUN_WORKER_MESSAGING_KEY
  BUN_WORKER_PARENT_PORT_KEY
  BUN_WORKER_STDIO_KEY
  BZMPOPCONFIGF
  CACHED_ACCESS_TOKEN_EXPIRED
  CACHE_KEY
  CACHE_KEY_SEPARATOR
  CANNOT_HAVE_BOTH_PRIVKEY_AND_METHOD
  CANNOT_RECOVER_MULTI_PRIME_KEY
  CANT_CHECK_DH_KEY
  CARGO_HOME
  CARGO_REGISTRY_TOKEN
  CATALOG_ID_TO_KEY
  CCR_AGENT_PROXY_TOKEN_FILE_DESCRIPTOR
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
  CLAUDE_CODE_CLIENT_KEY
  CLAUDE_CODE_CLIENT_KEY_PASSPHRASE
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
  CLAUDE_CODE_HOME_SEED_HOLD_TIMEOUT_MS
  CLAUDE_CODE_HOME_SEED_VERDICT_TIMEOUT_MS
  CLAUDE_CODE_HOST_AUTH_ENV_VAR
  CLAUDE_CODE_HOST_AUTH_REFRESH_TIMEOUT_MS
  CLAUDE_CODE_IDLE_COMPACT_MIN_TOKENS
  CLAUDE_CODE_IDLE_TOKEN_THRESHOLD
  CLAUDE_CODE_MANAGED_CONFIG_PREFETCH
  CLAUDE_CODE_MANAGED_SETTINGS_PATH
    [truncated: more than 200 lines]
baseline-env-CLAUDE_CONFIG_DIR:
  # /home/agent/probe-target
  d /home/agent/probe-target
  # /home/agent/.claude
  (absent)
delta-env-CLAUDE_CONFIG_DIR:
  + d /home/agent/probe-target/backups
  + f /home/agent/probe-target/.claude.json
  + f /home/agent/probe-target/backups/.claude.json.backup.1791326873483
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
  A /home/agent/.npm/_cacache/content-v2/sha512/07
  A /home/agent/.npm/_cacache/content-v2/sha512/07/8f
  A /home/agent/.npm/_cacache/content-v2/sha512/07/8f/257b6b596c341d4beeab918c979e3ff5ae9aeea61dcd087a7159d8c1734dc290c970fa17e26db9cc2b5a53d01d29283cd485bca31a89bce26095ed556902
  A /home/agent/.npm/_cacache/content-v2/sha512/0c
  A /home/agent/.npm/_cacache/content-v2/sha512/0c/d4
  A /home/agent/.npm/_cacache/content-v2/sha512/0c/d4/2b259a264005f6de39095e77d8b48c897527f90819dec57c0eda10010acee8899a5adc86cf588a3b2ba48f939b73448111cdbc297caded9a28dad42efe91
  A /home/agent/.npm/_cacache/content-v2/sha512/1e
  A /home/agent/.npm/_cacache/content-v2/sha512/1e/b7
  A /home/agent/.npm/_cacache/content-v2/sha512/1e/b7/114ce0f68459ba5778ccb9ca781de81ece9937520b0fb82af78550562e74fee592085d4c7c5acc53bc7e9a2e52229b75853e7b50388899a249bef27231c5
  A /home/agent/.npm/_cacache/content-v2/sha512/42
  A /home/agent/.npm/_cacache/content-v2/sha512/42/6c
  A /home/agent/.npm/_cacache/content-v2/sha512/42/6c/7c95a4c80f10cb79d3da6ccba69f0f4dcd643c1ded03b5256a34a1bb062483f29dbc4e1ff5306333a6f2abb133dc1d1b740ad0725cec242857d6622d80a1
  A /home/agent/.npm/_cacache/content-v2/sha512/57
  A /home/agent/.npm/_cacache/content-v2/sha512/57/c2
  A /home/agent/.npm/_cacache/content-v2/sha512/57/c2/e20cb4cc5d5765b1a95adff78fdb2fcd786e2b2019e58788f3cdd20fc78bc203098d78c478206c73aa4e1034c93dc7d1d3d25f7100ed1f479dc887f7df1c
  A /home/agent/.npm/_cacache/content-v2/sha512/95
  A /home/agent/.npm/_cacache/content-v2/sha512/95/fb
  A /home/agent/.npm/_cacache/content-v2/sha512/95/fb/25e5dcb304c239d1507929203c6c5b1bbe2a9e033a34caff1fe455be0b424cbd32f7ce57d32c4be792cb6151ce4cf5166c9bf4ab890b4f26fade29f037ab
  A /home/agent/.npm/_cacache/content-v2/sha512/9a
  A /home/agent/.npm/_cacache/content-v2/sha512/9a/63
  A /home/agent/.npm/_cacache/content-v2/sha512/9a/63/c3f08e2968c93d4bebd03413e5c34e3326cc2734aeed4c9513118b89ebcef233a3769482168969cb4492f2c9f7fe7e49dcae503a9b36230d7f0ef8ef9d8a
  A /home/agent/.npm/_cacache/content-v2/sha512/a6
  A /home/agent/.npm/_cacache/content-v2/sha512/a6/d4
  A /home/agent/.npm/_cacache/content-v2/sha512/a6/d4/fd51c3acc91d952cd81f1e017ced3a3c07bb42ad992d229602e1d106b7ef4c1b8d8d016b258c4181677b29747709d9a91e767b3311254db62ee15b166b26
  A /home/agent/.npm/_cacache/content-v2/sha512/be
  A /home/agent/.npm/_cacache/content-v2/sha512/be/bc
  A /home/agent/.npm/_cacache/content-v2/sha512/be/bc/04b2dbbaef4c6b6b6924caad9514b0e1bbb88dcbca3441303a8f1fa55acd3c7a1f809d7a9715f951a573dd474f88f49636026774f3026735c6df8bc48945
  A /home/agent/.npm/_cacache/content-v2/sha512/c0
  A /home/agent/.npm/_cacache/content-v2/sha512/c0/e7
  A /home/agent/.npm/_cacache/content-v2/sha512/c0/e7/dbc8b37495f8824201c9aad82bb50b6c004a8e1f668a9abe9dd61bd5c416c2dad43e942d5324bd723ce34be24c25d25e57efad0e931f5c3d8be8d84f6431
  A /home/agent/.npm/_cacache/content-v2/sha512/ce
  A /home/agent/.npm/_cacache/content-v2/sha512/ce/58
  A /home/agent/.npm/_cacache/content-v2/sha512/ce/58/50f23655c7fc682192cadfe1aed0748e245bd652b6d57079916a237a2b4c4dd9d68e7f68b4e7dfd1d438c1f22deb3f508a84544b3e1c0aee9e86c08ce29f
  A /home/agent/.npm/_cacache/index-v5
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
  A /home/agent/.npm/_cacache/index-v5/4c
  A /home/agent/.npm/_cacache/index-v5/4c/00
  A /home/agent/.npm/_cacache/index-v5/4c/00/1f1f7a91e32aaf607526c6d6f8d921533477ca45cfb1935b9aa86fd90c58
  A /home/agent/.npm/_cacache/index-v5/82
  A /home/agent/.npm/_cacache/index-v5/82/9a
  A /home/agent/.npm/_cacache/index-v5/82/9a/9dcc9bf7a0e36536bb95629820ae36b62e6711735332256da3a94795cd29
  A /home/agent/.npm/_cacache/index-v5/bb
  A /home/agent/.npm/_cacache/index-v5/bb/03
  A /home/agent/.npm/_cacache/index-v5/bb/03/6dca5c405b9351bc441003bc074a3ad39db203e253da77ad8e22167152dc
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
  A /home/agent/.npm/_logs/2026-10-06T22_47_45_367Z-debug-0.log
  A /home/agent/.npm/_update-notifier-last-checked
  A /home/agent/probe-target
  A /home/agent/probe-target/.claude.json
  A /home/agent/probe-target/backups
  A /home/agent/probe-target/backups/.claude.json.backup.1791326873483
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
