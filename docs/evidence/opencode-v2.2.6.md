custody: ci
run-id: 42654070717
run-url: https://github.com/ckir/aiprofiles/actions/runs/42654070717
harness-commit: 96c66274332c4054bab8794f8715ccc2e33604e1
---
install:
  npm install --global @opencode/cli
exit-codes:
  install 0
  version 0
  help 0
  behaviour-env-OPENCODE_CONFIG_DIR 0
  service-stop-dir 0
  behaviour-envfile-OPENCODE_CONFIG 0
  service-stop-file 0
probe-exit: 0
version:
  opencode v2.2.6
version-extracted: v2.2.6
help:
  DESCRIPTION
    OpenCode command line interface
  
  USAGE
    opencode <subcommand> [flags] [<directory>]
  
  ARGUMENTS
    directory string    Directory to start OpenCode in (optional)
  
  FLAGS
    --standalone            Run with a private server instead of the background service
    --server string         Connect to a server URL instead of the background service
    --auto                  Auto-approve permissions that are not explicitly denied
    --continue, -c          Continue the last session
    --session, -s string    Session ID to continue, or to create if it does not exist
    --prompt string         Prompt to use
  
  GLOBAL FLAGS
    --help, -h                                                          Show help information
    --version, -v                                                       Show version information
    --wizard                                                            Start wizard mode for a command
    --completions <bash|zsh|fish|powershell|sh|pwsh>                    Print shell completion script (choices: bash, zsh, fish, powershell, sh, pwsh)
    --log-level <all|trace|debug|info|warn|warning|error|fatal|none>    Sets the minimum log level (choices: all, trace, debug, info, warn, warning, error, fatal, none)
    --print-logs                                                        Print logs to stderr (server logs require --standalone)
  
  SUBCOMMANDS
    upgrade, update    Upgrade OpenCode to the latest or a specific version
    uninstall          Uninstall OpenCode, keeping session data, configuration, and state
    acp                Start an Agent Client Protocol server
    api                Make a request to the running server
    debug              Debugging and troubleshooting tools
    auth               manage integrations and credentials
    mcp                Manage MCP (Model Context Protocol) servers
    plugin             Manage plugins
    models             List all available models
    stats              Show shareable usage statistics
    mini               Start the minimal interactive interface
    run                Run OpenCode with a message
    session            Manage sessions
    service            Manage the background server
    reload             Reload configuration
    pair               Print one-time links to connect a browser or app
    serve              Start the v2 API and web server
strings:
  # /home/agent/.local/bin/opencode2 -> /home/agent/.local/lib/node_modules/@opencode/cli/bin/opencode.exe
  # scanned 11 file(s), 1298495538 byte(s) under /home/agent/.local/lib/node_modules/@opencode/cli
  documented ANTHROPIC_API_KEY: present
  documented OPENAI_API_KEY: present
  documented OPENCODE_CONFIG_DIR: present
  documented OPENCODE_CONFIG: present
  documented XDG_DATA_HOME: present
  #
  # variable-shaped tokens that name a credential or a location, documented or not
  ABORT_ERRERR_ACCESS_DENIEDERR_AMBIGUOUS_ARGUMENTERR_ARG_NOT_ITERABLEERR_ASSERTIONERR_ASYNC_CALLBACKERR_ASYNC_TYPEERR_BODY_ALREADY_USEDERR_BORINGSSLERR_BROTLI_INVALID_PARAMERR_BUFFER_OUT_OF_BOUNDSERR_BUFFER_TOO_LARGEERR_CHILD_PROCESS_IPC_REQUIREDERR_CHILD_PROCESS_STDIO_MAXBUFFERERR_CLOSED_MESSAGE_PORTERR_CONSOLE_WRITABLE_STREAMERR_CONSTRUCT_CALL_INVALIDERR_CONSTRUCT_CALL_REQUIREDERR_CRYPTO_CUSTOM_ENGINE_NOT_SUPPORTEDERR_CRYPTO_ECDH_INVALID_FORMATERR_CRYPTO_ECDH_INVALID_PUBLIC_KEYERR_CRYPTO_HASH_F  [line truncated]
  ACCESS_KEY_ID
  ACTIONS_ID_TOKEN_REQUEST_TOKEN
  ACTIONS_ID_TOKEN_REQUEST_URL
  ADDRCONFIG
  AES_KEY_SETUP_FAILED
  AFTER_DOCTYPE_PUBLIC_KEYWORD
  AFTER_DOCTYPE_SYSTEM_KEYWORD
  AGENT_PROXY_AUTH_TOKEN
  AICORE_SERVICE_KEY
  AI_GATEWAY_API_KEY
  ALIAS_KEYS
  ALIBABA_API_KEY
  ALLUSERSPROFILE
  ANDROID_HOME
  ANTHROPIC_API_KEY
  ANTHROPIC_AUTH_TOKEN
  ANTHROPIC_AWS_API_KEY
  ANTHROPIC_CONFIG_DIR
  ANTHROPIC_FOUNDRY_API_KEY
  ANTHROPIC_FOUNDRY_AUTH_TOKEN
  ANTHROPIC_IDENTITY_TOKEN
  ANTHROPIC_IDENTITY_TOKEN_FILE
  ANTHROPIC_PROFILE
  ANY_KEY__
  API_KEY_URL
  API_KEY_WITH_CREDENTIALS
  ATTR_ASPNETCORE_USER_IS_AUTHENTICATED
  AUTH
  AUTHCODE_EXPIRED
  AUTHORITY
  AUTHORITY1
  AUTHORITY_INFO_ACCESS
  AUTHORITY_KEYID
  AUTHORIZED
  AUTHORS
  AUTH_HEADER
  AVDISCARD_NONKEY
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
  AWS_IGNORE_CONFIGURED_ENDPOINT_URLS
  AWS_PROFILE
  AWS_SECRET_ACCESS_KEY
  AWS_SESSION_TOKEN
  AWS_SHARED_CREDENTIALS_FILE
  AWS_WEB_IDENTITY_TOKEN_FILE
  AZURE_API_KEY
  AZURE_COGNITIVE_SERVICES_API_KEY
  AZURE_OPENAI_API_KEY
  BABEL_SHOW_CONFIG_FOR
  BAD_KEY_LENGTH
  BAD_PASSWORD_READ
  BAD_SRTP_PROTECTION_PROFILE_LIST
  BAD_TOKEN
  BAGGAGE_KEY_PAIR_SEPARATOR
  BASETEN_API_KEY
  BROWSERSLIST_CONFIG
  BUILDER_KEYS
  BUN_CONFIG_DISABLE_
  BUN_CONFIG_DISABLE_COPY_FILE_RANGE
  BUN_CONFIG_DNS_TIME_TO_LIVE_SECONDS
  BUN_CONFIG_HTTP_IDLE_TIMEOUT
  BUN_CONFIG_HTTP_RETRY_BACKOFF
  BUN_CONFIG_MAX_HTTP_REQUESTS
  BUN_CONFIG_MAX_HTTP_REQUESTSBUN_CONFIG_MAX_HTTP_REQUESTS
  BUN_CONFIG_SKIP_INSTALL_PACKAGES
  BUN_CONFIG_TOKEN
  BUN_CONFIG_VERBOSE_FETCH
  BUN_CONFIG_WS_CLOSE_TIMEOUT
  BUN_CONFIG_WS_HANDSHAKE_TIMEOUT
  BUN_CONFIG_YARN_LOCKFILEBUN_CONFIG_HTTP_RETRY_COUNTBUN_CONFIG_SKIP_SAVE_LOCKFILEBUN_CONFIG_SKIP_LOAD_LOCKFILEBUN_CONFIG_NO_VERIFY
  BUN_FEATURE_FLAG_DISABLE_ADDRCONFIG
  BUN_GC_TIMER_INTERVALBUN_CONFIG_WS_CLOSE_TIMEOUTBUN_CONFIG_HTTP_IDLE_TIMEOUTBUN_INOTIFY_COALESCE_INTERVALBUN_INSTALL_STREAMING_MIN_SIZEBUN_CONFIG_WS_HANDSHAKE_TIMEOUTBUN_CONFIG_DNS_TIME_TO_LIVE_SECONDSBUN_GC_RUNS_UNTIL_SKIP_RELEASE_ACCESSBUN_INSTALL_STREAMING_DRAIN_THRESHOLDBUN_DEBUG_TEST_CHECK_PROGRESS_DELAY_MSCOLUMNS
  BUN_MANIFEST_CACHEBUN_INSTALL_GLOBAL_STOREBUN_INSTALL_PROGRESSBUN_CONFIG_REGISTRYNPM_CONFIG_REGISTRY
  BUN_WORKER_MESSAGING_KEY
  BUN_WORKER_PARENT_PORT_KEY
  BUN_WORKER_STDIO_KEY
  BZMPOPCONFIGF
  CANNOT_HAVE_BOTH_PRIVKEY_AND_METHOD
  CANNOT_RECOVER_MULTI_PRIME_KEY
  CANT_CHECK_DH_KEY
  CCR_AGENT_PROXY_TOKEN_FILE_DESCRIPTOR
  CCR_SESSION_PROFILE
  CEREBRAS_API_KEY
  CERTIFICATE_AND_PRIVATE_KEY_MISMATCH
  CERTIFICATE_CONFIGURATION_ENV_VARIABLE
  CERTIFICATE_HAS_NO_KEYID
  CF_AIG_TOKEN
  CLAUDE_AI_AUTHORIZE_URL
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
  CLAUDE_CODE_ENABLE_PROXY_AUTH_HELPER
  CLAUDE_CODE_ENABLE_TOKEN_USAGE_ATTACHMENT
  CLAUDE_CODE_FILE_READ_MAX_OUTPUT_TOKENS
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
  CLAUDE_CODE_MAX_CONTEXT_TOKENS
  CLAUDE_CODE_MAX_OUTPUT_TOKENS
  CLAUDE_CODE_MCP_SERVE_AUTH_TOKEN
  CLAUDE_CODE_MEMORY_API_TOKEN
  CLAUDE_CODE_MOCK_REMOTE_SETTINGS
  CLAUDE_CODE_OAUTH_401_WAIT_MS
  CLAUDE_CODE_OAUTH_CLIENT_ID
  CLAUDE_CODE_OAUTH_REFRESH_TOKEN
  CLAUDE_CODE_OAUTH_SCOPES
  CLAUDE_CODE_OAUTH_TOKEN
  CLAUDE_CODE_OAUTH_TOKEN_FILE_DESCRIPTOR
  CLAUDE_CODE_POST_TURN_MEMORY_CONFIG
  CLAUDE_CODE_PROFILE_STARTUP
  CLAUDE_CODE_PROXY_AUTH_HELPER_TTL_MS
  CLAUDE_CODE_RELAUNCH_HOME_TRUST
  CLAUDE_CODE_REMOTE_MCP_CONFIG
  CLAUDE_CODE_REMOTE_SETTINGS_PATH
  CLAUDE_CODE_REMOTE_SETTINGS_POLL_MS
  CLAUDE_CODE_RESTRICT_PERSONAL_CONFIG
  CLAUDE_CODE_RESUME_TOKEN_THRESHOLD
  CLAUDE_CODE_SDK_HAS_HOST_AUTH_REFRESH
  CLAUDE_CODE_SDK_HAS_OAUTH_REFRESH
  CLAUDE_CODE_SESSION_ACCESS_TOKEN
  CLAUDE_CODE_SKIP_ANTHROPIC_AWS_AUTH
  CLAUDE_CODE_SKIP_ANTHROPIC_GOOGLE_CLOUD_AUTH
  CLAUDE_CODE_SKIP_BEDROCK_AUTH
  CLAUDE_CODE_SKIP_FOUNDRY_AUTH
  CLAUDE_CODE_SKIP_MANTLE_AUTH
  CLAUDE_CODE_SKIP_VERTEX_AUTH
  CLAUDE_CODE_SLACK_TAG_TOKEN
  CLAUDE_CODE_SUBAGENT_CONFIG_WARNING
  CLAUDE_CODE_TOTAL_TOKENS_REMINDER
  CLAUDE_CODE_TOTAL_TOKENS_REMINDER_AFTER_USER_TURN
  CLAUDE_CODE_TOTAL_TOKENS_REMINDER_BUDGET
  CLAUDE_CODE_WEBSOCKET_AUTH_FILE_DESCRIPTOR
  CLAUDE_CODE_WORKFLOW_SIZE_WARNING_TOKENS
  CLAUDE_CONFIG_DIR
  CLAUDE_LOCAL_OAUTH_API_BASE
  CLAUDE_LOCAL_OAUTH_APPS_BASE
  CLAUDE_LOCAL_OAUTH_CONSOLE_BASE
  CLAUDE_SECURESTORAGE_CONFIG_DIR
  CLAUDE_SESSION_INGRESS_TOKEN_FILE
  CLAUDE_TRUSTED_DEVICE_TOKEN
  CLIENT_EARLY_TRAFFIC_SECRET
  CLIENT_HANDSHAKE_TRAFFIC_SECRET
  CLIENT_HTTP_AUTHENTICATION
  CLIENT_KEY
  CLIENT_TRAFFIC_SECRET_0
  CLOUDFLARE_API_KEY
  CLOUDFLARE_API_TOKEN
  CLOUDFLARE_WORKERS_AI_TOKEN
  CLOUDSDK_ACTIVE_CONFIG_NAME
  CLOUDSDK_AUTH_ACCESS_TOKEN
  CLOUDSDK_CONFIG
  CLOUDSDK_PROXY_PASSWORD
  CMAKE_CXX_COMPILER_LAUNCHERCMAKE_C_COMPILER_LAUNCHERNODE_TLS_REJECT_UNAUTHORIZEDBUN_CONFIG_NO_CLEAR_TERMINAL_ON_RELOADCITDDIUMGITHUB_ACTIONSJENKINS_URL
  COHERE_API_KEY
  COMMENT_KEYS
    [truncated: more than 200 lines]
baseline-env-OPENCODE_CONFIG_DIR:
  # /home/agent/probe-target
  d /home/agent/probe-target
  # /home/agent/.config/opencode
  d /home/agent/.config/opencode
  # /home/agent/.local/share/opencode
  d /home/agent/.local/share/opencode
  d /home/agent/.local/share/opencode/log
  d /home/agent/.local/share/opencode/repos
  f /home/agent/.local/share/opencode/log/opencode.log
  # /home/agent/.local/state/opencode
  d /home/agent/.local/state/opencode
delta-env-OPENCODE_CONFIG_DIR:
  + d /home/agent/.local/share/opencode/shell
  + d /home/agent/.local/share/opencode/shell/29d2f91ec979d68549ed8a21d25ee466922e3638
  + f /home/agent/.local/share/opencode/opencode.db
  + f /home/agent/.local/share/opencode/opencode.db-shm
  + f /home/agent/.local/share/opencode/opencode.db-wal
  + f /home/agent/.local/state/opencode/service.json
  + f /home/agent/probe-target/service.json
baseline-envfile-OPENCODE_CONFIG:
  # /home/agent/probe-target
  d /home/agent/probe-target
  f /home/agent/probe-target/opencode.json
  # /home/agent/.config/opencode
  d /home/agent/.config/opencode
  # /home/agent/.local/share/opencode
  d /home/agent/.local/share/opencode
  d /home/agent/.local/share/opencode/log
  d /home/agent/.local/share/opencode/repos
  f /home/agent/.local/share/opencode/log/opencode.log
  # /home/agent/.local/state/opencode
  d /home/agent/.local/state/opencode
delta-envfile-OPENCODE_CONFIG:
  + d /home/agent/.local/share/opencode/shell
  + d /home/agent/.local/share/opencode/shell/29d2f91ec979d68549ed8a21d25ee466922e3638
  + f /home/agent/.config/opencode/service.json
  + f /home/agent/.local/share/opencode/opencode.db
  + f /home/agent/.local/share/opencode/opencode.db-shm
  + f /home/agent/.local/share/opencode/opencode.db-wal
  + f /home/agent/.local/state/opencode/service.json
container-delta:
  C /etc
  C /home/agent
  C /home
  A /home/agent/.cache
  A /home/agent/.cache/opencode
  A /home/agent/.cache/opencode/bin
  A /home/agent/.config
  A /home/agent/.config/opencode
  A /home/agent/.config/opencode/service.json
  A /home/agent/.local
  A /home/agent/.local/bin
  A /home/agent/.local/bin/opencode
  A /home/agent/.local/bin/opencode2
  A /home/agent/.local/lib
  A /home/agent/.local/lib/node_modules
  A /home/agent/.local/lib/node_modules/@opencode
  A /home/agent/.local/lib/node_modules/@opencode/cli
  A /home/agent/.local/lib/node_modules/@opencode/cli/bin
  A /home/agent/.local/lib/node_modules/@opencode/cli/bin/opencode.exe
  A /home/agent/.local/lib/node_modules/@opencode/cli/node_modules
  A /home/agent/.local/lib/node_modules/@opencode/cli/node_modules/@opencode
  A /home/agent/.local/lib/node_modules/@opencode/cli/node_modules/@opencode/cli-linux-x64
  A /home/agent/.local/lib/node_modules/@opencode/cli/node_modules/@opencode/cli-linux-x64/bin
  A /home/agent/.local/lib/node_modules/@opencode/cli/node_modules/@opencode/cli-linux-x64/bin/opencode
  A /home/agent/.local/lib/node_modules/@opencode/cli/node_modules/@opencode/cli-linux-x64/package.json
  A /home/agent/.local/lib/node_modules/@opencode/cli/node_modules/@opencode/cli-linux-x64-baseline
  A /home/agent/.local/lib/node_modules/@opencode/cli/node_modules/@opencode/cli-linux-x64-baseline/bin
  A /home/agent/.local/lib/node_modules/@opencode/cli/node_modules/@opencode/cli-linux-x64-baseline/bin/opencode
  A /home/agent/.local/lib/node_modules/@opencode/cli/node_modules/@opencode/cli-linux-x64-baseline/package.json
  A /home/agent/.local/lib/node_modules/@opencode/cli/node_modules/@opencode/cli-linux-x64-baseline-musl
  A /home/agent/.local/lib/node_modules/@opencode/cli/node_modules/@opencode/cli-linux-x64-baseline-musl/bin
  A /home/agent/.local/lib/node_modules/@opencode/cli/node_modules/@opencode/cli-linux-x64-baseline-musl/bin/opencode
  A /home/agent/.local/lib/node_modules/@opencode/cli/node_modules/@opencode/cli-linux-x64-baseline-musl/package.json
  A /home/agent/.local/lib/node_modules/@opencode/cli/node_modules/@opencode/cli-linux-x64-musl
  A /home/agent/.local/lib/node_modules/@opencode/cli/node_modules/@opencode/cli-linux-x64-musl/bin
  A /home/agent/.local/lib/node_modules/@opencode/cli/node_modules/@opencode/cli-linux-x64-musl/bin/opencode
  A /home/agent/.local/lib/node_modules/@opencode/cli/node_modules/@opencode/cli-linux-x64-musl/package.json
  A /home/agent/.local/lib/node_modules/@opencode/cli/package.json
  A /home/agent/.local/lib/node_modules/@opencode/cli/postinstall.mjs
  A /home/agent/.local/share
  A /home/agent/.local/share/opencode
  A /home/agent/.local/share/opencode/log
  A /home/agent/.local/share/opencode/log/opencode.log
  A /home/agent/.local/share/opencode/opencode.db
  A /home/agent/.local/share/opencode/repos
  A /home/agent/.local/share/opencode/shell
  A /home/agent/.local/share/opencode/shell/29d2f91ec979d68549ed8a21d25ee466922e3638
  A /home/agent/.local/state
  A /home/agent/.local/state/opencode
  A /home/agent/.npm
  A /home/agent/.npm/_cacache
  A /home/agent/.npm/_cacache/content-v2
  A /home/agent/.npm/_cacache/content-v2/sha512
  A /home/agent/.npm/_cacache/content-v2/sha512/18
  A /home/agent/.npm/_cacache/content-v2/sha512/18/0e
  A /home/agent/.npm/_cacache/content-v2/sha512/18/0e/db9ebbdf2473875d6b288862f3cedd9229c2f6a0c0f92e11c8c9f4143f5e0bd975e9c731bf4bc42810ca2bd66feaf7deb57fd9efe1d4ea62bf6fb8c3928c
  A /home/agent/.npm/_cacache/content-v2/sha512/1a
  A /home/agent/.npm/_cacache/content-v2/sha512/1a/6a
  A /home/agent/.npm/_cacache/content-v2/sha512/1a/6a/a26808f1b8cba98cc443f0655353db9ae32a4e3caa1a11c073a93b66d50e9a5e5d5213352e08be93225f5431e383b39a4561f6f4b31f65351190b539ea05
  A /home/agent/.npm/_cacache/content-v2/sha512/1b
  A /home/agent/.npm/_cacache/content-v2/sha512/1b/65
  A /home/agent/.npm/_cacache/content-v2/sha512/1b/65/5542b352a86f2e25d23c2d72e69b2f9795c0445f88ffd05c93a7aa6cc9ad71b8e01ed74ef400f0dd8036acb534ffde8f58516fa8fc953bd54a983b1e7285
  A /home/agent/.npm/_cacache/content-v2/sha512/30
  A /home/agent/.npm/_cacache/content-v2/sha512/30/2c
  A /home/agent/.npm/_cacache/content-v2/sha512/30/2c/5600a892fae8b8fc937e3688b0db3e863d848ca87665a5931d2fdeb82a9631d488745b2c3c9241f44d09fb53c4e09a8dd04dee633ae8d669354975e3ee6d
  A /home/agent/.npm/_cacache/content-v2/sha512/41
  A /home/agent/.npm/_cacache/content-v2/sha512/41/36
  A /home/agent/.npm/_cacache/content-v2/sha512/41/36/b8301194493944e358ee040311671ca36595979bfa8aba5d5924c60ac6badd0b2ab902c9f7222c975170d00caf7b038059e4b42ec87cfce8a169f3903391
  A /home/agent/.npm/_cacache/content-v2/sha512/45
  A /home/agent/.npm/_cacache/content-v2/sha512/45/be
  A /home/agent/.npm/_cacache/content-v2/sha512/45/be/b76e512bf45b2a73035f4e7844e1b1d1da2aef34193135bcf28d7a993f08a667c2c13cb9c504555c79ec0b59492832c6fd98aebd87f7263c6b2b9228978b
  A /home/agent/.npm/_cacache/content-v2/sha512/47
  A /home/agent/.npm/_cacache/content-v2/sha512/47/c9
  A /home/agent/.npm/_cacache/content-v2/sha512/47/c9/92b18b1815958b8dde4c537c2a842ae9cca96ba7169e0b119ebbd63ab56a8e08b493916d9a96f3533eec1000887573464a0c27c7d43f31439ab29955eb02
  A /home/agent/.npm/_cacache/content-v2/sha512/57
  A /home/agent/.npm/_cacache/content-v2/sha512/57/86
  A /home/agent/.npm/_cacache/content-v2/sha512/57/86/daeee8b8657b715927b4664af910ef761de83ceddfa77f4c7d2632dcd6f7eca7af27162223c36045bc226e93c1aa2eadcdb5b4d39d198a9cd8536af51f77
  A /home/agent/.npm/_cacache/content-v2/sha512/61
  A /home/agent/.npm/_cacache/content-v2/sha512/61/54
  A /home/agent/.npm/_cacache/content-v2/sha512/61/54/d17b949ebe7eaa2760f85152102a0fe1d20d1c0f50d9daada2ec08c12bdec6abda4d2fdda33c8a9b58eb8b926b1e29916d4a137f1652905375dd3dc24ef3
  A /home/agent/.npm/_cacache/content-v2/sha512/68
  A /home/agent/.npm/_cacache/content-v2/sha512/68/9b
  A /home/agent/.npm/_cacache/content-v2/sha512/68/9b/7d66acaf490dc4217849d776b7b4f97522d1788c8f5bc2fd4dea69ecbe759d553cb7b1875a0d4cb82c6f07f88d823db8f23b14a03f83339da27f8890c5a4
  A /home/agent/.npm/_cacache/content-v2/sha512/6f
  A /home/agent/.npm/_cacache/content-v2/sha512/6f/cd
  A /home/agent/.npm/_cacache/content-v2/sha512/6f/cd/830a5be9ccd591c0e1f00f0503878ebed5632bddc0c619fec3c809771f79e6ffe3a18e8e37e934032a71651eb4b1a7504647e247a5f66d1029049b3adb95
  A /home/agent/.npm/_cacache/content-v2/sha512/7e
  A /home/agent/.npm/_cacache/content-v2/sha512/7e/b8
  A /home/agent/.npm/_cacache/content-v2/sha512/7e/b8/2ce0e65cd6b6b31a4c0434b04f700483b0d4d93eecb38a8f5cf5491f4ca06359059d7a603dcc0b9a394abea94a2811c53ce42ec6756be640fecfb19787eb
  A /home/agent/.npm/_cacache/content-v2/sha512/80
  A /home/agent/.npm/_cacache/content-v2/sha512/80/4b
  A /home/agent/.npm/_cacache/content-v2/sha512/80/4b/f86664f3f94b236276899f5257e8786ee75f546729fb21ff686f9eca69bdd6ecff4cdc6c73d99a719ae5eb9e7fad768b43e32fd1eb3d2d7193d2acfc5151
  A /home/agent/.npm/_cacache/content-v2/sha512/83
  A /home/agent/.npm/_cacache/content-v2/sha512/83/53
  A /home/agent/.npm/_cacache/content-v2/sha512/83/53/52846b7edb4cbfee7cd0bea0273bb6d0b210bc9d919acfc582ec9f2f8e05c8399d6ece8c73e4e88b9c26d459eaed7f608cdc3961f6dceb99925fd2273d03
  A /home/agent/.npm/_cacache/content-v2/sha512/86
  A /home/agent/.npm/_cacache/content-v2/sha512/86/ac
  A /home/agent/.npm/_cacache/content-v2/sha512/86/ac/8e99d01bf03f89a3e9a08108760e1d9f27571e07198e7023ad33e27c8090634e5dda29196fda7d02ff9ec7618a4ef71e7cf2255a6de4f4da3c446e4cd521
  A /home/agent/.npm/_cacache/content-v2/sha512/a7
  A /home/agent/.npm/_cacache/content-v2/sha512/a7/40
  A /home/agent/.npm/_cacache/content-v2/sha512/a7/40/720956251c1ff2a4e576603c68e8b90d0a969eeea6de75c31c1a180b9776fd4eb28f12a5b7ade613b60bc1385db4be08ca50d9a0747b8bced03ae0bbc0f8
  A /home/agent/.npm/_cacache/content-v2/sha512/ab
  A /home/agent/.npm/_cacache/content-v2/sha512/ab/16
  A /home/agent/.npm/_cacache/content-v2/sha512/ab/16/a4d1a64c9f9088129cc1c8e040a1ef89dc89ca077a471c8f30d3900f40a65fe5576a3e40cad9d3347a89a2b852318780b7d39b74e649120cb01309c7dd8e
  A /home/agent/.npm/_cacache/content-v2/sha512/c4
  A /home/agent/.npm/_cacache/content-v2/sha512/c4/d5
  A /home/agent/.npm/_cacache/content-v2/sha512/c4/d5/a3d049746bfcfda514f312cc78804a7d1318eee3bc5bdf45cc5c37d8c93cf85d8e03ee0d766aa8426de1dd09f054e41b40e6ff1d563fff561ec06835bbd3
  A /home/agent/.npm/_cacache/index-v5
  A /home/agent/.npm/_cacache/index-v5/08
  A /home/agent/.npm/_cacache/index-v5/08/2e
  A /home/agent/.npm/_cacache/index-v5/08/2e/52a9f97fb647545c790e500dc51191c5e4c375a60142eae254dbcee9f504
  A /home/agent/.npm/_cacache/index-v5/2b
  A /home/agent/.npm/_cacache/index-v5/2b/1b
  A /home/agent/.npm/_cacache/index-v5/2b/1b/0c9622df0e8a899a5033b8c363f97d16b249f6fe51364d4c1ed795c3c32e
  A /home/agent/.npm/_cacache/index-v5/31
  A /home/agent/.npm/_cacache/index-v5/31/b9
  A /home/agent/.npm/_cacache/index-v5/31/b9/6f4765b4224cd87e72db0fc90b9753fb93d45bc031435734154a050ba0b3
  A /home/agent/.npm/_cacache/index-v5/31/e5
  A /home/agent/.npm/_cacache/index-v5/31/e5/c44684fe4e0650c556e2d92f198e3c19dbcf4a1f8bbd6a82e48aa540218f
  A /home/agent/.npm/_cacache/index-v5/3f
  A /home/agent/.npm/_cacache/index-v5/3f/b5
  A /home/agent/.npm/_cacache/index-v5/3f/b5/768f077b96e18c9b12ce33096ce1fcff6a8970754561449cb62722e93644
  A /home/agent/.npm/_cacache/index-v5/57
  A /home/agent/.npm/_cacache/index-v5/57/62
  A /home/agent/.npm/_cacache/index-v5/57/62/ff540f909bd36357226821ab0c99d3c6f7bfc4f0d94bd1ec3ced68f080cb
  A /home/agent/.npm/_cacache/index-v5/71
  A /home/agent/.npm/_cacache/index-v5/71/a3
  A /home/agent/.npm/_cacache/index-v5/71/a3/764f8d526ca8497debb8c4dd3ef5b08248a4e6e34b214f3265dcba3b41e2
  A /home/agent/.npm/_cacache/index-v5/89
  A /home/agent/.npm/_cacache/index-v5/89/a0
  A /home/agent/.npm/_cacache/index-v5/89/a0/bfa29b6f6dd6d076e5be806eebca32586b46f4a7adcba9ab76bfc03e7ad1
  A /home/agent/.npm/_cacache/index-v5/ad
  A /home/agent/.npm/_cacache/index-v5/ad/a0
  A /home/agent/.npm/_cacache/index-v5/ad/a0/12100d721273755db112241301dc2706902c4935ccd229956104de4e4fe7
  A /home/agent/.npm/_cacache/index-v5/bf
  A /home/agent/.npm/_cacache/index-v5/bf/53
  A /home/agent/.npm/_cacache/index-v5/bf/53/c643f047d385e390e4b7b4d29f5957a6bb84e9e3fdf1fa5f09c8fef4450c
  A /home/agent/.npm/_cacache/index-v5/d7
  A /home/agent/.npm/_cacache/index-v5/d7/d8
  A /home/agent/.npm/_cacache/index-v5/d7/d8/99ffef73c1f258a8d4f453aadefa5299c73fbd07ae424d0b69429eedbe71
  A /home/agent/.npm/_cacache/index-v5/de
  A /home/agent/.npm/_cacache/index-v5/de/ff
  A /home/agent/.npm/_cacache/index-v5/de/ff/f7fb4442679371dd78cf820d55414fba9b114c21da5fa074dddaaefb1024
  A /home/agent/.npm/_cacache/index-v5/e2
  A /home/agent/.npm/_cacache/index-v5/e2/57
  A /home/agent/.npm/_cacache/index-v5/e2/57/e80f7eea239bdb863d3bbd8ef2852d0c8921ee18d17928671d8fdae0a748
  A /home/agent/.npm/_cacache/index-v5/f2
  A /home/agent/.npm/_cacache/index-v5/f2/15
  A /home/agent/.npm/_cacache/index-v5/f2/15/47ecc4de437d6a4b8dd58e43ddd37bc7909a7aa5ceb92126ccbb1ac5e733
  A /home/agent/.npm/_cacache/index-v5/f4
  A /home/agent/.npm/_cacache/index-v5/f4/24
  A /home/agent/.npm/_cacache/index-v5/f4/24/58004a309cd78f71e3cd7892290678a702d796b7a783aecff6ef952037c4
  A /home/agent/.npm/_cacache/index-v5/f4/26
  A /home/agent/.npm/_cacache/index-v5/f4/26/77f77416be3ee0710cb97f2b22e047d817c09c3e232658a7b4aee0d5494a
  A /home/agent/.npm/_cacache/index-v5/f4/c5
  A /home/agent/.npm/_cacache/index-v5/f4/c5/11cd946eabff23dd99d3f5509f37bcff5270e518d3e9721e89851153f29f
  A /home/agent/.npm/_cacache/index-v5/fb
  A /home/agent/.npm/_cacache/index-v5/fb/82
  A /home/agent/.npm/_cacache/index-v5/fb/82/5df1718069e939cc0241a011755c5af560f9bb680f9c61593c9c9017a8e7
  A /home/agent/.npm/_cacache/tmp
  A /home/agent/.npm/_logs
  A /home/agent/.npm/_logs/2026-11-09T16_23_27_911Z-debug-0.log
  A /home/agent/.npm/_update-notifier-last-checked
  A /home/agent/probe-target
  A /home/agent/probe-target/opencode.json
  C /home/probe
  A /home/probe/.probe-out
  A /home/probe/.probe-out/after-env-OPENCODE_CONFIG_DIR.txt
  A /home/probe/.probe-out/after-envfile-OPENCODE_CONFIG.txt
  A /home/probe/.probe-out/baseline-env-OPENCODE_CONFIG_DIR.txt
  A /home/probe/.probe-out/baseline-envfile-OPENCODE_CONFIG.txt
  A /home/probe/.probe-out/behaviour-env-OPENCODE_CONFIG_DIR.cmd
  A /home/probe/.probe-out/behaviour-env-OPENCODE_CONFIG_DIR.exit-code
  A /home/probe/.probe-out/behaviour-env-OPENCODE_CONFIG_DIR.txt
  A /home/probe/.probe-out/behaviour-envfile-OPENCODE_CONFIG.cmd
  A /home/probe/.probe-out/behaviour-envfile-OPENCODE_CONFIG.exit-code
  A /home/probe/.probe-out/behaviour-envfile-OPENCODE_CONFIG.txt
  A /home/probe/.probe-out/delta-env-OPENCODE_CONFIG_DIR.txt
  A /home/probe/.probe-out/delta-envfile-OPENCODE_CONFIG.txt
  A /home/probe/.probe-out/help.cmd
  A /home/probe/.probe-out/help.exit-code
  A /home/probe/.probe-out/help.txt
  A /home/probe/.probe-out/install.cmd
  A /home/probe/.probe-out/install.exit-code
  A /home/probe/.probe-out/install.txt
  A /home/probe/.probe-out/service-stop-dir.cmd
  A /home/probe/.probe-out/service-stop-dir.exit-code
  A /home/probe/.probe-out/service-stop-dir.txt
  A /home/probe/.probe-out/service-stop-file.cmd
  A /home/probe/.probe-out/service-stop-file.exit-code
  A /home/probe/.probe-out/service-stop-file.txt
  A /home/probe/.probe-out/strings.txt
  A /home/probe/.probe-out/version.cmd
  A /home/probe/.probe-out/version.exit-code
  A /home/probe/.probe-out/version.extracted
  A /home/probe/.probe-out/version.txt
  A /home/probe/.probe-state
  A /home/probe/.probe-state/failures
  A /home/probe/.probe-state/pristine
  A /home/probe/.probe-state/pristine/1.tar
  A /home/probe/.probe-state/pristine/2.tar
  A /home/probe/.probe-state/pristine/3.tar
  A /home/probe/.probe-state/pristine/index
  A /home/probe/.probe-state/steps
  A /src
  C /tmp
  A /tmp/.bun-1001-71b650bef1b3e52c.node
  A /tmp/.bun-1001-89812e6e6a68aaeb.so
  A /tmp/.bun-1001-a2b2948027b90162.so
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
    [truncated: more than 300 lines]
