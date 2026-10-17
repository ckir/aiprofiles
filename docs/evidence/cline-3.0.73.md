custody: ci
run-id: 39024332187
run-url: https://github.com/ckir/aiprofiles/actions/runs/39024332187
harness-commit: 16737cb79a0cde5bb698fae7784e69b9eeb6b551
---
install:
  npm install --global cline
exit-codes:
  install 0
  version 0
  help 0
  behaviour-env-CLINE_DATA_DIR 0
  behaviour-flagdir---data-dir 0
  behaviour-flagdir---config 0
probe-exit: 0
version:
  3.0.73
version-extracted: 3.0.73
help:
  Usage: cline [options] [command] [prompt]
  
  Cline CLI - AI coding assistant in your terminal
  
  Arguments:
    prompt                        Your prompt. Default to start in act mode with
                                  auto-approve enabled.
  
  Options:
    -V, --version                 Output the version number
    -p, --plan                    Run in plan mode
    -y, --yolo                    Auto-approve a limited tool set for unsupervised
                                  runs (benchmarks, CI). DANGEROUS: use only in
                                  sandboxed environments.
    --json                        Output messages as JSON instead of styled text
    --auto-approve <boolean>      Set tool auto-approval for all tools (default:
                                  true)
    -c, --cwd <path>              Working directory
    --thinking <level>            Set reasoning effort:
                                  none|low|medium|high|xhigh. Bare --thinking uses
                                  medium; omitted uses the level last chosen with
                                  /model, or the provider default.
    --compaction <mode>           Context compaction mode: agentic|basic|off
                                  (default: agentic)
    -i, --tui                     Open the terminal user interface (TUI) for
                                  interactive sessions
    --id <session-id>             Resume an existing session by ID
    -P, --provider <id>           Provider id (default: cline)
    -k, --key <api-key>           API key for this run, also saved as the
                                  provider's key
    -m, --model <model-id>        Model to use for the session with the selected
                                  provider
    -s, --system <system-prompt>  Override the default system prompt
    -z, --zen                     Start a session that runs in the background hub
    --retries <count>             Number of maximum consecutive mistakes (retries)
                                  before exiting (default: 3)
    -t, --timeout <seconds>       Optional timeout in seconds (default: 0 for no
                                  timeout)
    --acp                         Run in Agent Client Protocol (ACP) mode for
                                  editor integration
    --config <path>               Configuration directory (default: ~/.cline)
    --data-dir <path>             Use isolated local state at this directory path
                                  (default: ~/.cline/data)
    --hooks-dir <path>            Directory path to additional hooks for runtime
                                  hook injection (default: ~/.cline/hooks)
    --worktree                    Auto-create a detached git worktree under
                                  ~/.cline/worktrees/ and run the task there
    --update                      Check for updates and install if available
    --kanban                      Run the kanban app
    -v, --verbose                 Show verbose output
    -h, --help                    display help for command
  
  Commands:
    auth [options] [provider]     Authenticate a provider and configure what model
                                  is used
    config [options]              Show current configuration
    plugin                        Manage Cline Plugins
    skill [args...]               Manage Cline Skills via the open skills CLI (npx
                                  skills)
    connect [options] [channel]   Connect to an external channel
    mcp                           Manage MCP servers
    doctor                        Diagnose and fix configuration issues
    history|h [options]           List session history or manage saved sessions
    hook                          Handle a hook payload from stdin
    schedule                      Manage scheduled tasks
    hub                           Manage the local hub daemon
    update [options]              Check for updates and install if available
    version                       Show Cline CLI version number
    kanban                        Run the kanban app
strings:
  # /home/agent/.local/bin/cline -> /home/agent/.local/lib/node_modules/cline/bin/cline
  # scanned 28185 file(s), 462057430 byte(s) under /home/agent/.local/lib/node_modules/cline
  documented ANTHROPIC_API_KEY: present
  documented CLINE_API_KEY: present
  documented OPENAI_API_KEY: present
  documented OPENROUTER_API_KEY: present
  documented CLINE_DATA_DIR: present
  #
  # variable-shaped tokens that name a credential or a location, documented or not
  ABACUS_API_KEY
  ABLIT_KEY
  ABORT_ERRERR_ACCESS_DENIEDERR_AMBIGUOUS_ARGUMENTERR_ARG_NOT_ITERABLEERR_ASSERTIONERR_ASYNC_CALLBACKERR_ASYNC_TYPEERR_BODY_ALREADY_USEDERR_BORINGSSLERR_BROTLI_INVALID_PARAMERR_BUFFER_OUT_OF_BOUNDSERR_BUFFER_TOO_LARGEERR_CHILD_PROCESS_IPC_REQUIREDERR_CHILD_PROCESS_STDIO_MAXBUFFERERR_CLOSED_MESSAGE_PORTERR_CONSOLE_WRITABLE_STREAMERR_CONSTRUCT_CALL_INVALIDERR_CONSTRUCT_CALL_REQUIREDERR_CRYPTO_CUSTOM_ENGINE_NOT_SUPPORTEDERR_CRYPTO_ECDH_INVALID_FORMATERR_CRYPTO_ECDH_INVALID_PUBLIC_KEYERR_CRYPTO_HASH_F  [line truncated]
  ABOVE_API_KEY
  ACCESS_KEY_ID
  ADDRCONFIG
  AES_KEY_SETUP_FAILED
  AFTER_DOCTYPE_PUBLIC_KEYWORD
  AFTER_DOCTYPE_SYSTEM_KEYWORD
  AGENTROUTER_API_KEY
  AGENT_CONFIG_DIRECTORY_NAME
  AGENT_PROXY_AUTH_TOKEN
  AGENT_UNEXPECTED_REASONING_TOKENS_EVENT
  AGNES_API_KEY
  AI21_API_KEY
  AIAND_API_KEY
  AICORE_SERVICE_KEY
  AIHUBMIX_API_KEY
  AIMLAPI_API_KEY
  AINETCAFE_API_KEY
  AIXY_API_KEY
  AI_API_KEY
  AI_GATEWAY_API_KEY
  AI_ROUTER_API_KEY
  AKI_IO_API_KEY
  ALIAS_KEYS
  ALIBABA_CODING_PLAN_API_KEY
  ALIBABA_TOKEN_PLAN_API_KEY
  ALLOWED_TOKEN_PATTERN
  AMAZONTOKEN
  AMBIENT_API_KEY
  AMD_API_KEY
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
  ANTHROPIC_WEBHOOK_SIGNING_KEY
  ANYAPI_API_KEY
  API_KEY
  API_KEY_URL
  API_KEY_WITH_CREDENTIALS
  APPLICATION_AUTHORIZED
  APPLICATION_DEAUTHORIZED
  ARCEE_API_KEY
  ARK_API_KEY
  ARK_CODING_PLAN_API_KEY
  ASKSAGE_API_KEY
  ASPNETCORE_AUTHENTICATION_RESULT_VALUE_FAILURE
  ASPNETCORE_AUTHENTICATION_RESULT_VALUE_NONE
  ASPNETCORE_AUTHENTICATION_RESULT_VALUE_SUCCESS
  ASPNETCORE_AUTHORIZATION_RESULT_VALUE_FAILURE
  ASPNETCORE_AUTHORIZATION_RESULT_VALUE_SUCCESS
  ASPNETCORE_IDENTITY_PASSWORD_CHECK_RESULT_VALUE_FAILURE
  ASPNETCORE_IDENTITY_PASSWORD_CHECK_RESULT_VALUE_PASSWORD_MISSING
  ASPNETCORE_IDENTITY_PASSWORD_CHECK_RESULT_VALUE_SUCCESS
  ASPNETCORE_IDENTITY_PASSWORD_CHECK_RESULT_VALUE_SUCCESS_REHASH_NEEDED
  ASPNETCORE_IDENTITY_PASSWORD_CHECK_RESULT_VALUE_USER_MISSING
  ASPNETCORE_IDENTITY_SIGN_IN_TYPE_VALUE_PASSKEY
  ASPNETCORE_IDENTITY_SIGN_IN_TYPE_VALUE_PASSWORD
  ASPNETCORE_IDENTITY_SIGN_IN_TYPE_VALUE_TWO_FACTOR_AUTHENTICATOR
  ASPNETCORE_IDENTITY_TOKEN_PURPOSE_VALUE_CHANGE_EMAIL
  ASPNETCORE_IDENTITY_TOKEN_PURPOSE_VALUE_CHANGE_PHONE_NUMBER
  ASPNETCORE_IDENTITY_TOKEN_PURPOSE_VALUE_EMAIL_CONFIRMATION
  ASPNETCORE_IDENTITY_TOKEN_PURPOSE_VALUE_OTHER
  ASPNETCORE_IDENTITY_TOKEN_PURPOSE_VALUE_RESET_PASSWORD
  ASPNETCORE_IDENTITY_TOKEN_PURPOSE_VALUE_TWO_FACTOR
  ASPNETCORE_IDENTITY_TOKEN_VERIFIED_VALUE_FAILURE
  ASPNETCORE_IDENTITY_TOKEN_VERIFIED_VALUE_SUCCESS
  ASPNETCORE_IDENTITY_USER_UPDATE_TYPE_VALUE_ADD_PASSWORD
  ASPNETCORE_IDENTITY_USER_UPDATE_TYPE_VALUE_CHANGE_PASSWORD
  ASPNETCORE_IDENTITY_USER_UPDATE_TYPE_VALUE_PASSWORD_REHASH
  ASPNETCORE_IDENTITY_USER_UPDATE_TYPE_VALUE_REMOVE_AUTHENTICATION_TOKEN
  ASPNETCORE_IDENTITY_USER_UPDATE_TYPE_VALUE_REMOVE_PASSKEY
  ASPNETCORE_IDENTITY_USER_UPDATE_TYPE_VALUE_REMOVE_PASSWORD
  ASPNETCORE_IDENTITY_USER_UPDATE_TYPE_VALUE_RESET_AUTHENTICATOR_KEY
  ASPNETCORE_IDENTITY_USER_UPDATE_TYPE_VALUE_RESET_PASSWORD
  ASPNETCORE_IDENTITY_USER_UPDATE_TYPE_VALUE_SET_AUTHENTICATION_TOKEN
  ASPNETCORE_IDENTITY_USER_UPDATE_TYPE_VALUE_SET_PASSKEY
  ASYMMETRIC_KEY_DETAILS_SUPPORTED
  ATOMIC_CHAT_API_KEY
  ATTRIBUTE_KEY
  ATTR_ASPNETCORE_AUTHENTICATION_RESULT
  ATTR_ASPNETCORE_AUTHENTICATION_SCHEME
  ATTR_ASPNETCORE_AUTHORIZATION_POLICY
  ATTR_ASPNETCORE_AUTHORIZATION_RESULT
  ATTR_ASPNETCORE_IDENTITY_PASSWORD_CHECK_RESULT
  ATTR_ASPNETCORE_IDENTITY_TOKEN_PURPOSE
  ATTR_ASPNETCORE_IDENTITY_TOKEN_VERIFIED
  ATTR_ASPNETCORE_USER_IS_AUTHENTICATED
  ATTR_AWS_S3_KEY
  ATTR_AWS_SECRETSMANAGER_SECRET_ARN
  ATTR_FEATURE_FLAG_KEY
  ATTR_GEN_AI_REQUEST_MAX_TOKENS
  ATTR_GEN_AI_TOKEN_TYPE
  ATTR_GEN_AI_USAGE_CACHE_CREATION_INPUT_TOKENS
  ATTR_GEN_AI_USAGE_CACHE_READ_INPUT_TOKENS
  ATTR_GEN_AI_USAGE_COMPLETION_TOKENS
  ATTR_GEN_AI_USAGE_INPUT_TOKENS
  ATTR_GEN_AI_USAGE_OUTPUT_TOKENS
  ATTR_GEN_AI_USAGE_PROMPT_TOKENS
  ATTR_GEN_AI_USAGE_REASONING_OUTPUT_TOKENS
  ATTR_MESSAGING_GCP_PUBSUB_MESSAGE_ORDERING_KEY
  ATTR_MESSAGING_KAFKA_MESSAGE_KEY
  ATTR_MESSAGING_RABBITMQ_DESTINATION_ROUTING_KEY
  ATTR_MESSAGING_ROCKETMQ_MESSAGE_KEYS
  ATTR_PPROF_PROFILE_COMMENT
  ATTR_PPROF_PROFILE_DOC_URL
  ATTR_PPROF_PROFILE_DROP_FRAMES
  ATTR_PPROF_PROFILE_KEEP_FRAMES
  ATTR_PROFILE_FRAME_TYPE
  AURIKO_API_KEY
  AUTH
  AUTHCODE_EXPIRED
  AUTHENTICATE
  AUTHENTICATED
  AUTHENTICATING
  AUTHENTICATION_ERROR_KEYWORDS
  AUTHOR
  AUTHORITY
  AUTHORITY1
  AUTHORITY_INFO_ACCESS
  AUTHORITY_INTRODUCER_REGION
  AUTHORITY_KEYID
  AUTHORITY_PREFIX
  AUTHORIZATION
  AUTHORIZATION_CODE_CHALLENGE_METHOD
  AUTHORIZATION_CODE_RESPONSE_TYPE
  AUTHORIZE
  AUTHORIZED
  AUTHORS
  AUTH_ERROR_PATTERNS
  AUTH_FAILED
  AUTH_HEADER
  AUTH_LOGGED_OUT
  AUTH_METHODS
  AUTH_PORT
  AUTH_REFRESH_SOFT_FAILURE
  AUTH_RUN_RETRY
  AUTH_STARTED
  AUTH_SUCCEEDED
  AUTH_TOKEN
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
  AWS_DEFAULTS_MODE_CONFIG
  AWS_IGNORE_CONFIGURED_ENDPOINT_URLS
  AWS_PROFILE
  AWS_SECRET_ACCESS_KEY
  AWS_SESSION_TOKEN
  AWS_SHARED_CREDENTIALS_FILE
  AWS_WEB_IDENTITY_TOKEN_FILE
  BABEL_SHOW_CONFIG_FOR
  BAD_KEY_LENGTH
  BAD_PASSWORD_READ
  BAD_SRTP_PROTECTION_PROFILE_LIST
  BAGGAGE_KEY
  BAGGAGE_KEY_PAIR_SEPARATOR
  BAILING_API_TOKEN
  BASETEN_API_KEY
  BASIC_AUTH_HEADER_PREFIX
  BEARER_TOKEN
  BEE_API_KEY
  BERGET_API_KEY
  BLOCK_AS_IMPLICIT_KEY
  BLOCK_DIAGRAM_KEY
  BLUECLAW_API_KEY
  BOTHUB_API_KEY
  BOUNDLESS_API_KEY
  BUILDER_KEYS
  BUN_CONFIG_DISABLE_
  BUN_CONFIG_DISABLE_COPY_FILE_RANGE
  BUN_CONFIG_DNS_TIME_TO_LIVE_SECONDS
  BUN_CONFIG_HTTP_IDLE_TIMEOUT
  BUN_CONFIG_MAX_HTTP_REQUESTS
  BUN_CONFIG_MAX_HTTP_REQUESTSBUN_CONFIG_MAX_HTTP_REQUESTS
  BUN_CONFIG_SKIP_INSTALL_PACKAGESC
    [truncated: more than 200 lines]
baseline-env-CLINE_DATA_DIR:
  # /home/agent/probe-target
  d /home/agent/probe-target
  # /home/agent/.cline
  d /home/agent/.cline
  f /home/agent/.cline/cli-node-extra-ca-certs.pem
delta-env-CLINE_DATA_DIR:
  (no change: nothing was created, moved or removed at the watched locations)
baseline-flagdir---config:
  # /home/agent/probe-target
  d /home/agent/probe-target
  # /home/agent/.cline
  d /home/agent/.cline
  f /home/agent/.cline/cli-node-extra-ca-certs.pem
delta-flagdir---config:
  (no change: nothing was created, moved or removed at the watched locations)
baseline-flagdir---data-dir:
  # /home/agent/probe-target
  d /home/agent/probe-target
  # /home/agent/.cline
  d /home/agent/.cline
  f /home/agent/.cline/cli-node-extra-ca-certs.pem
delta-flagdir---data-dir:
  (no change: nothing was created, moved or removed at the watched locations)
container-delta:
  C /etc
  C /home/agent
  C /home
  A /home/agent/.cline
  A /home/agent/.cline/cli-node-extra-ca-certs.pem
  A /home/agent/.local
  A /home/agent/.local/bin
  A /home/agent/.local/bin/cline
  A /home/agent/.local/lib
  A /home/agent/.local/lib/node_modules
  A /home/agent/.local/lib/node_modules/cline
  A /home/agent/.local/lib/node_modules/cline/LICENSE
  A /home/agent/.local/lib/node_modules/cline/README.md
  A /home/agent/.local/lib/node_modules/cline/bin
  A /home/agent/.local/lib/node_modules/cline/bin/.cline
  A /home/agent/.local/lib/node_modules/cline/bin/ca-certs.cjs
  A /home/agent/.local/lib/node_modules/cline/bin/cline
  A /home/agent/.local/lib/node_modules/cline/node_modules
  A /home/agent/.local/lib/node_modules/cline/node_modules/.bin
  A /home/agent/.local/lib/node_modules/cline/node_modules/.bin/cline
  A /home/agent/.local/lib/node_modules/cline/node_modules/.bin/jiti
  A /home/agent/.local/lib/node_modules/cline/node_modules/.bin/jsonrepair
  A /home/agent/.local/lib/node_modules/cline/node_modules/.bin/nanoid
  A /home/agent/.local/lib/node_modules/cline/node_modules/.bin/node-which
  A /home/agent/.local/lib/node_modules/cline/node_modules/.bin/semver
  A /home/agent/.local/lib/node_modules/cline/node_modules/.bin/yaml
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/CHANGELOG.md
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/LICENSE
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/README.md
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/anthropic
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/anthropic/index.d.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/dist
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/dist/anthropic
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/dist/anthropic/index.d.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/dist/anthropic/index.d.ts.map
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/dist/anthropic/index.js
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/dist/anthropic/index.js.map
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/dist/index.d.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/dist/index.d.ts.map
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/dist/index.js
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/dist/index.js.map
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/dist/mantle
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/dist/mantle/index.cjs
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/dist/mantle/index.cjs.map
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/dist/mantle/index.d.cts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/dist/mantle/index.d.cts.map
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/dist/mantle/index.d.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/dist/mantle/index.d.ts.map
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/dist/mantle/index.js
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/dist/mantle/index.js.map
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/docs
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/docs/08-amazon-bedrock.mdx
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/mantle
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/mantle/index.d.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/package.json
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/src
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/src/amazon-bedrock-anthropic-model-support.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/src/amazon-bedrock-api-types.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/src/amazon-bedrock-chat-language-model-options.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/src/amazon-bedrock-chat-language-model.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/src/amazon-bedrock-embedding-model-options.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/src/amazon-bedrock-embedding-model.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/src/amazon-bedrock-error.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/src/amazon-bedrock-event-stream-decoder.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/src/amazon-bedrock-event-stream-response-handler.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/src/amazon-bedrock-image-model-options.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/src/amazon-bedrock-image-model.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/src/amazon-bedrock-image-settings.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/src/amazon-bedrock-prepare-tools.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/src/amazon-bedrock-provider.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/src/amazon-bedrock-reasoning-metadata.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/src/amazon-bedrock-sigv4-fetch.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/src/amazon-bedrock-stream-error.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/src/anthropic
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/src/anthropic/amazon-bedrock-anthropic-fetch.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/src/anthropic/amazon-bedrock-anthropic-options.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/src/anthropic/amazon-bedrock-anthropic-provider.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/src/anthropic/index.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/src/convert-amazon-bedrock-usage.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/src/convert-to-amazon-bedrock-chat-messages.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/src/index.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/src/inject-fetch-headers.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/src/mantle
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/src/mantle/bedrock-mantle-options.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/src/mantle/bedrock-mantle-provider.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/src/mantle/index.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/src/map-amazon-bedrock-finish-reason.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/src/normalize-tool-call-id.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/src/reranking
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/src/reranking/amazon-bedrock-reranking-api.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/src/reranking/amazon-bedrock-reranking-model-options.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/src/reranking/amazon-bedrock-reranking-model.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/src/resolve-amazon-bedrock-base-url.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/amazon-bedrock/src/version.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/CHANGELOG.md
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/LICENSE
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/README.md
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/dist
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/dist/index.d.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/dist/index.d.ts.map
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/dist/index.js
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/dist/index.js.map
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/dist/internal
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/dist/internal/index.d.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/dist/internal/index.d.ts.map
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/dist/internal/index.js
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/dist/internal/index.js.map
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/docs
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/docs/05-anthropic.mdx
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/internal.d.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/package.json
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src/anthropic-api.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src/anthropic-batch.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src/anthropic-error.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src/anthropic-files.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src/anthropic-language-model-options.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src/anthropic-language-model.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src/anthropic-message-metadata.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src/anthropic-prepare-tools.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src/anthropic-provider-metadata.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src/anthropic-provider.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src/anthropic-tools.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src/convert-anthropic-usage.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src/convert-to-anthropic-prompt.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src/forward-anthropic-container-id-from-last-step.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src/get-cache-control.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src/index.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src/internal
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src/internal/index.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src/map-anthropic-citation.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src/map-anthropic-stop-reason.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src/sanitize-json-schema.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src/skills
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src/skills/anthropic-skills-api.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src/skills/anthropic-skills.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src/tool
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src/tool/advisor_20260301.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src/tool/bash_20241022.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src/tool/bash_20250124.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src/tool/code-execution_20250522.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src/tool/code-execution_20250825.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src/tool/code-execution_20260120.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src/tool/computer-toolset_20260801.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src/tool/computer_20241022.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src/tool/computer_20250124.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src/tool/computer_20251124.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src/tool/memory_20250818.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src/tool/text-editor_20241022.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src/tool/text-editor_20250124.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src/tool/text-editor_20250429.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src/tool/text-editor_20250728.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src/tool/tool-search-bm25_20251119.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src/tool/tool-search-regex_20251119.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src/tool/web-fetch-20250910.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src/tool/web-fetch-20260209.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src/tool/web-fetch-20260318.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src/tool/web-search_20250305.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src/tool/web-search_20260209.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src/tool/web-search_20260318.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/anthropic/src/version.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/CHANGELOG.md
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/LICENSE
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/README.md
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/dist
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/dist/index.d.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/dist/index.d.ts.map
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/dist/index.js
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/dist/index.js.map
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/docs
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/docs/00-ai-gateway.mdx
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/package.json
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/errors
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/errors/as-gateway-error.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/errors/create-gateway-error.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/errors/extract-api-call-response.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/errors/gateway-authentication-error.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/errors/gateway-error.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/errors/gateway-failed-dependency-error.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/errors/gateway-forbidden-error.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/errors/gateway-internal-server-error.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/errors/gateway-invalid-request-error.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/errors/gateway-model-not-found-error.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/errors/gateway-not-found-error.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/errors/gateway-rate-limit-error.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/errors/gateway-response-error.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/errors/gateway-timeout-error.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/errors/index.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/errors/parse-auth-method.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/gateway-batch.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/gateway-config.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/gateway-decision-model-settings.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/gateway-decision-model.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/gateway-embedding-model-settings.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/gateway-embedding-model.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/gateway-fetch-metadata.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/gateway-generation-info.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/gateway-headers.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/gateway-image-model-settings.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/gateway-image-model.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/gateway-language-model-settings.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/gateway-language-model.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/gateway-model-entry.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/gateway-provider-metadata.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/gateway-provider-options.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/gateway-provider.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/gateway-realtime-auth.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/gateway-realtime-model-settings.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/gateway-realtime-model.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/gateway-reranking-model-settings.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/gateway-reranking-model.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/gateway-speech-model-settings.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/gateway-speech-model.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/gateway-spend-report.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/gateway-tools.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/gateway-transcription-model-settings.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/gateway-transcription-model.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/gateway-video-model-settings.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/gateway-video-model.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/index.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/tool
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/tool/browserbase-fetch.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/tool/browserbase-search.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/tool/exa-search.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/tool/parallel-search.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/tool/perplexity-search.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/tool/tako-search.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/vercel-environment.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/version.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/gateway/src/zod.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/CHANGELOG.md
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/LICENSE
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/README.md
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/dist
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/dist/index.d.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/dist/index.d.ts.map
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/dist/index.js
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/dist/index.js.map
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/dist/internal
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/dist/internal/index.d.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/dist/internal/index.d.ts.map
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/dist/internal/index.js
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/dist/internal/index.js.map
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/docs
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/docs/15-google.mdx
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/internal.d.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/package.json
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/src
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/src/continue-google-stream.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/src/convert-google-usage.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/src/convert-to-google-messages.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/src/download-tool-result-files.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/src/get-model-path.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/src/get-realtime-base-url.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/src/google-batch.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/src/google-embedding-model-options.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/src/google-embedding-model.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/src/google-error.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/src/google-files.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/src/google-image-model-options.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/src/google-image-model.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/src/google-image-settings.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/src/google-json-accumulator.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/src/google-language-model-options.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/src/google-language-model.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/src/google-model-capabilities.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/src/google-prepare-tools.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/src/google-prompt.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/src/google-provider.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/src/google-speech-api.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/src/google-speech-input.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/src/google-speech-model-options.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/src/google-speech-model.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/src/google-supported-file-url.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/src/google-tools.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/src/google-video-model-options.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/src/google-video-model.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/src/google-video-settings.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/src/index.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/src/interactions
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/src/interactions/build-google-interactions-stream-transform.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/src/interactions/cancel-google-interaction.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/src/interactions/convert-google-interactions-usage.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/src/interactions/convert-to-google-interactions-input.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/src/interactions/extract-google-interactions-sources.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/src/interactions/google-interactions-agent.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/src/interactions/google-interactions-api.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/src/interactions/google-interactions-language-model-options.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/src/interactions/google-interactions-language-model.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/src/interactions/google-interactions-prompt.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/src/interactions/google-interactions-provider-metadata.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/src/interactions/map-google-interactions-finish-reason.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/src/interactions/parse-google-interactions-outputs.ts
  A /home/agent/.local/lib/node_modules/cline/node_modules/@ai-sdk/google/src/interactions/poll-google-interactions.ts
    [truncated: more than 300 lines]
