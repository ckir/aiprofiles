custody: ci
run-id: 39024332187
run-url: https://github.com/ckir/aiprofiles/actions/runs/39024332187
harness-commit: 16737cb79a0cde5bb698fae7784e69b9eeb6b551
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
  opencode v2.0.32
version-extracted: v2.0.32
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
    --completions <bash|zsh|fish|sh>                                    Print shell completion script (choices: bash, zsh, fish, sh)
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
  # scanned 11 file(s), 1059926243 byte(s) under /home/agent/.local/lib/node_modules/@opencode/cli
  documented ANTHROPIC_API_KEY: present
  documented OPENAI_API_KEY: present
  documented OPENCODE_CONFIG_DIR: present
  documented OPENCODE_CONFIG: present
  documented XDG_DATA_HOME: present
  #
  # variable-shaped tokens that name a credential or a location, documented or not
  ABORT_ERRERR_ACCESS_DENIEDERR_AMBIGUOUS_ARGUMENTERR_ARG_NOT_ITERABLEERR_ASSERTIONERR_ASYNC_CALLBACKERR_ASYNC_TYPEERR_BODY_ALREADY_USEDERR_BORINGSSLERR_BROTLI_INVALID_PARAMERR_BUFFER_OUT_OF_BOUNDSERR_BUFFER_TOO_LARGEERR_CHILD_PROCESS_IPC_REQUIREDERR_CHILD_PROCESS_STDIO_MAXBUFFERERR_CLOSED_MESSAGE_PORTERR_CONSOLE_WRITABLE_STREAMERR_CONSTRUCT_CALL_INVALIDERR_CONSTRUCT_CALL_REQUIREDERR_CRYPTO_CUSTOM_ENGINE_NOT_SUPPORTEDERR_CRYPTO_ECDH_INVALID_FORMATERR_CRYPTO_ECDH_INVALID_PUBLIC_KEYERR_CRYPTO_HASH_F  [line truncated]
  ACTIONS_ID_TOKEN_REQUEST_TOKEN
  ACTIONS_ID_TOKEN_REQUEST_URL
  ADDRCONFIG
  AES_KEY_SETUP_FAILED
  AFTER_DOCTYPE_PUBLIC_KEYWORD
  AFTER_DOCTYPE_SYSTEM_KEYWORD
  AICORE_SERVICE_KEY
  AI_GATEWAY_API_KEY
  ALIAS_KEYS
  ALIBABA_API_KEY
  ANTHROPIC_API_KEY
  ANTHROPIC_AUTH_TOKEN
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
  BUN_CONFIG_MAX_HTTP_REQUESTS
  BUN_CONFIG_MAX_HTTP_REQUESTSBUN_CONFIG_MAX_HTTP_REQUESTS
  BUN_CONFIG_SKIP_INSTALL_PACKAGESC
  BUN_CONFIG_TOKEN
  BUN_CONFIG_VERBOSE_FETCH
  BUN_CONFIG_WS_HANDSHAKE_TIMEOUT
  BUN_CONFIG_YARN_LOCKFILEBUN_CONFIG_HTTP_RETRY_COUNTBUN_CONFIG_SKIP_SAVE_LOCKFILEBUN_CONFIG_SKIP_LOAD_LOCKFILEBUN_CONFIG_NO_VERIFYM
  BUN_FEATURE_FLAG_DISABLE_ADDRCONFIG
  BUN_GC_TIMER_INTERVALBUN_CONFIG_HTTP_IDLE_TIMEOUTBUN_INOTIFY_COALESCE_INTERVALBUN_INSTALL_STREAMING_MIN_SIZEBUN_CONFIG_WS_HANDSHAKE_TIMEOUTBUN_CONFIG_DNS_TIME_TO_LIVE_SECONDSBUN_GC_RUNS_UNTIL_SKIP_RELEASE_ACCESSBUN_INSTALL_STREAMING_DRAIN_THRESHOLDCOLUMNS
  BUN_INSTALL_GLOBAL_STOREBUN_INSTALL_PROGRESSBUN_CONFIG_REGISTRYNPM_CONFIG_REGISTRY
  BUN_WORKER_MESSAGING_KEY
  BUN_WORKER_PARENT_PORT_KEY
  BUN_WORKER_STDIO_KEY
  BZMPOPCONFIGF
  CANNOT_HAVE_BOTH_PRIVKEY_AND_METHOD
  CANNOT_RECOVER_MULTI_PRIME_KEY
  CANT_CHECK_DH_KEY
  CEREBRAS_API_KEY
  CERTIFICATE_AND_PRIVATE_KEY_MISMATCH
  CERTIFICATE_CONFIGURATION_ENV_VARIABLE
  CERTIFICATE_HAS_NO_KEYID
  CF_AIG_TOKEN
  CLIENT_EARLY_TRAFFIC_SECRET
  CLIENT_HANDSHAKE_TRAFFIC_SECRET
  CLIENT_HTTP_AUTHENTICATION
  CLIENT_KEY
  CLIENT_TRAFFIC_SECRET_0
  CLOUDFLARE_API_KEY
  CLOUDFLARE_API_TOKEN
  CLOUDFLARE_WORKERS_AI_TOKEN
  CLOUDSDK_CONFIG
  CMAKE_CXX_COMPILER_LAUNCHERCMAKE_C_COMPILER_LAUNCHERNODE_TLS_REJECT_UNAUTHORIZEDBUN_CONFIG_NO_CLEAR_TERMINAL_ON_RELOADCITDDIUMGITHUB_ACTIONSJENKINS_URL
  COHERE_API_KEY
  COMMENT_KEYS
  CONFIG
  CONFIG_MAX_ATTEMPTS
  CONFIG_PREFIX_SEPARATOR
  CONFIG_RETRY_MODE
  CONFIG_USE_DUALSTACK_ENDPOINT
  CONFIG_USE_FIPS_ENDPOINT
  CREDENTIALS_CODE
  CREDENTIALS_ENV_VARS
  CREDENTIALS_ENV_VARS0
  CREDENTIALS_ENV_VARS_STS_WEB_ID_TOKEN
  CREDENTIALS_HTTP
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
  CREDENTIALS_STS_ASSUME_ROLE_WEB_ID
  CREDENTIAL_QUERY_PARAM
  DASHSCOPE_API_KEY
  DATABRICKS_TOKEN
  DATABRICKS_TOKENG
  DATA_TOO_LARGE_FOR_KEY_SIZE
  DATA_TOO_SMALL_FOR_KEY_SIZE
  DEBUG_AUTH
  DEEPINFRA_API_KEY
  DEEPSEEK_API_KEY
  DEEP_HEADER_AUTHORIZATION
  DEFAULT_MAX_TOKEN_LIFETIME_SECS_
  DEFAULT_PROFILE
  DEFAULT_SETTINGS_ENABLE_CONNECT_PROTOCOL
  DEFAULT_SETTINGS_ENABLE_PUSH
  DEFAULT_SETTINGS_HEADER_TABLE_SIZE
  DEFAULT_SETTINGS_INITIAL_WINDOW_SIZE
  DEFAULT_SETTINGS_MAX_CONCURRENT_STREAMS
  DEFAULT_SETTINGS_MAX_FRAME_SIZE
  DEFAULT_SETTINGS_MAX_HEADER_LIST_SIZE
  DEPRECATED_KEYS
  DIFFERENT_KEY_TYPES
  DIGEST_AND_KEY_TYPE_NOT_SUPPORTED
  DIGEST_TOO_BIG_FOR_RSA_KEY
  DIGITALOCEAN_ACCESS_TOKEN
  DIGITALOCEAN_API_KEY
  DO_INFERENCE_API_KEY
  DUMPHGETHSETXDELZREMHOME
  DUPLICATE_KEY_SHARE
  EAUTHIP
  ECH_SERVER_CONFIG_AND_PRIVATE_KEY_MISMATCH
  ECH_SERVER_CONFIG_UNSUPPORTED_EXTENSION
  ECH_SERVER_WOULD_HAVE_NO_RETRY_CONFIGS
  EEXPIREDSIGNATUREKEY
  EIOEADVEDOME2BIGEBADEEBADFEBADREBUSYECOMMEFBIGEIDRMEINTRELOOPENOSRENXIOEPERMEPIPEEROFSESRCHETIMEEXDEVEACCESEAGAINEBADFDEBFONTECHILDECHRNGEDQUOTEEXISTEFAULTEILSEQEINVALEISDIREISNAMEL2HLTEL3HLTEL3RSTELNRNGEMFILEEMLINKENFILEENOANOENOCSIENODEVENOENTENOKEYENOLCKENOMEMENOMSGENONETENOPKGENOSPCENOSTRENOSYSENOTTYEPROTOERANGEESPIPEESRMNTESTALEEUSERSEXFULLEBADMSGEBADRQCEBADSLTEDEADLKEDOTDOTEISCONNELIBACCELIBBADELIBMAXELIBSCNENAVAILENOBUFSENODATAENOEXECENOLINKENOTBLKENOTDIRENOTNAMENOTSUPEREMCHGEREMOTEERFKIL  [line truncated]
  EMISSINGSIGNATUREKEY
  EMPTY_PUBLIC_KEY
  ENGINE_METHOD_PKEY_ASN1_METHS
  ENGINE_METHOD_PKEY_METHS
  ENV_KEY
  ENV_PROFILE
  ENV_SECRET
  EPERMENOENTESRCHEINTREIOENXIOE2BIGENOEXECEBADFECHILDEAGAINENOMEMEACCESEFAULTENOTBLKEBUSYEEXISTEXDEVENODEVENOTDIREISDIREINVALENFILEEMFILEENOTTYETXTBSYEFBIGENOSPCESPIPEEROFSEMLINKEPIPEEDOMEADVERANGEEDEADLKENAMETOOLONGENOLCKENOSYSENOTEMPTYELOOPENOMSGEIDRMECHRNGEL3HLTEL3RSTELNRNGEUNATCHENOCSIEL2HLTEBADEEBADREXFULLENOANOEBADRQCEBADSLTEBFONTENOSTRENODATAETIMEENOSRENONETENOPKGEREMOTEENOLINKESRMNTECOMMEPROTOEMULTIHOPEDOTDOTEBADMSGEOVERFLOWEBADFDEREMCHGELIBACCELIBBADELIBSCNELIBMAXEILSEQEUSERSEDESTADDRREQ  [line truncated]
  EPERMENOENTESRCHEINTREIOENXIOE2BIGENOEXECEBADFECHILDEAGAINENOMEMEACCESEFAULTENOTBLKEBUSYEEXISTEXDEVENODEVENOTDIREISDIREINVALENFILEEMFILEENOTTYETXTBSYEFBIGENOSPCESPIPEEROFSEMLINKEPIPEERANGEEDEADLKENAMETOOLONGENOLCKENOSYSENOTEMPTYELOOPENOMSGEIDRMECHRNGEL3HLTEL3RSTELNRNGEUNATCHENOCSIEL2HLTEBADEEBADREXFULLENOANOEBADRQCEBADSLTEBFONTENOSTRENODATAETIMEENOSRENONETENOPKGEREMOTEENOLINKESRMNTECOMMEPROTOEMULTIHOPEDOTDOTEBADMSGEOVERFLOWEBADFDEREMCHGELIBACCELIBBADELIBSCNELIBMAXEILSEQEUSERSEDESTADDRREQEPROTOTY  [line truncated]
  EPERMENOENTESRCHEINTREIOENXIOE2BIGENOEXECEBADFECHILDEAGAINENOMEMEACCESEFAULTENOTBLKEBUSYEEXISTEXDEVENODEVENOTDIREISDIREINVALENFILEEMFILEENOTTYETXTBSYEFBIGENOSPCESPIPEEROFSEMLINKEPIPEERANGEEDEADLKENAMETOOLONGENOLCKENOSYSENOTEMPTYELOOPENOMSGEIDRMECHRNGEL3HLTEL3RSTELNRNGEUNATCHENOCSIEL2HLTEBADEEBADREXFULLENOANOEBADRQCEBADSLTEBFONTENOSTRENODATAETIMEENOSRENONETENOPKGEREMOTEENOLINKESRMNTECOMMEPROTOEMULTIHOPEDOTDOTEBADMSGEOVERFLOWEBADFDEREMCHGELIBACCELIBBADELIBSCNELIBMAXEILSEQEUSERSEDESTADDRREQEPROTOTY  [line truncated]
  ERANGEEDEADLKENAMETOOLONGENOLCKENOSYSENOTEMPTYELOOPENOMSGEIDRMECHRNGEL3HLTEL3RSTELNRNGEUNATCHENOCSIEL2HLTEBADEEBADREXFULLENOANOEBADRQCEBADSLTEBFONTENOSTRENODATAETIMEENOSRENONETENOPKGEREMOTEENOLINKESRMNTECOMMEPROTOEMULTIHOPEDOTDOTEBADMSGEOVERFLOWEBADFDEREMCHGELIBACCELIBBADELIBSCNELIBMAXEILSEQEUSERSEDESTADDRREQEPROTOTYPEENOPROTOOPTEPROTONOSUPPORTESOCKTNOSUPPORTEOPNOTSUPPEPFNOSUPPORTEAFNOSUPPORTEADDRINUSEEADDRNOTAVAILENETUNREACHENETRESETECONNABORTEDECONNRESETENOBUFSEISCONNESHUTDOWNETOOMANYREFSETIME  [line truncated]
  ERROR_CONVERTING_PRIVATE_KEY
  ERR_CRYPTO_ECDH_INVALID_PUBLIC_KEY
  ERR_CRYPTO_INCOMPATIBLE_KEY
  ERR_CRYPTO_INCOMPATIBLE_KEY_OPTIONS
  ERR_CRYPTO_INVALID_AUTH_TAG
  ERR_CRYPTO_INVALID_KEYLEN
  ERR_CRYPTO_INVALID_KEYPAIR
  ERR_CRYPTO_INVALID_KEYTYPE
  ERR_CRYPTO_INVALID_KEY_OBJECT_TYPE
  ERR_CRYPTO_JWK_UNSUPPORTED_KEY_TYPE
  ERR_CRYPTO_SIGN_KEY_REQUIRED
  ERR_HTTP2_CONNECT_AUTHORITY
  ERR_HTTP2_INVALID_PACKED_SETTINGS_LENGTH
  ERR_HTTP2_MAX_PENDING_SETTINGS_ACK
  ERR_HTTP2_TOO_MANY_CUSTOM_SETTINGS
  ERR_INVALID_AUTH
  ERR_INVALID_HTTP_TOKEN
  ERR_INVALID_PACKAGE_CONFIG
  ERR_INVALID_REPL_EVAL_CONFIG
  ERR_JWKS_MULTIPLE_MATCHING_KEYS
  ERR_JWKS_NO_MATCHING_KEY
  ERR_JWKS_NO_MATCHING_KEY4
  ERR_KEY_GENERATION_JOB_FAILED
  ERR_MYSQL_AUTHENTICATION_FAILED
  ERR_MYSQL_FAILED_TO_ENCRYPT_PASSWORD
  ERR_MYSQL_INVALID_PUBLIC_KEY
  ERR_MYSQL_MISSING_AUTH_DATA
  ERR_MYSQL_PASSWORD_REQUIRED
  ERR_MYSQL_PUBLIC_KEY_RETRIEVAL_NOT_ALLOWEDERR_MYSQL_INVALID_PUBLIC_KEYERR_MYSQL_FAILED_TO_ENCRYPT_PASSWORDERR_MYSQL_MISSING_AUTH_DATAERR_MYSQL_PASSWORD_REQUIREDERR_MYSQL_LIFETIME_TIMEOUTERR_MYSQL_IDLE_TIMEOUTERR_MYSQL_CONNECTION_TIMEOUTERR_MYSQL_UNEXPECTED_PACKETERR_MYSQL_INVALID_ERROR_PACKETERR_MYSQL_INVALID_EOF_PACKETERR_MYSQL_INVALID_OK_PACKETERR_MYSQL_INVALID_PREPARE_OK_PACKETERR_MYSQL_INVALID_ENCODED_INTEGERERR_MYSQL_INVALID_BINARY_VALUEERR_MYSQL_INVALID_RESULT_ROWERR_MYSQL_INVALID_QUERY_BI  [line truncated]
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
  A /home/agent/.npm/_cacache/content-v2/sha512/1e
  A /home/agent/.npm/_cacache/content-v2/sha512/1e/02
  A /home/agent/.npm/_cacache/content-v2/sha512/1e/02/7ae3581949898caacc061865c561d5d8e2cae7bdaa12c11bc134ce1c874f28fe9f730a119fba1c2b5bed435ac2dc0acab4f435de45efcff600dc558d39dd
  A /home/agent/.npm/_cacache/content-v2/sha512/27
  A /home/agent/.npm/_cacache/content-v2/sha512/27/25
  A /home/agent/.npm/_cacache/content-v2/sha512/27/25/46be20df5a9337035ac3a269d596a17194ad513a3b970eafae956a60f336177a401d01589854b30125afc9c0ef3e5cda59844b1e720965109aeec58a0328
  A /home/agent/.npm/_cacache/content-v2/sha512/36
  A /home/agent/.npm/_cacache/content-v2/sha512/36/b0
  A /home/agent/.npm/_cacache/content-v2/sha512/36/b0/788a09496fbbc665627a074a951f510c1e1581678003b001bd5b4c4b6a7ce0c49e9e2c412c181bf1457d1c9bf28f6a0e44fcdd32646b03543658727cff39
  A /home/agent/.npm/_cacache/content-v2/sha512/3d
  A /home/agent/.npm/_cacache/content-v2/sha512/3d/5f
  A /home/agent/.npm/_cacache/content-v2/sha512/3d/5f/6a0962c27d2c9d88ebdc88796b860cf305fedd10f538beca20a39bf38eafaeaa2f256260b514b4cb35e4fa730f2eeca331e056ec4f0c65a4678169b040c0
  A /home/agent/.npm/_cacache/content-v2/sha512/43
  A /home/agent/.npm/_cacache/content-v2/sha512/43/9d
  A /home/agent/.npm/_cacache/content-v2/sha512/43/9d/287c143781dc230fa65086b551f1f5659bc0f6c4f32c872e1dca763a97a6fea8b74a801489f56a68683cab417d585a67f6b3b20de116d79a85d77498ad0c
  A /home/agent/.npm/_cacache/content-v2/sha512/49
  A /home/agent/.npm/_cacache/content-v2/sha512/49/d6
  A /home/agent/.npm/_cacache/content-v2/sha512/49/d6/4532139b655a0af974185f1bf61f07b2fe7ebeb47cd50043202028ddfd3f27ea7a3d0744fffb954c16716c11aa8f188c771af5377b401d9e81cba74521d3
  A /home/agent/.npm/_cacache/content-v2/sha512/4f
  A /home/agent/.npm/_cacache/content-v2/sha512/4f/26
  A /home/agent/.npm/_cacache/content-v2/sha512/4f/26/30e5551d96792271d05263cb5cac34da7a07e3f72fbbb38b7185e7d50afc7e0751a728eb3e9a944820c6abe9ecb927d3fb2b4f3847a37efb7445f1feb9f5
  A /home/agent/.npm/_cacache/content-v2/sha512/52
  A /home/agent/.npm/_cacache/content-v2/sha512/52/b8
  A /home/agent/.npm/_cacache/content-v2/sha512/52/b8/272074f292fea9b21f59bbf4652cfb86aaeea588fef025e08b49149cf14096bc6cc809e157504be0ae2afc0024317be40eddfb0a6c3dbfd61c1c31fc249a
  A /home/agent/.npm/_cacache/content-v2/sha512/55
  A /home/agent/.npm/_cacache/content-v2/sha512/55/4f
  A /home/agent/.npm/_cacache/content-v2/sha512/55/4f/94a4ca5f7dafb0bb96f31085ba334006828835a1471a12e56296ca5013d8869c6f4b84a0c5d02068d924877a177fca80065f3b7dccb93e4b4a455f206a2b
  A /home/agent/.npm/_cacache/content-v2/sha512/68
  A /home/agent/.npm/_cacache/content-v2/sha512/68/8f
  A /home/agent/.npm/_cacache/content-v2/sha512/68/8f/e5bf0dc3fd028d7d9db62595dc16daf782b03d3761b7d7fbd0da5894b55d0439c26ccd1ff35849d7cf61a76333d048c0e560056641dfa9d5550534617d17
  A /home/agent/.npm/_cacache/content-v2/sha512/96
  A /home/agent/.npm/_cacache/content-v2/sha512/96/90
  A /home/agent/.npm/_cacache/content-v2/sha512/96/90/d78dcc95b64af6088ec2c2f13cf77d4b89b3da7bbdcdb4c1f1a52047b4c57a03421adbc83d8232bde34fe07476a0cafbd43c22a0e2f61ca0b4d154dd62a9
  A /home/agent/.npm/_cacache/content-v2/sha512/9c
  A /home/agent/.npm/_cacache/content-v2/sha512/9c/4b
  A /home/agent/.npm/_cacache/content-v2/sha512/9c/4b/693f3d8c1b075450f2b8df74d73678fd4e0e7c3c341ca592b93ad38a7bc11bf2abc1b526c2cfe81c7539c349941e6e8fce068891ba7b7d795ffb92ee921e
  A /home/agent/.npm/_cacache/content-v2/sha512/9e
  A /home/agent/.npm/_cacache/content-v2/sha512/9e/14
  A /home/agent/.npm/_cacache/content-v2/sha512/9e/14/6540614b328ab1e7456d22f526095ff8dd6e9d47c63e397cd7ab5ef3b2d4a3ccb2147707c1646eb33f7329ce8d1c98b43686f5d6e3f3db86f94bca1c061a
  A /home/agent/.npm/_cacache/content-v2/sha512/aa
  A /home/agent/.npm/_cacache/content-v2/sha512/aa/14
  A /home/agent/.npm/_cacache/content-v2/sha512/aa/14/f3101759b53963134b05d8ecf18ee4d4f71ff7ee048680c8c255ee09e084b21f66c59ecc6276cc97d48c9440d51846e0b676860407445ade4545a4c3b4b9
  A /home/agent/.npm/_cacache/content-v2/sha512/b1
  A /home/agent/.npm/_cacache/content-v2/sha512/b1/ef
  A /home/agent/.npm/_cacache/content-v2/sha512/b1/ef/3d7ab0565d26ddebce15236d4d4efe43fcf6ffbd013fc44e5f7e7162903b2ecdf614d0882f039c02fd96969f893f15ec8f9571e74e31644075aa246db667
  A /home/agent/.npm/_cacache/content-v2/sha512/cd
  A /home/agent/.npm/_cacache/content-v2/sha512/cd/f3
  A /home/agent/.npm/_cacache/content-v2/sha512/cd/f3/ccb13a6578fb5415d712bf500acd071466bb45a29de8d17e23b601718e0e2e5416efeaaa42619ddefa5e772f3e24c88ba40d9fb167ac100370b1c22a467b
  A /home/agent/.npm/_cacache/content-v2/sha512/db
  A /home/agent/.npm/_cacache/content-v2/sha512/db/7b
  A /home/agent/.npm/_cacache/content-v2/sha512/db/7b/b2dd7e69c427459637acd65b90a78c5f309f99899366d6291cad09193c508b70039746f2604044003a2aed97cee6331e41c31b9a17914fb814c9b30ed6df
  A /home/agent/.npm/_cacache/content-v2/sha512/f9
  A /home/agent/.npm/_cacache/content-v2/sha512/f9/33
  A /home/agent/.npm/_cacache/content-v2/sha512/f9/33/a91e228387b9dee773f7357046424c766c2c78a2f7e1f4fe50c38f463ec371f37c56e6094e168e4754e207f9077621119a9fc4d34fc7019cde6379ca1fa6
  A /home/agent/.npm/_cacache/index-v5
  A /home/agent/.npm/_cacache/index-v5/2b
  A /home/agent/.npm/_cacache/index-v5/2b/1b
  A /home/agent/.npm/_cacache/index-v5/2b/1b/0c9622df0e8a899a5033b8c363f97d16b249f6fe51364d4c1ed795c3c32e
  A /home/agent/.npm/_cacache/index-v5/31
  A /home/agent/.npm/_cacache/index-v5/31/b9
  A /home/agent/.npm/_cacache/index-v5/31/b9/6f4765b4224cd87e72db0fc90b9753fb93d45bc031435734154a050ba0b3
  A /home/agent/.npm/_cacache/index-v5/3f
  A /home/agent/.npm/_cacache/index-v5/3f/b5
  A /home/agent/.npm/_cacache/index-v5/3f/b5/768f077b96e18c9b12ce33096ce1fcff6a8970754561449cb62722e93644
  A /home/agent/.npm/_cacache/index-v5/5e
  A /home/agent/.npm/_cacache/index-v5/5e/57
  A /home/agent/.npm/_cacache/index-v5/5e/57/72e098ff1fee7dcfd72dfb57c8dde70dbed2f434211a0f49a90e46c34e43
  A /home/agent/.npm/_cacache/index-v5/7f
  A /home/agent/.npm/_cacache/index-v5/7f/bd
  A /home/agent/.npm/_cacache/index-v5/7f/bd/2fb9666d3c13c8c1a1e6fe11eeee4af0256d5b3808dacb66234f74c798e6
  A /home/agent/.npm/_cacache/index-v5/89
  A /home/agent/.npm/_cacache/index-v5/89/a0
  A /home/agent/.npm/_cacache/index-v5/89/a0/bfa29b6f6dd6d076e5be806eebca32586b46f4a7adcba9ab76bfc03e7ad1
  A /home/agent/.npm/_cacache/index-v5/ad
  A /home/agent/.npm/_cacache/index-v5/ad/a0
  A /home/agent/.npm/_cacache/index-v5/ad/a0/12100d721273755db112241301dc2706902c4935ccd229956104de4e4fe7
  A /home/agent/.npm/_cacache/index-v5/b7
  A /home/agent/.npm/_cacache/index-v5/b7/b7
  A /home/agent/.npm/_cacache/index-v5/b7/b7/70d035e5a39443e5b039f5b17fb5903104ca4d4f67ec043d8419485edf29
  A /home/agent/.npm/_cacache/index-v5/bf
  A /home/agent/.npm/_cacache/index-v5/bf/53
  A /home/agent/.npm/_cacache/index-v5/bf/53/c643f047d385e390e4b7b4d29f5957a6bb84e9e3fdf1fa5f09c8fef4450c
  A /home/agent/.npm/_cacache/index-v5/c6
  A /home/agent/.npm/_cacache/index-v5/c6/6e
  A /home/agent/.npm/_cacache/index-v5/c6/6e/3d6d372326b94aedf1c7a581b6aa5503f1c5de1c9d2ffad0d4f09c2c4d2b
  A /home/agent/.npm/_cacache/index-v5/d7
  A /home/agent/.npm/_cacache/index-v5/d7/d8
  A /home/agent/.npm/_cacache/index-v5/d7/d8/99ffef73c1f258a8d4f453aadefa5299c73fbd07ae424d0b69429eedbe71
  A /home/agent/.npm/_cacache/index-v5/de
  A /home/agent/.npm/_cacache/index-v5/de/ff
  A /home/agent/.npm/_cacache/index-v5/de/ff/f7fb4442679371dd78cf820d55414fba9b114c21da5fa074dddaaefb1024
  A /home/agent/.npm/_cacache/index-v5/f2
  A /home/agent/.npm/_cacache/index-v5/f2/15
  A /home/agent/.npm/_cacache/index-v5/f2/15/47ecc4de437d6a4b8dd58e43ddd37bc7909a7aa5ceb92126ccbb1ac5e733
  A /home/agent/.npm/_cacache/index-v5/f4
  A /home/agent/.npm/_cacache/index-v5/f4/24
  A /home/agent/.npm/_cacache/index-v5/f4/24/58004a309cd78f71e3cd7892290678a702d796b7a783aecff6ef952037c4
  A /home/agent/.npm/_cacache/index-v5/f4/26
  A /home/agent/.npm/_cacache/index-v5/f4/26/77f77416be3ee0710cb97f2b22e047d817c09c3e232658a7b4aee0d5494a
  A /home/agent/.npm/_cacache/index-v5/f4/83
  A /home/agent/.npm/_cacache/index-v5/f4/83/b4d63c79f7abdf61b09f70767867807511460071290730d6a4d64a5882c5
  A /home/agent/.npm/_cacache/index-v5/f4/c5
  A /home/agent/.npm/_cacache/index-v5/f4/c5/11cd946eabff23dd99d3f5509f37bcff5270e518d3e9721e89851153f29f
  A /home/agent/.npm/_cacache/index-v5/fb
  A /home/agent/.npm/_cacache/index-v5/fb/82
  A /home/agent/.npm/_cacache/index-v5/fb/82/5df1718069e939cc0241a011755c5af560f9bb680f9c61593c9c9017a8e7
  A /home/agent/.npm/_cacache/tmp
  A /home/agent/.npm/_logs
  A /home/agent/.npm/_logs/2026-10-17T10_18_33_330Z-debug-0.log
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
  A /tmp/.bun-1001-e83060072c005618.so
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
