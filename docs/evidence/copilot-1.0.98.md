custody: ci
run-id: 39024332187
run-url: https://github.com/ckir/aiprofiles/actions/runs/39024332187
harness-commit: 16737cb79a0cde5bb698fae7784e69b9eeb6b551
---
install:
  npm install --global @github/copilot
exit-codes:
  install 0
  version 0
  help 0
  behaviour-env-COPILOT_HOME 0
probe-exit: 0
version:
  GitHub Copilot CLI 1.0.98.
  Run 'copilot update' to check for updates.
version-extracted: 1.0.98
help:
  Usage: copilot [OPTIONS] [COMMAND]
  
  GitHub Copilot CLI - An AI-powered coding assistant.
  
  Start an interactive session to chat with Copilot, or use -p/--prompt for
  non-interactive scripting. Copilot can edit files, run shell commands, search
  your codebase, and more — all with configurable permissions.
  
  Run `copilot <command> --help` for details on any subcommand.
  
  Commands:
    app          Open the GitHub Copilot app
    login        Authenticate with Copilot
    help         Display help information
    init         Initialize Copilot instructions
    config       Manage configuration settings
    update       Download the latest version
    version      Display version information
    workflow     Run dynamic workflows
    sessions     Manage saved sessions
    memories     Manage memories
    plugin       Manage plugins
    mcp          Manage MCP servers
    skill        Manage skills
    instruction  Inspect instruction sources
    lsp          Inspect language server configuration
    sandbox      Manage command sandboxing
    completion   Generate a shell completion script
  
  Options:
    -v, --version
            show version information
    -i, --interactive <prompt>
            Start interactive mode and automatically execute this prompt
        --fleet
            Run the prompt in fleet mode (parallel subagent
            orchestration); combine with -i, -p, or piped stdin
    -p, --prompt <text>
            Execute a prompt in non-interactive mode (exits after
            completion)
    -s, --silent
            Output only the agent response (no stats), useful for
            scripting with -p
        --enable-memory
            Enable memory in prompt mode (disabled by default)
        --model <model>
            Set the AI model to use (use 'auto' to let Copilot pick
            automatically)
        --reasoning-effort <level>
            Set the reasoning effort level [possible values: none, minimal, low, medium, high, xhigh, max]
        --context <tier>
            Set the context window tier (overrides persisted setting) [possible values: default, long_context]
        --auto-tier <preference>
            Set the Auto routing profile (validated against the available catalog)
        --agent <agent>
            Specify a custom agent to use
    -r, --resume [<value>]
            Resume from a previous session (optionally specify existing
            session ID, task ID, ID prefix, or name; name matching is
            exact, case-insensitive)
        --continue
            Resume the most recent session
    -n, --name <name>
            Set a name for the new session
        --session-id <id>
            Resume an existing session or task by ID, or set the UUID
            for a new session
        --connect [<sessionId>]
            Connect directly to a remote session (optionally specify
            session ID or task ID)
        --allow-all-tools
            Allow all tools to run automatically without confirmation;
            required for non-interactive mode. Run `copilot help environment`
            for what COPILOT_ALLOW_ALL accepts and what exact "true" grants [env: COPILOT_ALLOW_ALL=]
        --allow-all-paths
            Disable file path verification and allow access to any path
        --disallow-temp-dir
            Prevent automatic access to the system temporary directory
        --banner
            Show the startup banner
        --screen-reader
            Enable screen reader optimizations
        --plain-diff
            Disable rich diff rendering (syntax highlighting via diff
            tool specified by git config)
    -C <directory>
            Change working directory before doing anything else
        --log-dir <directory>
            Set log file directory (default: ~/.copilot/logs/)
        --extension-sdk-path <directory>
            Override the bundled @github/copilot-sdk injected into
            extension subprocesses with a local `copilot-sdk/` folder.
            Invalid paths fall back to the bundled SDK.
        --log-level <level>
            Set the log level [possible values: none, error, warning, info, debug, all, default]
        --stream <mode>
            Enable or disable streaming mode [possible values: on, off]
        --output-format <format>
            Output format: 'text' (default) or 'json' (JSONL, one JSON
            object per line) [possible values: text, json]
        --share [<path>]
            Share session to markdown file after completion in
            non-interactive mode (default: ./copilot-session-<id>.md)
        --share-gist
            Share session to a secret GitHub gist after completion in
            non-interactive mode
        --add-dir <directory>
            Allow file access to a directory and load its .github/skills
            and .github/agents as trusted configuration (can be used
            multiple times). A relative path resolves against the
            session working directory (the --resume/--worktree/-C
            directory), whatever order the options appear in
        --attachment <path>
            Attach a file (image or native document) to the initial
            prompt; only valid in non-interactive mode (can be used
            multiple times)
        --disable-mcp-server <server-name>
            Disable a specific MCP server (can be used multiple times)
        --disable-builtin-mcps
            Disable all built-in MCP servers (currently:
            github-mcp-server, githubiq)
        --enable-all-github-mcp-tools
            Enable all GitHub MCP server tools instead of the default
            CLI subset. Overrides --add-github-mcp-toolset and
            --add-github-mcp-tool options.
        --add-github-mcp-toolset <toolset>
            Add a toolset to enable for the GitHub MCP server instead of
            the default CLI subset (can be used multiple times). Use
            "all" for all toolsets.
        --add-github-mcp-tool <tool>
            Add a tool to enable for the GitHub MCP server instead of
            the default CLI subset (can be used multiple times). Use "*"
            for all tools.
        --plugin-dir <directory>
            Load a plugin from a local directory (can be used multiple
            times). A relative path resolves against the session working
            directory (the --resume/--worktree/-C directory), whatever
            order the options appear in
        --additional-mcp-config <json>
            Additional MCP servers configuration as JSON string or file
            path (prefix with @) (can be used multiple times; augments
            config from ~/.copilot/mcp-config.json for this session). A
            relative @ path resolves against the session working
            directory (the --resume/--worktree/-C directory) and a
            leading ~/ expands to your home directory
        --mcp-github-auth <server=origin>
            Send the signed-in GitHub account credential only to this
            explicit --additional-mcp-config server and approved origin.
            HTTPS is required except for literal loopback HTTP. Never
            put a token in this argument. Can be used multiple times.
        --allow-all-mcp-server-instructions
            Include initialization instructions from all MCP servers in
            the system prompt instead of only allowlisted servers
        --allow-tool [<tools>...]
            Tools the CLI has permission to use; will not prompt for
            permission
        --deny-tool [<tools>...]
            Tools the CLI does not have permission to use; will not
            prompt for permission
        --available-tools [<tools>...]
            Only these tools will be available to the model
        --excluded-tools [<tools>...]
            These tools will not be available to the model
        --secret-env-vars [<vars>...]
            Environment variable names whose values are stripped from
            shell and MCP server environments and redacted from output
            (e.g., --secret-env-vars=MY_KEY,OTHER_KEY)
        --allow-url [<urls>...]
            Allow access to specific URLs or domains
        --deny-url [<urls>...]
            Deny access to specific URLs or domains, takes precedence
            over --allow-url
        --allow-all-urls
            Allow access to all URLs without confirmation
        --allow-all
            Enable all permissions (equivalent to --allow-all-tools
            --allow-all-paths --allow-all-urls)
        --yolo
            Enable all permissions (equivalent to --allow-all-tools
            --allow-all-paths --allow-all-urls)
        --max-autopilot-continues <count>
            Maximum number of continuation messages in autopilot mode [default: 5]
        --mode <mode>
            Set the initial agent mode [possible values: interactive, plan, autopilot]
        --autopilot
            Start in autopilot mode
        --plan
            Start in plan mode (combine with --mode autopilot to
            auto-approve the plan and implement it autonomously)
        --experimental
            Enable experimental features
        --bash-env [<value>]
            Enable BASH_ENV support for bash shells (on|off)
        --mouse [<value>]
            Enable mouse support in alt screen mode (on|off)
        --acp
            Start as Agent Client Protocol server
        --remote
            Enable remote control of your session from GitHub web and
            mobile
    [truncated: more than 200 lines]
strings:
  # /home/agent/.local/bin/copilot -> /home/agent/.local/lib/node_modules/@github/copilot/npm-loader.js
  # scanned 16 file(s), 186103004 byte(s) under /home/agent/.local/lib/node_modules/@github/copilot
  documented COPILOT_GITHUB_TOKEN: ABSENT
  documented GH_TOKEN: present
  documented GITHUB_TOKEN: present
  documented COPILOT_HOME: present
  #
  # variable-shaped tokens that name a credential or a location, documented or not
  ADDRCONFIG
  AI_ADDRCONFIG
  AUTHENTICATED
  AUTHENTICATING
  AUTHENTICATION
  AUTHOR
  AUTHORITY_INFO_ACCESS
  AUTHORITY_INFO_ACCESS_
  AUTHORITY_KEYID
  AUTHORITY_KEYID_
  AUTHORS
  AUTH_METHODS
  BADKEY
  BAKEYAQC
  BARE_PUBKEY
  BKEY
  BKEYNDSAKIB
  BKEYNHSAMIB
  BROKEN_PROFILE_FILE
  CALL_PROFILE_CUTOFF_PCT
  CLIENT_EARLY_TRAFFIC_SECRET
  CLIENT_HANDSHAKE_TRAFFIC_SECRET
  CLIENT_TRAFFIC_SECRET_0
  CLIENT_TRAFFIC_SECRET_N
  CONNECTION_TOKEN_CHARS
  COPILOT_CACHE_HOME
  COPILOT_HOME
  COPILOT_PKG_CACHE_HOME
  CREATUREORCAMOOSEDONKEYBLUEBERRIES
  DEFAULT_SETTINGS_ENABLE_CONNECT_PROTOCOL
  DEFAULT_SETTINGS_ENABLE_PUSH
  DEFAULT_SETTINGS_HEADER_TABLE_SIZE
  DEFAULT_SETTINGS_INITIAL_WINDOW_SIZE
  DEFAULT_SETTINGS_MAX_CONCURRENT_STREAMS
  DEFAULT_SETTINGS_MAX_FRAME_SIZE
  DEFAULT_SETTINGS_MAX_HEADER_LIST_SIZE
  DH_PUBKEY
  DKEY1WJ
  DSA_PUBKEY
  DSA_PUBKEY_
  EARLY_EXPORTER_SECRET
  ECX_KEY
  EC_KEY
  EC_KEY_
  EC_KEY_METHOD_
  EC_KEY_O
  EC_PRIVATEKEY
  EC_PRIVATEKEY_
  EC_PUBKEY
  EC_PUBKEY_
  ED25519_PUBKEY
  ED448_PUBKEY
  ENGINE_METHOD_PKEY_ASN1_METHS
  ENGINE_METHOD_PKEY_METHS
  ENUM_KEYS_CACHE_TYPE
  ERR_CPU_PROFILE_NOT_STARTED
  ERR_CPU_PROFILE_TOO_MANY
  ERR_CRYPTO_ECDH_INVALID_PUBLIC_KEY
  ERR_CRYPTO_INCOMPATIBLE_KEY
  ERR_CRYPTO_INCOMPATIBLE_KEY_OPTIONS
  ERR_CRYPTO_INVALID_AUTH_TAG
  ERR_CRYPTO_INVALID_AUTH_TAGIJR
  ERR_CRYPTO_INVALID_AUTH_TAGIJRPK
  ERR_CRYPTO_INVALID_KEYLEN
  ERR_CRYPTO_INVALID_KEYPAIR
  ERR_CRYPTO_INVALID_KEYTYPE
  ERR_CRYPTO_INVALID_KEY_OBJECT_TYPE
  ERR_CRYPTO_JWK_UNSUPPORTED_KEY_TYPE
  ERR_CRYPTO_SIGN_KEY_REQUIRED
  ERR_HEAP_PROFILE_HAVE_BEEN_STARTED
  ERR_HEAP_PROFILE_NOT_STARTED
  ERR_HTTP2_CONNECT_AUTHORITY
  ERR_HTTP2_INVALID_PACKED_SETTINGS_LENGTH
  ERR_HTTP2_MAX_PENDING_SETTINGS_ACK
  ERR_HTTP2_SETTINGS_CANCEL
  ERR_HTTP2_TOO_MANY_CUSTOM_SETTINGS
  ERR_INVALID_HTTP_TOKEN
  ERR_INVALID_PACKAGE_CONFIG
  ERR_INVALID_PACKAGE_CONFIGIJPK
  ERR_INVALID_REPL_EVAL_CONFIG
  ERR_PACKAGE_MAP_KEY_NOT_FOUND
  ERR_PROXY_INVALID_CONFIG
  ERR_UNKNOWN_CREDENTIAL
  EVP_KEYEXCH_
  EVP_KEYMGMT_
  EVP_MAX_KEY_LENGTH
  EVP_PKCS82PKEY
  EVP_PKCS82PKEY_
  EVP_PKEY
  EVP_PKEY2PKCS8
  EVP_PKEY_
  EVP_PKEY_CTX_
  EVP_PKEY_ED25519
  EVP_PKEY_ED448
  EVP_PKEY_ML_DSA_44
  EVP_PKEY_ML_DSA_65
  EVP_PKEY_ML_DSA_87
  EVP_PKEY_ML_KEM_1024
  EVP_PKEY_ML_KEM_512
  EVP_PKEY_ML_KEM_768
  EVP_PKEY_NONE
  EVP_PKEY_Q_
  EVP_PKEY_SLH_DSA_SHA2_128F
  EVP_PKEY_SLH_DSA_SHA2_128S
  EVP_PKEY_SLH_DSA_SHA2_192F
  EVP_PKEY_SLH_DSA_SHA2_192S
  EVP_PKEY_SLH_DSA_SHA2_256F
  EVP_PKEY_SLH_DSA_SHA2_256S
  EVP_PKEY_SLH_DSA_SHAKE_128F
  EVP_PKEY_SLH_DSA_SHAKE_128S
  EVP_PKEY_SLH_DSA_SHAKE_192F
  EVP_PKEY_SLH_DSA_SHAKE_192S
  EVP_PKEY_SLH_DSA_SHAKE_256F
  EVP_PKEY_SLH_DSA_SHAKE_256S
  EVP_PKEY_X25519
  EVP_PKEY_X448
  EVP_SKEYMGMT_
  EVP_SKEY_
  EXAMPLE_KEY1
  EXAMPLE_KEY2
  EXTENDED_KEY_USAGE
  EXTENDED_KEY_USAGE_
  GH_TOKEN
  GITHUB_TOKEN
  HOCKEY
  HOME
  HPE_INVALID_HEADER_TOKEN
  HTTP2SETTINGS
  HTTP2SETTINGS_CALLBACK
  HTTP2_HEADER_ACCESS_CONTROL_ALLOW_CREDENTIALS
  HTTP2_HEADER_AUTHORITY
  HTTP2_HEADER_AUTHORIZATION
  HTTP2_HEADER_HTTP2_SETTINGS
  HTTP2_HEADER_PROXY_AUTHENTICATE
  HTTP2_HEADER_PROXY_AUTHORIZATION
  HTTP2_HEADER_WWW_AUTHENTICATE
  HTTP_QUOTED_STRING_TOKENS
  HTTP_STATUS_NETWORK_AUTHENTICATION_REQUIRED
  HTTP_STATUS_NON_AUTHORITATIVE_INFORMATION
  HTTP_STATUS_PROXY_AUTHENTICATION_REQUIRED
  HTTP_STATUS_UNAUTHORIZED
  HTTP_TOKEN_CODEPOINTS
  IAKAIAEKEYR
  IDENTITY_PROVIDER_AUTHENTICATION_ERROR
  IDX_OPTIONS_MAX_OUTSTANDING_SETTINGS
  IDX_OPTIONS_MAX_SETTINGS
  IDX_SETTINGS_ENABLE_CONNECT_PROTOCOL
  IDX_SETTINGS_ENABLE_PUSH
  IDX_SETTINGS_FLAGS
  IDX_SETTINGS_HEADER_TABLE_SIZE
  IDX_SETTINGS_INITIAL_WINDOW_SIZE
  IDX_SETTINGS_MAX_CONCURRENT_STREAMS
  IDX_SETTINGS_MAX_FRAME_SIZE
  IDX_SETTINGS_MAX_HEADER_LIST_SIZE
  INSPECTOR_PROFILER
  INVALID_HEADER_TOKEN
  INVALID_TOKEN
  JS_MAP_KEY_ITERATOR_TYPE
  JS_MAP_KEY_VALUE_ITERATOR_TYPE
  JS_SET_KEY_VALUE_ITERATOR_TYPE
  KAIAEKEYIQ
  KEYBOARD
  KEYBOARD_INTERRUPT
  KEYCAP
  KEYEXPORTREQUEST
  KEYEXPORTREQUEST_CALLBACK
  KEYGENREQUEST
  KEYGENREQUEST_CALLBACK
  KEYHOLE
  KEYPAIRGENREQUEST
  KEYPAIRGENREQUEST_CALLBACK
  KEYPRESS_DECODER
  KEYS
  KEYYN
  KEY_TAG
  KEY_UPDATE_ERROR
  LISTENER_NEW_TOKEN
  MAX_ADDITIONAL_SETTINGS
  MAX_AUTH_TAG_LENGTH
  MONKEY
  NAMING_AUTHORITY
  NAMING_AUTHORITY_
  NAUTHS
  NETSCAPE_ENCRYPTED_PKEY
  NETSCAPE_ENCRYPTED_PKEY_
  NETSCAPE_PKEY
  NETSCAPE_PKEY_
  NETWORK_AUTHENTICATION_REQUIRED
  NEW_TOKEN
  NGHTTP2_SETTINGS_ENABLE_CONNECT_PROTOCOL
  NGHTTP2_SETTINGS_ENABLE_PUSH
  NGHTTP2_SETTINGS_HEADER_TABLE_SIZE
    [truncated: more than 200 lines]
baseline-env-COPILOT_HOME:
  # /home/agent/probe-target
  d /home/agent/probe-target
  # /home/agent/.copilot
  (absent)
delta-env-COPILOT_HOME:
  (no change: nothing was created, moved or removed at the watched locations)
container-delta:
  C /etc
  C /home/agent
  C /home
  A /home/agent/.cache
  A /home/agent/.cache/copilot
  A /home/agent/.cache/copilot/pkg
  A /home/agent/.cache/copilot/pkg/linux-x64
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/.extraction-complete
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/LICENSE.md
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/animations
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/animations/banner.json.gz
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/app.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/assets
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/assets/copilot.ico
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/assets/copilot.png
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/assets/copilot.svg
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/assets/copilot_win.png
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/builtin
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/builtin/customize-cloud-agent
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/builtin/customize-cloud-agent/SKILL.md
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/builtin/discover-resources
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/builtin/discover-resources/SKILL.md
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/builtin/github-pr-media
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/builtin/github-pr-media/SKILL.md
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/builtin-skills
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/builtin-skills/customize-cloud-agent
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/builtin-skills/customize-cloud-agent/SKILL.md
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/builtin-skills/discover-resources
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/builtin-skills/discover-resources/SKILL.md
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/builtin-skills/github-pr-media
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/builtin-skills/github-pr-media/SKILL.md
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/changelog.json
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/cli-main.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/copilot-sdk
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/copilot-sdk/canvas.d.ts
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/copilot-sdk/cliVersion.d.ts
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/copilot-sdk/client.d.ts
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/copilot-sdk/copilotRequestHandler.d.ts
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/copilot-sdk/docs
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/copilot-sdk/docs/agent-author.md
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/copilot-sdk/docs/examples.md
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/copilot-sdk/docs/extensions.md
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/copilot-sdk/docs/workflows.md
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/copilot-sdk/extension.d.ts
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/copilot-sdk/extension.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/copilot-sdk/ffiRuntimeHost.d.ts
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/copilot-sdk/generated
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/copilot-sdk/generated/rpc.d.ts
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/copilot-sdk/generated/session-events.d.ts
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/copilot-sdk/host.d.ts
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/copilot-sdk/index.d.ts
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/copilot-sdk/index.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/copilot-sdk/installationConfirmation.d.ts
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/copilot-sdk/runtimeArtifacts.d.ts
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/copilot-sdk/schema.d.ts
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/copilot-sdk/sdkProtocolVersion.d.ts
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/copilot-sdk/session.d.ts
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/copilot-sdk/sessionFsProvider.d.ts
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/copilot-sdk/telemetry.d.ts
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/copilot-sdk/toolSet.d.ts
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/copilot-sdk/types.d.ts
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/copilot-sdk/workflow.d.ts
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/definitions
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/definitions/code-review.agent.yaml
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/definitions/explore.agent.yaml
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/definitions/rem-agent.agent.yaml
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/definitions/research.agent.yaml
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/definitions/rubber-duck.agent.yaml
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/definitions/security-review.agent.yaml
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/definitions/sidekick
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/definitions/sidekick/cloud-session-search.yaml
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/definitions/sidekick/github-context-memory.yaml
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/definitions/sidekick/github-context.yaml
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/definitions/sidekick/session-search.yaml
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/definitions/sidekick/subconscious-agent.yaml
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/definitions/sidekick/test-sidekick-context-changed.yaml
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/definitions/sidekick/test-sidekick-persistent.yaml
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/definitions/sidekick/test-sidekick-restart.yaml
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/definitions/sidekick/test-sidekick-trigger-once.yaml
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/definitions/task.agent.yaml
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/index.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/README.md
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/deps_versions.json
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/dist
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/dist/catalog.d.ts
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/dist/catalog.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/dist/configuration.d.ts
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/dist/configuration.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/dist/detail
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/dist/detail/coreInterop.d.ts
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/dist/detail/coreInterop.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/dist/detail/model.d.ts
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/dist/detail/model.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/dist/detail/modelLoadManager.d.ts
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/dist/detail/modelLoadManager.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/dist/detail/modelVariant.d.ts
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/dist/detail/modelVariant.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/dist/foundryLocalManager.d.ts
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/dist/foundryLocalManager.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/dist/imodel.d.ts
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/dist/imodel.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/dist/index.d.ts
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/dist/index.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/dist/openai
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/dist/openai/audioClient.d.ts
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/dist/openai/audioClient.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/dist/openai/chatClient.d.ts
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/dist/openai/chatClient.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/dist/openai/embeddingClient.d.ts
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/dist/openai/embeddingClient.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/dist/openai/liveAudioSession.d.ts
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/dist/openai/liveAudioSession.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/dist/openai/liveAudioTypes.d.ts
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/dist/openai/liveAudioTypes.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/dist/openai/responsesClient.d.ts
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/dist/openai/responsesClient.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/dist/types.d.ts
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/dist/types.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/node_modules
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/node_modules/adm-zip
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/node_modules/adm-zip/LICENSE
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/node_modules/adm-zip/README.md
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/node_modules/adm-zip/adm-zip.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/node_modules/adm-zip/headers
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/node_modules/adm-zip/headers/entryHeader.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/node_modules/adm-zip/headers/index.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/node_modules/adm-zip/headers/mainHeader.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/node_modules/adm-zip/methods
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/node_modules/adm-zip/methods/deflater.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/node_modules/adm-zip/methods/index.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/node_modules/adm-zip/methods/inflater.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/node_modules/adm-zip/methods/zipcrypto.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/node_modules/adm-zip/package.json
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/node_modules/adm-zip/types.d.ts
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/node_modules/adm-zip/util
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/node_modules/adm-zip/util/constants.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/node_modules/adm-zip/util/decoder.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/node_modules/adm-zip/util/errors.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/node_modules/adm-zip/util/fattr.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/node_modules/adm-zip/util/index.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/node_modules/adm-zip/util/utils.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/node_modules/adm-zip/zipEntry.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/node_modules/adm-zip/zipFile.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/package.json
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/prebuilds
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/prebuilds/linux-x64
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/prebuilds/linux-x64/foundry_local_napi.node
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/script
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/foundry-local-sdk/node_modules/foundry-local-sdk/script/install-utils.cjs
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/index.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/npm-loader.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/package.json
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/plugins
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/plugins/github-copilot-browser
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/plugins/github-copilot-browser/.mcp.json
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/plugins/github-copilot-browser/.plugin
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/plugins/github-copilot-browser/.plugin/plugin.json
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/plugins/github-copilot-browser/.release-target
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/plugins/github-copilot-browser/channel.json
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/plugins/github-copilot-browser/dist
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/plugins/github-copilot-browser/dist/browser-mcp.cjs
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/plugins/github-copilot-browser/dist/browser-run-worker.cjs
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/plugins/github-copilot-browser/dist/native-host.cjs
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/plugins/github-copilot-browser/hooks
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/plugins/github-copilot-browser/hooks/hooks.json
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/plugins/github-copilot-browser/scripts
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/plugins/github-copilot-browser/scripts/activate.mjs
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/plugins/github-copilot-browser/scripts/deactivate.mjs
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/plugins/github-copilot-browser/scripts/doctor.mjs
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/plugins/github-copilot-browser/scripts/nativeMessagingRegistration.mjs
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/plugins/github-copilot-browser/skills
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/plugins/github-copilot-browser/skills/activate
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/plugins/github-copilot-browser/skills/activate/SKILL.md
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/plugins/github-copilot-browser/skills/browser-use
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/plugins/github-copilot-browser/skills/browser-use/BROWSER-RUN.md
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/plugins/github-copilot-browser/skills/browser-use/SKILL.md
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/plugins/github-copilot-browser/skills/deactivate
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/plugins/github-copilot-browser/skills/deactivate/SKILL.md
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/plugins/github-copilot-browser/skills/doctor
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/plugins/github-copilot-browser/skills/doctor/SKILL.md
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/prebuilds
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/prebuilds/linux-x64
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/prebuilds/linux-x64/cli-native.node
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/prebuilds/linux-x64/copilot-runtime
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/prebuilds/linux-x64/runtime.node
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/preloads
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/preloads/extension_bootstrap.mjs
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/preloads/extension_sdk_resolver.mjs
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/pvrecorder
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/pvrecorder/index.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/pvrecorder/node_modules
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/pvrecorder/node_modules/@picovoice
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/pvrecorder/node_modules/@picovoice/pvrecorder-node
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/pvrecorder/node_modules/@picovoice/pvrecorder-node/README.md
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/pvrecorder/node_modules/@picovoice/pvrecorder-node/dist
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/pvrecorder/node_modules/@picovoice/pvrecorder-node/dist/errors.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/pvrecorder/node_modules/@picovoice/pvrecorder-node/dist/index.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/pvrecorder/node_modules/@picovoice/pvrecorder-node/dist/platforms.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/pvrecorder/node_modules/@picovoice/pvrecorder-node/dist/pv_recorder.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/pvrecorder/node_modules/@picovoice/pvrecorder-node/dist/pv_recorder_status_t.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/pvrecorder/node_modules/@picovoice/pvrecorder-node/dist/types
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/pvrecorder/node_modules/@picovoice/pvrecorder-node/dist/types/errors.d.ts
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/pvrecorder/node_modules/@picovoice/pvrecorder-node/dist/types/index.d.ts
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/pvrecorder/node_modules/@picovoice/pvrecorder-node/dist/types/platforms.d.ts
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/pvrecorder/node_modules/@picovoice/pvrecorder-node/dist/types/pv_recorder.d.ts
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/pvrecorder/node_modules/@picovoice/pvrecorder-node/dist/types/pv_recorder_status_t.d.ts
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/pvrecorder/node_modules/@picovoice/pvrecorder-node/lib
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/pvrecorder/node_modules/@picovoice/pvrecorder-node/lib/linux
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/pvrecorder/node_modules/@picovoice/pvrecorder-node/lib/linux/x86_64
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/pvrecorder/node_modules/@picovoice/pvrecorder-node/lib/linux/x86_64/pv_recorder.node
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/pvrecorder/node_modules/@picovoice/pvrecorder-node/package.json
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/queries
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/queries/bash-highlights.scm
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/queries/c-highlights.scm
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/queries/cpp-highlights.scm
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/queries/csharp-highlights.scm
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/queries/css-highlights.scm
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/queries/go-highlights.scm
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/queries/html-highlights.scm
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/queries/java-highlights.scm
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/queries/javascript-highlights.scm
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/queries/json-highlights.scm
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/queries/php-highlights.scm
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/queries/python-highlights.scm
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/queries/ruby-highlights.scm
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/queries/rust-highlights.scm
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/queries/scala-highlights.scm
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/queries/typescript-highlights.scm
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/ripgrep
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/ripgrep/bin
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/ripgrep/bin/linux-x64
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/ripgrep/bin/linux-x64/rg
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/schemas
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/schemas/api.schema.json
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/schemas/session-events.schema.json
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/sea-loader.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/tgrep
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/tgrep/bin
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/tgrep/bin/linux-x64
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/tgrep/bin/linux-x64/tgrep
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/tree-sitter-bash.wasm
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/tree-sitter-c.wasm
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/tree-sitter-c_sharp.wasm
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/tree-sitter-cpp.wasm
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/tree-sitter-css.wasm
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/tree-sitter-go.wasm
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/tree-sitter-html.wasm
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/tree-sitter-java.wasm
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/tree-sitter-javascript.wasm
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/tree-sitter-json.wasm
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/tree-sitter-php.wasm
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/tree-sitter-python.wasm
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/tree-sitter-ruby.wasm
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/tree-sitter-rust.wasm
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/tree-sitter-scala.wasm
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/tree-sitter-tsx.wasm
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/tree-sitter-typescript.wasm
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/tree-sitter.wasm
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/voice-engine.worker.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/voice-installer.worker.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/voice-server.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/webview
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/webview/index.js
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/webview/node_modules
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/webview/node_modules/@webviewjs
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/webview/node_modules/@webviewjs/webview
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/webview/node_modules/@webviewjs/webview/LICENSE
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/webview/node_modules/@webviewjs/webview/README.md
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/webview/node_modules/@webviewjs/webview/cli
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/webview/node_modules/@webviewjs/webview/cli/build.mjs
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/webview/node_modules/@webviewjs/webview/cli/index.mjs
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/webview/node_modules/@webviewjs/webview/cli/utils.mjs
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/webview/node_modules/@webviewjs/webview/docs
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/webview/node_modules/@webviewjs/webview/docs/CNAME
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/webview/node_modules/@webviewjs/webview/docs/README.md
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/webview/node_modules/@webviewjs/webview/docs/api
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/webview/node_modules/@webviewjs/webview/docs/api/application.md
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/webview/node_modules/@webviewjs/webview/docs/api/browser-window.md
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/webview/node_modules/@webviewjs/webview/docs/api/menu.md
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/webview/node_modules/@webviewjs/webview/docs/api/notification.md
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/webview/node_modules/@webviewjs/webview/docs/api/tray.md
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/webview/node_modules/@webviewjs/webview/docs/api/types.md
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/webview/node_modules/@webviewjs/webview/docs/api/web-context.md
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/webview/node_modules/@webviewjs/webview/docs/api/webview.md
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/webview/node_modules/@webviewjs/webview/docs/getting-started
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/webview/node_modules/@webviewjs/webview/docs/getting-started/event-loop.md
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/webview/node_modules/@webviewjs/webview/docs/getting-started/installation.md
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/webview/node_modules/@webviewjs/webview/docs/getting-started/quick-start.md
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/webview/node_modules/@webviewjs/webview/docs/guides
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/webview/node_modules/@webviewjs/webview/docs/guides/building-executables.md
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/webview/node_modules/@webviewjs/webview/docs/guides/cookies-and-storage.md
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/webview/node_modules/@webviewjs/webview/docs/guides/custom-protocols.md
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/webview/node_modules/@webviewjs/webview/docs/guides/ipc-messaging.md
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/webview/node_modules/@webviewjs/webview/docs/guides/menus.md
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/webview/node_modules/@webviewjs/webview/docs/guides/multiple-windows.md
  A /home/agent/.cache/copilot/pkg/linux-x64/1.0.98/webview/node_modules/@webviewjs/webview/docs/platform
    [truncated: more than 300 lines]
