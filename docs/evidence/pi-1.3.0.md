custody: ci
run-id: 39024332187
run-url: https://github.com/ckir/aiprofiles/actions/runs/39024332187
harness-commit: 16737cb79a0cde5bb698fae7784e69b9eeb6b551
---
install:
  npm install --global --ignore-scripts @earendil-works/pi-coding-agent
exit-codes:
  install 0
  version 0
  help 0
  behaviour-env-PI_CODING_AGENT_DIR 1
probe-exit: 0
version:
  1.3.0
version-extracted: 1.3.0
help:
  pi - AI coding assistant with read, bash, edit, write tools
  
  Usage:
    pi [options] [--] [@files...] [messages...]
  
  Commands:
    pi install <source> [-l]     Install extension source and add to settings
    pi remove <source> [-l]      Remove extension source from settings
    pi uninstall <source> [-l]   Alias for remove
    pi update [source|self|pi]   Update pi, extensions, or model catalogs
    pi list                      List installed extensions from settings
    pi config [-l]               Open TUI to enable/disable package resources (Tab switches scope)
    pi auth <command>            Print credentials or check provider readiness
    pi mcp <command>             Check MCP servers, sign in to or out of OAuth servers
    pi <command> --help          Show help for install/remove/uninstall/update/list/config/auth/mcp
  
  Options:
    --provider <name>              Provider to search for --model (requires --model)
    --model <pattern>              Model pattern or ID (supports "provider/id" and optional ":<thinking>")
    --api-key <key>                API key (defaults to env vars)
    --system-prompt <text>         System prompt (default: coding assistant prompt)
    --append-system-prompt <text>  Append text or file contents to the system prompt (can be used multiple times)
    --mode <mode>                  Output mode: text (default), json, or rpc
    --print, -p                    Non-interactive mode: process prompt and exit
    --continue, -c                 Continue previous session
    --resume, -r                   Select a session to resume
    --session <path|id>            Use specific session file or partial UUID
    --session-id <id>              Use exact project session ID, creating it if missing
    --fork <path|id>               Fork specific session file or partial UUID into a new session
    --session-dir <dir>            Directory for session storage and lookup
    --no-session                   Don't save session (ephemeral)
    --name, -n <name>              Set session display name
    --models <patterns>            Comma-separated model patterns for Ctrl+P cycling
                                   Supports globs (anthropic/*, *sonnet*) and fuzzy matching
    --no-tools, -nt                Disable all tools by default (built-in and extension)
    --no-builtin-tools, -nbt       Disable built-in tools by default but keep extension/custom tools enabled
    --tools, -t <tools>            Comma-separated allowlist of tool names or patterns (*) to enable
                                   Keeps MCP tools unless an entry starts with mcp__
                                   Only +name/-name entries add to or remove from the defaults
    --exclude-tools, -xt <tools>   Comma-separated denylist of tool names or patterns (*) to disable
                                   Applies to all tools, MCP tools included
    --thinking <level>             Set thinking level: off, minimal, low, medium, high, xhigh, max
    --extension, -e <path>         Load an extension file or builtin:<name> (can be used multiple times)
    --no-extensions, -ne           Disable extension discovery and built-in extensions (explicit -e paths still work)
    --no-mcp                       Disable built-in MCP support: no servers connect and no MCP tools
    --skill <path>                 Load a skill file or directory (can be used multiple times)
    --no-skills, -ns               Disable skills discovery and loading
    --prompt-template <path>       Load a prompt template file or directory (can be used multiple times)
    --no-prompt-templates, -np     Disable prompt template discovery and loading
    --theme <path>                 Load a theme file or directory (can be used multiple times)
    --use-theme <name[/name]>      Set the initial interactive theme for this run
    --no-themes                    Disable theme discovery and loading
    --no-context-files, -nc        Disable AGENTS.md and CLAUDE.md discovery and loading
    --export <file>                Export session file to HTML and exit
    --list-models [search]         List available models (with optional fuzzy search)
    --verbose                      Force verbose startup (overrides quietStartup setting)
    --tui-mode <mode>              TUI mode: fullscreen (default) or regular
    --approve, -a                  Trust project-local files for this run
    --no-approve, -na              Ignore project-local files for this run
    --offline                      Disable startup network operations (same as PI_OFFLINE=1)
    --                             End option parsing; treat remaining arguments as messages/files
    --help, -h                     Show this help
    --version, -v                  Show version number
  
  Extensions can register additional flags (e.g., --plan from plan-mode extension).
  
  Examples:
    # Print a provider API key for an external client
    pi auth print-api-key --provider openai
  
    # Print an OAuth bearer token for an external client (refreshes if expired)
    pi auth print-bearer-token --provider openai-codex
  
    # Interactive mode
    pi
  
    # Interactive mode with initial prompt
    pi "List all .ts files in src/"
  
    # Include files in initial message
    pi @prompt.md @image.png "What color is the sky?"
  
    # Non-interactive mode (process and exit)
    pi -p "List all .ts files in src/"
  
    # Prompt beginning with a dash
    pi -p -- "- Summarize these points"
  
    # Multiple messages (interactive)
    pi "Read package.json" "What dependencies do we have?"
  
    # Continue previous session
    pi --continue "What did we discuss?"
  
    # Start a named session
    pi --name "Refactor auth module"
  
    # Use different model
    pi --provider openai --model gpt-4o-mini "Help me refactor this code"
  
    # Use model with provider prefix (no --provider needed)
    pi --model openai/gpt-4o "Help me refactor this code"
  
    # Use model with thinking level shorthand
    pi --model sonnet:high "Solve this complex problem"
  
    # Limit model cycling to specific models
    pi --models claude-sonnet,claude-haiku,gpt-4o
  
    # Limit to a specific provider with glob pattern
    pi --models "github-copilot/*"
  
    # Cycle models with fixed thinking levels
    pi --models sonnet:high,haiku:low
  
    # Start with a specific thinking level
    pi --thinking high "Solve this complex problem"
  
    # Read-only mode (no file modifications possible)
    pi --tools read,grep,find,ls -p "Review the code in src/"
  
    # Add codemode to the default tools
    pi --tools +codemode
  
    # Codemode with only the tools of one MCP server
    pi --tools read,bash,codemode,'mcp__radius__*'
  
    # Disable one tool while keeping the rest available
    pi --exclude-tools ask_question
  
    # Export a session file to HTML
    pi --export ~/.pi/agent/sessions/--path--/session.jsonl
    pi --export session.jsonl output.html
  
  Environment Variables:
    ANTHROPIC_AUTH_TOKEN             - Anthropic bearer auth token
    ANTHROPIC_API_KEY                - Anthropic Claude API key
    ANTHROPIC_OAUTH_TOKEN            - Anthropic OAuth token (alternative to API key)
    ANT_LING_API_KEY                 - Ant Ling API key
    OPENAI_API_KEY                   - OpenAI GPT API key
    AZURE_OPENAI_API_KEY             - Azure OpenAI API key
    AZURE_OPENAI_BASE_URL            - Azure OpenAI/Cognitive Services base URL (e.g. https://{resource}.openai.azure.com)
    AZURE_OPENAI_RESOURCE_NAME       - Azure OpenAI resource name (alternative to base URL)
    AZURE_OPENAI_API_VERSION         - Azure OpenAI API version (default: v1)
    AZURE_OPENAI_DEPLOYMENT_NAME_MAP - Azure OpenAI model=deployment map (comma-separated)
    DEEPSEEK_API_KEY                 - DeepSeek API key
    NVIDIA_API_KEY                   - NVIDIA NIM API key
    GEMINI_API_KEY                   - Google Gemini API key
    GROQ_API_KEY                     - Groq API key
    CEREBRAS_API_KEY                 - Cerebras API key
    XAI_API_KEY                      - xAI Grok API key
    FIREWORKS_API_KEY                - Fireworks API key
    TOGETHER_API_KEY                 - Together AI API key
    BASETEN_API_KEY                  - Baseten API key
    OPENROUTER_API_KEY               - OpenRouter API key
    AI_GATEWAY_API_KEY               - Vercel AI Gateway API key
    ZAI_API_KEY                      - ZAI Coding Plan API key (Global)
    ZAI_CODING_CN_API_KEY            - ZAI Coding Plan API key (China)
    MISTRAL_API_KEY                  - Mistral API key
    MINIMAX_API_KEY                  - MiniMax API key
    MOONSHOT_API_KEY                 - Moonshot AI API key
    OPENCODE_API_KEY                 - OpenCode Zen/OpenCode Go API key
    KIMI_API_KEY                     - Kimi For Coding API key
    META_API_KEY                     - Meta Model API key
    CLOUDFLARE_API_KEY               - Cloudflare API token (Workers AI and AI Gateway)
    CLOUDFLARE_ACCOUNT_ID            - Cloudflare account id (required for both)
    CLOUDFLARE_GATEWAY_ID            - Cloudflare AI Gateway slug (required for AI Gateway)
    QWEN_TOKEN_PLAN_API_KEY          - Qwen Token Plan API key (international region)
    QWEN_TOKEN_PLAN_CN_API_KEY       - Qwen Token Plan API key (China region)
    XIAOMI_API_KEY                   - Xiaomi MiMo API key (api.xiaomimimo.com billing)
    XIAOMI_TOKEN_PLAN_CN_API_KEY     - Xiaomi MiMo Token Plan API key (China region)
    XIAOMI_TOKEN_PLAN_AMS_API_KEY    - Xiaomi MiMo Token Plan API key (Amsterdam region)
    XIAOMI_TOKEN_PLAN_SGP_API_KEY    - Xiaomi MiMo Token Plan API key (Singapore region)
    AWS_PROFILE                      - AWS profile for Amazon Bedrock
    AWS_ACCESS_KEY_ID                - AWS access key for Amazon Bedrock
    AWS_SECRET_ACCESS_KEY            - AWS secret key for Amazon Bedrock
    AWS_BEARER_TOKEN_BEDROCK         - Bedrock API key (bearer token)
    AWS_REGION                       - AWS region for Amazon Bedrock (e.g., us-east-1)
    PI_CODING_AGENT_DIR              - Config directory (default: ~/.pi/agent)
    PI_CODING_AGENT_SESSION_DIR      - Session storage directory (overridden by --session-dir)
    PI_PACKAGE_DIR                   - Override package directory (for Nix/Guix store paths)
    PI_OFFLINE                       - Disable startup network operations when set to 1/true/yes
    PI_TELEMETRY                     - Override install telemetry when set to 1/true/yes or 0/false/no
    PI_SHARE_VIEWER_URL              - Base URL for /share command (default: https://pi.dev/session/)
  
  Built-in Tool Names:
    read       - Read file contents
    bash       - Execute bash commands
    powershell - Execute PowerShell commands on Windows
    edit       - Edit files with find/replace
    write      - Write files (creates/overwrites)
    grep       - Search file contents (read-only, off by default)
    find       - Find files by glob pattern (read-only, off by default)
    ls         - List directory contents (read-only, off by default)
  
strings:
  # /home/agent/.local/bin/pi -> /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/cli.js
  # scanned 15026 file(s), 111516642 byte(s) under /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent
  documented ANTHROPIC_API_KEY: present
  documented OPENAI_API_KEY: present
  documented PI_CODING_AGENT_DIR: present
  #
  # variable-shaped tokens that name a credential or a location, documented or not
  ACCESS_TOKEN_TYPE
  ADDRCONFIG
  AGENT_CURSOR_HOME_COL
  AGENT_CURSOR_HOME_ROW
  AI_GATEWAY_API_KEY
  ALIAS_KEYS
  AMBIENT_AUTH_MARKER
  ANTHROPIC_API_KEY
  ANTHROPIC_API_KEY_ENV
  ANTHROPIC_AUTH_TOKEN
  ANTHROPIC_AUTH_TOKEN_ENV
  ANTHROPIC_CONFIG_DIR
  ANTHROPIC_ENVIRONMENT_KEY
  ANTHROPIC_IDENTITY_TOKEN
  ANTHROPIC_IDENTITY_TOKEN_FILE
  ANTHROPIC_IDENTITY_TOKEN_FILE_ENV
  ANTHROPIC_OAUTH_TOKEN
  ANTHROPIC_OAUTH_TOKEN_ENV
  ANTHROPIC_PROFILE
  ANTHROPIC_STRICT_UNSUPPORTED_KEYWORDS
  ANTHROPIC_SUBSCRIPTION_AUTH_WARNING
  ANTHROPIC_WEBHOOK_SIGNING_KEY
  ANTHROPIC_WORK_SECRET
  ANT_LING_API_KEY
  ANY_KEY
  API_KEY
  API_KEY_AUTH
  API_KEY_LIFETIME_MS
  API_KEY_MINT_URL
  API_KEY_WITH_CREDENTIALS
  APP_KEYBINDINGS
  AT_KEYWORDS
  AUTH
  AUTHCODE_EXPIRED
  AUTHENTICATED
  AUTHENTICATING
  AUTHOR
  AUTHORIZATION
  AUTHORIZED
  AUTHORIZE_URL
  AUTHORS
  AUTH_BASE_URL
  AUTH_COMMAND_USAGE
  AUTH_FILE
  AUTH_FILE_WRITE_OPTIONS
  AUTH_HEADER
  AUTH_HOST
  AUTH_METHODS
  AUTH_TYPE_ORDER
  AUTH_TYPE_UNSPECIFIED
  AWS_ACCESS_KEY
  AWS_ACCESS_KEY_ID
  AWS_AUTH_SCHEME_PREFERENCE
  AWS_BEARER_TOKEN_
  AWS_BEARER_TOKEN_BEDROCK
  AWS_BEDROCK_SKIP_AUTH
  AWS_CONFIG_FILE
  AWS_CONTAINER_AUTHORIZATION_TOKEN
  AWS_CONTAINER_AUTHORIZATION_TOKEN_FILE
  AWS_CONTAINER_CREDENTIALS_
  AWS_CONTAINER_CREDENTIALS_FULL_URI
  AWS_CONTAINER_CREDENTIALS_RELATIVE_URI
  AWS_CREDENTIAL_EXPIRATION
  AWS_CREDENTIAL_SCOPE
  AWS_DEFAULTS_MODE_CONFIG
  AWS_IGNORE_CONFIGURED_ENDPOINT_URLS
  AWS_PROFILE
  AWS_SECRET_ACCESS_KEY
  AWS_SECRET_KEY
  AWS_SESSION_TOKEN
  AWS_SHARED_CREDENTIALS_FILE
  AWS_WEB_IDENTITY_TOKEN_FILE
  AZURE_OPENAI_API_KEY
  BABEL_SHOW_CONFIG_FOR
  BARE_KEY
  BASETEN_API_KEY
  BETA_CLIENT_TOOL_UNION_KEYS
  BKEY
  BLOCK_AS_IMPLICIT_KEY
  BUILDER_KEYS
  BUILT_IN_KEYWORDS
  CDNSKEY
  CEREBRAS_API_KEY
  CERTIFICATE_CONFIGURATION_ENV_VARIABLE
  CHARS_PER_TOKEN
  CLASS_KEYWORDS
  CLIENT_SECRET
  CLOUDFLARE_API_KEY
  CLOUDFLARE_GATEWAY_BINDING_AUTH_SENTINEL
  CLOUDSDK_CONFIG
  CLOUDSDK_CONFIG_DIRECTORY
  COFFEE_KEYWORDS
  COMMENT_KEYS
  COMMON_KEYWORDS
  COMPACT_THRESHOLD_TOKENS
  COMPONENT_TOKEN_LINK_KIND
  COMPONENT_TOKEN_LOCK_TYPE
  CONFIG
  CONFIGADD
  CONFIGGET
  CONFIGURATION
  CONFIGURATION_LIMIT_EXCEEDED
  CONFIGURATION_UPDATE_REQUEST_EFFORT
  CONFIG_ACCOUNT_ID_ENDPOINT_MODE
  CONFIG_COMMAND_USAGE
  CONFIG_DIR_NAME
  CONFIG_DISABLE_CLOCK_SKEW_CORRECTION
  CONFIG_ENDPOINT_MODE_NAME
  CONFIG_ENDPOINT_NAME
  CONFIG_ENDPOINT_URL
  CONFIG_FILE_ERROR
  CONFIG_FILE_VERSION
  CONFIG_IGNORE_CONFIGURED_ENDPOINT_URLS
  CONFIG_MAX_ATTEMPTS
  CONFIG_PREFIX_SEPARATOR
  CONFIG_RETRY_MODE
  CONFIG_USE_DUALSTACK_ENDPOINT
  CONFIG_USE_FIPS_ENDPOINT
  CONNECTION_TOKEN_CHARS
  CONTEXTUAL_KEYWORDS
  CONTEXT_SAFETY_TOKENS
  COPILOT_GITHUB_TOKEN
  COS_KEYWORDS
  CPP_KEYWORDS
  CREDENTIALS_AWS_SDK_STORE
  CREDENTIALS_BOTO2_CONFIG_FILE
  CREDENTIALS_CODE
  CREDENTIALS_ENV_VARS
  CREDENTIALS_ENV_VARS_STS_WEB_ID_TOKEN
  CREDENTIALS_FILE_VERSION
  CREDENTIALS_HTTP
  CREDENTIALS_IMDS
  CREDENTIALS_LOGIN
  CREDENTIALS_PROCESS
  CREDENTIALS_PROFILE
  CREDENTIALS_PROFILE_LOGIN
  CREDENTIALS_PROFILE_NAMED_PROVIDER
  CREDENTIALS_PROFILE_PROCESS
  CREDENTIALS_PROFILE_SOURCE_PROFILE
  CREDENTIALS_PROFILE_SSO
  CREDENTIALS_PROFILE_SSO_LEGACY
  CREDENTIALS_PROFILE_STS_WEB_ID_TOKEN
  CREDENTIALS_SSO
  CREDENTIALS_SSO_LEGACY
  CREDENTIALS_STS_ASSUME_ROLE
  CREDENTIALS_STS_ASSUME_ROLE_SAML
  CREDENTIALS_STS_ASSUME_ROLE_WEB_ID
  CREDENTIALS_STS_FEDERATION_TOKEN
  CREDENTIALS_STS_SESSION_TOKEN
  CREDENTIAL_QUERY_PARAM
  CRYSTAL_KEYWORDS
  CUSTOM_ANTHROPIC_API_KEY
  DEBUG_AUTH
  DEEPSEEK_API_KEY
  DEFAULT_AWS_REGIONAL_CREDENTIAL_VERIFICATION_URL
  DEFAULT_BEARER_TOKEN_MIN_EXPIRY_MS
  DEFAULT_COMPACTION_SETTINGS
  DEFAULT_COMPACTION_TOKEN_SETTINGS
  DEFAULT_CONFIG
  DEFAULT_KEYWORD_CLASSNAME
  DEFAULT_MAX_OUTPUT_TOKENS
  DEFAULT_MAX_TOKEN_LIFETIME_SECS_
  DEFAULT_MAX_TOKEN_SIZE
  DEFAULT_MIN_TOKEN_SIZE
  DEFAULT_OAUTH_HOST
  DEFAULT_OAUTH_MINIMUM_VALIDITY_MS
  DEFAULT_OAUTH_REFRESH_TIMEOUT_MS
  DEFAULT_OAUTH_SCOPE
  DEFAULT_PROFILE
  DEFAULT_SETTINGS_ENABLE_PUSH
  DEFAULT_SETTINGS_HEADER_TABLE_SIZE
  DEFAULT_SETTINGS_INITIAL_WINDOW_SIZE
  DEFAULT_SETTINGS_MAX_FRAME_SIZE
  DEFAULT_TOKEN_LIFESPAN
  DEFAULT_TOKEN_LIFETIME_SECONDS
  DEFAULT_TOKEN_URL
  DEPRECATED_KEYS
  DESIRED_KITTY_KEYBOARD_PROTOCOL_FLAGS
  DEVICE_AUTHORIZATION_URL
  DEVICE_TOKEN_URL
  DIRECT_TOKEN_SCOPE
  DNSKEY
  DOCS_TOKEN
  DOTTED_KEY
  DOT_KEYWORD
  DTS_KEYWORD
  DUPLICATE_KEY
  D_KEYWORDS
  D_SPECIAL_TOKEN_SEQUENCE_MODE
  D_TOKEN_STRING_MODE
  ELIXIR_KEYWORDS
  ENDPOINT_CONFIG_OPTIONS
  ENDPOINT_MODE_CONFIG_OPTIONS
    [truncated: more than 200 lines]
baseline-env-PI_CODING_AGENT_DIR:
  # /home/agent/probe-target
  d /home/agent/probe-target
  # /home/agent/.pi
  d /home/agent/.pi
  d /home/agent/.pi/agent
  f /home/agent/.pi/agent/auth.json
  f /home/agent/.pi/agent/models-store.json
delta-env-PI_CODING_AGENT_DIR:
  + d /home/agent/probe-target/sessions
  + d /home/agent/probe-target/sessions/--home-probe-work--
  + f /home/agent/probe-target/auth.json
  + f /home/agent/probe-target/models-store.json
container-delta:
  C /etc
  C /home/agent
  C /home
  A /home/agent/.local
  A /home/agent/.local/bin
  A /home/agent/.local/bin/pi
  A /home/agent/.local/lib
  A /home/agent/.local/lib/node_modules
  A /home/agent/.local/lib/node_modules/@earendil-works
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/CHANGELOG.md
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/README.md
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bun
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bun/cli.d.ts
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bun/cli.d.ts.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bun/cli.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bun/cli.js.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bun/restore-sandbox-env.d.ts
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bun/restore-sandbox-env.d.ts.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bun/restore-sandbox-env.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bun/restore-sandbox-env.js.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bun/runtime-setup.d.ts
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bun/runtime-setup.d.ts.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bun/runtime-setup.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bun/runtime-setup.js.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bun/sandbox-env-setup.d.ts
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bun/sandbox-env-setup.d.ts.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bun/sandbox-env-setup.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bun/sandbox-env-setup.js.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/anthropic-messages-UIH6DSZT.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/anthropic.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/azure-openai-responses-KZZFKCXW.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/bedrock-converse-stream.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/chunk-22CYIUS4.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/chunk-2JW42LFQ.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/chunk-2LOB3CBH.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/chunk-3ZATHNCK.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/chunk-4L3WN2XY.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/chunk-5GTPJSRI.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/chunk-66G7ITPS.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/chunk-6CYCI3EJ.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/chunk-6MWTY2OB.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/chunk-6QGV3IWD.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/chunk-7ZC4UXSL.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/chunk-AI62ELJX.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/chunk-AKJ3ZOR3.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/chunk-DOST5PQW.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/chunk-DRQDKNXA.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/chunk-F5ZABL6Z.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/chunk-GDR5JFDJ.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/chunk-H3QXYIF7.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/chunk-HACVPRBJ.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/chunk-ITZNWD4K.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/chunk-L2ZPNLYD.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/chunk-LPXNMA2W.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/chunk-PGXDEOVV.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/chunk-PJACZCIR.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/chunk-PMRZO46Y.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/chunk-PRRNXB7S.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/chunk-PYGXRV27.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/chunk-RDZVAGPR.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/chunk-RH6M5TBT.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/chunk-S3GGRYIO.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/chunk-TEPMHNKQ.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/chunk-UCKJ6OH5.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/chunk-USJZMAIR.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/chunk-UTB4DTYQ.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/chunk-WJUNYZC5.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/chunk-XIX7LUWI.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/chunk-ZBURYEWF.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/cli-SWM4FQ6V.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/cloudflare-workers-ai-system-one-UTBL2KOF.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/codemode-worker.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/easter-egg-3d-OX6RUCIS.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/execute-HXNYK3DM.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/github-copilot.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/google-generative-ai-KJOKBJM3.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/google-vertex-VCLZ36OB.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/https-proxy-agent-2VXB7436.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/image-resize-worker.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/jiti-loader-C3NU2SDH.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/jiti-static-loader-E6ANQD5T.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/kimi-coding.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/lib-TEDK3E5I.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/llama-cpp-classify-FYN623RH.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/meta.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/mistral-conversations-AMQJ43DD.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/multipart-parser-SQ6GDJL7.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/node-N5BSDOA5.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/node-TWVVAGPO.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/openai-chatgpt.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/openai-codex-responses-ZYGHVE5B.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/openai-codex.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/openai-completions-BGCEXFAB.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/openai-decisions-WVPMQ4JD.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/openai-responses-KCCCXI3B.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/openrouter-images-DUPG5DZO.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/openrouter.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/pi-messages-V27DSLVC.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/radius.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/runtime-366HE7CW.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/src-UK77SYZE.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/typesafe-system-one-ZSK3RN4B.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/virtual-modules-A7JIGWCY.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/chunks/xai.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/cli-runtime.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/cli.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/index.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/bundle/rpc-entry.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli/args.d.ts
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli/args.d.ts.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli/args.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli/args.js.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli/auth-check.d.ts
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli/auth-check.d.ts.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli/auth-check.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli/auth-check.js.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli/auth-command.d.ts
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli/auth-command.d.ts.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli/auth-command.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli/auth-command.js.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli/config-selector.d.ts
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli/config-selector.d.ts.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli/config-selector.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli/config-selector.js.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli/credential-print.d.ts
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli/credential-print.d.ts.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli/credential-print.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli/credential-print.js.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli/file-processor.d.ts
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli/file-processor.d.ts.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli/file-processor.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli/file-processor.js.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli/initial-message.d.ts
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli/initial-message.d.ts.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli/initial-message.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli/initial-message.js.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli/list-models.d.ts
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli/list-models.d.ts.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli/list-models.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli/list-models.js.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli/project-trust.d.ts
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli/project-trust.d.ts.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli/project-trust.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli/project-trust.js.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli/session-picker.d.ts
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli/session-picker.d.ts.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli/session-picker.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli/session-picker.js.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli/setup.d.ts
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli/setup.d.ts.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli/setup.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli/setup.js.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli/startup-ui.d.ts
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli/startup-ui.d.ts.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli/startup-ui.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli/startup-ui.js.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli.d.ts
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli.d.ts.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli.js.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/config.d.ts
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/config.d.ts.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/config.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/config.js.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/agent-session-runtime.d.ts
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/agent-session-runtime.d.ts.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/agent-session-runtime.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/agent-session-runtime.js.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/agent-session-services.d.ts
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/agent-session-services.d.ts.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/agent-session-services.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/agent-session-services.js.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/agent-session.d.ts
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/agent-session.d.ts.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/agent-session.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/agent-session.js.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/auth-guidance.d.ts
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/auth-guidance.d.ts.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/auth-guidance.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/auth-guidance.js.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/auth-storage.d.ts
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/auth-storage.d.ts.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/auth-storage.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/auth-storage.js.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/bash-executor.d.ts
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/bash-executor.d.ts.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/bash-executor.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/bash-executor.js.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/bug-report-upload.d.ts
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/bug-report-upload.d.ts.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/bug-report-upload.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/bug-report-upload.js.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/bug-report.d.ts
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/bug-report.d.ts.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/bug-report.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/bug-report.js.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/cache-stats.d.ts
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/cache-stats.d.ts.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/cache-stats.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/cache-stats.js.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/cache-warmer.d.ts
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/cache-warmer.d.ts.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/cache-warmer.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/cache-warmer.js.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/compaction
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/compaction/branch-summarization.d.ts
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/compaction/branch-summarization.d.ts.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/compaction/branch-summarization.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/compaction/branch-summarization.js.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/compaction/compaction.d.ts
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/compaction/compaction.d.ts.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/compaction/compaction.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/compaction/compaction.js.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/compaction/index.d.ts
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/compaction/index.d.ts.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/compaction/index.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/compaction/index.js.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/compaction/utils.d.ts
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/compaction/utils.d.ts.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/compaction/utils.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/compaction/utils.js.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/crash-log.d.ts
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/crash-log.d.ts.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/crash-log.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/crash-log.js.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/defaults.d.ts
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/defaults.d.ts.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/defaults.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/defaults.js.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/diagnostics.d.ts
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/diagnostics.d.ts.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/diagnostics.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/diagnostics.js.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/event-bus.d.ts
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/event-bus.d.ts.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/event-bus.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/event-bus.js.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/exec.d.ts
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/exec.d.ts.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/exec.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/exec.js.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/experimental.d.ts
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/experimental.d.ts.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/experimental.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/experimental.js.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/export-html
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/export-html/ansi-to-html.d.ts
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/export-html/ansi-to-html.d.ts.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/export-html/ansi-to-html.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/export-html/ansi-to-html.js.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/export-html/index.d.ts
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/export-html/index.d.ts.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/export-html/index.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/export-html/index.js.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/export-html/template.css
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/export-html/template.html
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/export-html/template.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/export-html/tool-renderer.d.ts
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/export-html/tool-renderer.d.ts.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/export-html/tool-renderer.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/export-html/tool-renderer.js.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/export-html/vendor
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/export-html/vendor/highlight.min.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/export-html/vendor/marked.min.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/extensions
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/extensions/index.d.ts
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/extensions/index.d.ts.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/extensions/index.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/extensions/index.js.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/extensions/jiti-loader.d.ts
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/extensions/jiti-loader.d.ts.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/extensions/jiti-loader.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/extensions/jiti-loader.js.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/extensions/jiti-static-loader.d.ts
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/extensions/jiti-static-loader.d.ts.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/extensions/jiti-static-loader.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/extensions/jiti-static-loader.js.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/extensions/loader.d.ts
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/extensions/loader.d.ts.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/extensions/loader.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/extensions/loader.js.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/extensions/runner.d.ts
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/extensions/runner.d.ts.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/extensions/runner.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/extensions/runner.js.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/extensions/types.d.ts
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/extensions/types.d.ts.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/extensions/types.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/extensions/types.js.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/extensions/virtual-modules.d.ts
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/extensions/virtual-modules.d.ts.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/extensions/virtual-modules.js
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/extensions/virtual-modules.js.map
  A /home/agent/.local/lib/node_modules/@earendil-works/pi-coding-agent/dist/core/extensions/wrapper.d.ts
    [truncated: more than 300 lines]
