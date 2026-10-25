custody: ci
run-id: 40146391660
run-url: https://github.com/ckir/aiprofiles/actions/runs/40146391660
harness-commit: e51b7714ef11ba55e69db6cadbb4fd1bda676a00
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
  opencode v2.1.4
version-extracted: v2.1.4
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
  # scanned 11 file(s), 1199921202 byte(s) under /home/agent/.local/lib/node_modules/@opencode/cli
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
  ANY_KEY__
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
  ERR_MYSQL_MISSING_AUTH_DATAERR_MYSQL_AUTHENTICATION_FAILEDERR_MYSQL_FAILED_TO_ENCRYPT_PASSWORDERR_MYSQL_INVALID_PUBLIC_KEYERR_MYSQL_PUBLIC_KEY_RETRIEVAL_NOT_ALLOWEDERR_MYSQL_UNSUPPORTED_AUTH_PLUGINERR_MYSQL_UNSUPPORTED_PROTOCOL_VERSIONERR_MYSQL_OVERFLOWERR_MYSQL_QUERY_CANCELLEDERR_MYSQL_WRONG_NUMBER_OF_PARAMETERS_PROVIDEDERR_MYSQL_UNSUPPORTED_COLUMN_TYPEERR_MYSQL_INVALID_AUTH_SWITCH_REQUESTERR_MYSQL_INVALID_RESULT_ROWERR_MYSQL_INVALID_BINARY_VALUEERR_MYSQL_INVALID_ENCODED_INTEGERERR_MYSQL_INVALI  [line truncated]
  ERR_POSTGRES_AUTHENTICATION_FAILED_PBKDF2
  ERR_POSTGRES_INVALID_BACKEND_KEY_DATA
  ERR_POSTGRES_INVALID_SERVER_KEY
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
  A /home/agent/.npm/_cacache/content-v2/sha512/01
  A /home/agent/.npm/_cacache/content-v2/sha512/01/ba
  A /home/agent/.npm/_cacache/content-v2/sha512/01/ba/6e4a7bcd79f971705bbf0c7eb55f2900917255f810fc3f5973f9d25221387fcb93c05fe7e85578ce922e29c74d0d36485c26a1d259b12f95a085598bc0fc
  A /home/agent/.npm/_cacache/content-v2/sha512/04
  A /home/agent/.npm/_cacache/content-v2/sha512/04/17
  A /home/agent/.npm/_cacache/content-v2/sha512/04/17/11d12d64f8e76135d1cbe960a510cd9daf67138a75f4d3f473e041768de72328ae36c45307f455416f5ebaa241194db618f5e9238e09d1194747856e5e71
  A /home/agent/.npm/_cacache/content-v2/sha512/1b
  A /home/agent/.npm/_cacache/content-v2/sha512/1b/84
  A /home/agent/.npm/_cacache/content-v2/sha512/1b/84/a713f3ce522596b58e8f730b75cf64c82c66ab49dd49d3f043d9fc945516b97d3f80b9556ada854982e81eb1582e8c5831d22bf0a22155b1e1b0057d7a4c
  A /home/agent/.npm/_cacache/content-v2/sha512/23
  A /home/agent/.npm/_cacache/content-v2/sha512/23/85
  A /home/agent/.npm/_cacache/content-v2/sha512/23/85/6fa9078b2a79a3795d765b3999ac1bc29a363afbb24805b8639f420a51bab341f0a64142040c7c5c5ece1b409d748de96a43d9b27feebecdabdd9f5b696f
  A /home/agent/.npm/_cacache/content-v2/sha512/2f
  A /home/agent/.npm/_cacache/content-v2/sha512/2f/a3
  A /home/agent/.npm/_cacache/content-v2/sha512/2f/a3/607d077ddccc34a9682c06ed0bdfd0c233f1b80979f4f21e0f983467d4952091eb2a91c3ee4b2d0a960152c9f9188b2c84944632f0328212be89387434d3
  A /home/agent/.npm/_cacache/content-v2/sha512/3c
  A /home/agent/.npm/_cacache/content-v2/sha512/3c/bc
  A /home/agent/.npm/_cacache/content-v2/sha512/3c/bc/460e9d0eb7e97be1aae1ab85a6835b3860f28413302d066a1b6eb5e6911258668caa6aeb98752228249e67e071ca6a9619933938a178c4b2907d647b6277
  A /home/agent/.npm/_cacache/content-v2/sha512/61
  A /home/agent/.npm/_cacache/content-v2/sha512/61/9a
  A /home/agent/.npm/_cacache/content-v2/sha512/61/9a/1b6015a149d3adbc94844d7bc9831722b6174706d37e3ad7f524b2dd4bcac09888049984687ae92d788ec77d4a17d6b9e5b0dad80d01efabfa475957e75a
  A /home/agent/.npm/_cacache/content-v2/sha512/72
  A /home/agent/.npm/_cacache/content-v2/sha512/72/0c
  A /home/agent/.npm/_cacache/content-v2/sha512/72/0c/bfd261765e49aaf1d780a48fd7caeaaddd7563c7c1cff069c291492db418f885dffbf890834b4d863807713b753cda4c11c89656f47c048a1995050bbb4f
  A /home/agent/.npm/_cacache/content-v2/sha512/8f
  A /home/agent/.npm/_cacache/content-v2/sha512/8f/27
  A /home/agent/.npm/_cacache/content-v2/sha512/8f/27/63764476c0f1454099bc522143349dfe409af7edc081483f99ee09ac88d023f9e4911eb555367706a10db02b290f2557079076d16e255fce6c09d2d263d5
  A /home/agent/.npm/_cacache/content-v2/sha512/9a
  A /home/agent/.npm/_cacache/content-v2/sha512/9a/bc
  A /home/agent/.npm/_cacache/content-v2/sha512/9a/bc/79f57a51057e160baf782324441ddacc830b796d4fd4b6e46ddb1a789699ac4c6f2b72a917d6e8dcbd22f1d71586d4cf63907a8433fdf33dcd6ab5c87e56
  A /home/agent/.npm/_cacache/content-v2/sha512/9e
  A /home/agent/.npm/_cacache/content-v2/sha512/9e/cd
  A /home/agent/.npm/_cacache/content-v2/sha512/9e/cd/6a355f022b839de39a4fd17b0d9acc8bd823b78efa7e88a4162be5bcb887e8d786a63ea6ee6c2186f499d28cf9a92ee4b15f47188d4dfcf762aa61e983ac
  A /home/agent/.npm/_cacache/content-v2/sha512/a3
  A /home/agent/.npm/_cacache/content-v2/sha512/a3/c1
  A /home/agent/.npm/_cacache/content-v2/sha512/a3/c1/09d19b13e1dcc328dfb49a91b610a6cd1b3272a7e175fbe02fdbc51179155f884bfe377e1e6449cdfee9e488ca1eaca1da77a5ba222ee4e2e9139cf36635
  A /home/agent/.npm/_cacache/content-v2/sha512/b0
  A /home/agent/.npm/_cacache/content-v2/sha512/b0/ce
  A /home/agent/.npm/_cacache/content-v2/sha512/b0/ce/6203fd2ee9f8c92adcb99a05cf3f3c69e630dfb77dc5cadf908c808291b632240cbfecaaff563a262fe15503c144e34f8c8a7e6b7eb3e58586e8b3fea864
  A /home/agent/.npm/_cacache/content-v2/sha512/b2
  A /home/agent/.npm/_cacache/content-v2/sha512/b2/4c
  A /home/agent/.npm/_cacache/content-v2/sha512/b2/4c/f08f8ada64088364a5a0b359a36f72916462b55a57c1af81848c05c06b7a90c67b7eb40674ace6d4e13d767e823df67d5efb908e8b8887865d2a3a41eac8
  A /home/agent/.npm/_cacache/content-v2/sha512/be
  A /home/agent/.npm/_cacache/content-v2/sha512/be/d1
  A /home/agent/.npm/_cacache/content-v2/sha512/be/d1/c56bfd5fad2ca5a71d5c651bb398cd9e7075a7e9807c70dbe1c3c79628fbd216f0e7724657ba00db8b72c9603f531a3471d94e89468ef6e6ac2ab703147d
  A /home/agent/.npm/_cacache/content-v2/sha512/ce
  A /home/agent/.npm/_cacache/content-v2/sha512/ce/94
  A /home/agent/.npm/_cacache/content-v2/sha512/ce/94/2ac0a5c51cc078430f37b79c93f7910f578e9e674eba5158e6eb0dd63cff91465e4d41ab0b02415d05f42c97cea25ca7e27bb1cd3b1f2b039b1e66308091
  A /home/agent/.npm/_cacache/content-v2/sha512/d3
  A /home/agent/.npm/_cacache/content-v2/sha512/d3/f1
  A /home/agent/.npm/_cacache/content-v2/sha512/d3/f1/f8c110c05b8840e4858de38d17c03ad10136ef4f7727a3e514b2a1729d3bb44de1973400e32b60197ef84d4dfc89695c826bebfb56107eafb821b8c6b911
  A /home/agent/.npm/_cacache/content-v2/sha512/fc
  A /home/agent/.npm/_cacache/content-v2/sha512/fc/6f
  A /home/agent/.npm/_cacache/content-v2/sha512/fc/6f/0019e376cc631af3d406636f26d01f4ed665202b6d5a648588fb2d1bc851dc3559196f0608965ad8f22c7f0c80adbfcb74cb92ac1000dec4cc1f98256291
  A /home/agent/.npm/_cacache/index-v5
  A /home/agent/.npm/_cacache/index-v5/2b
  A /home/agent/.npm/_cacache/index-v5/2b/1b
  A /home/agent/.npm/_cacache/index-v5/2b/1b/0c9622df0e8a899a5033b8c363f97d16b249f6fe51364d4c1ed795c3c32e
  A /home/agent/.npm/_cacache/index-v5/2c
  A /home/agent/.npm/_cacache/index-v5/2c/34
  A /home/agent/.npm/_cacache/index-v5/2c/34/cbdb396ceb673e1154a4eefdbbb4c6fdcc5c743d3e5d82a65b3ec6aed9b8
  A /home/agent/.npm/_cacache/index-v5/30
  A /home/agent/.npm/_cacache/index-v5/30/65
  A /home/agent/.npm/_cacache/index-v5/30/65/87f83501ea5bc70f5086011d5658d014cdb058c3ce19914f085879453f4a
  A /home/agent/.npm/_cacache/index-v5/31
  A /home/agent/.npm/_cacache/index-v5/31/b9
  A /home/agent/.npm/_cacache/index-v5/31/b9/6f4765b4224cd87e72db0fc90b9753fb93d45bc031435734154a050ba0b3
  A /home/agent/.npm/_cacache/index-v5/3f
  A /home/agent/.npm/_cacache/index-v5/3f/b5
  A /home/agent/.npm/_cacache/index-v5/3f/b5/768f077b96e18c9b12ce33096ce1fcff6a8970754561449cb62722e93644
  A /home/agent/.npm/_cacache/index-v5/51
  A /home/agent/.npm/_cacache/index-v5/51/ab
  A /home/agent/.npm/_cacache/index-v5/51/ab/4044a4cf44a9c74ecf852f2be03f0e34db57e5712328bbee52d6d523da85
  A /home/agent/.npm/_cacache/index-v5/89
  A /home/agent/.npm/_cacache/index-v5/89/a0
  A /home/agent/.npm/_cacache/index-v5/89/a0/bfa29b6f6dd6d076e5be806eebca32586b46f4a7adcba9ab76bfc03e7ad1
  A /home/agent/.npm/_cacache/index-v5/ad
  A /home/agent/.npm/_cacache/index-v5/ad/a0
  A /home/agent/.npm/_cacache/index-v5/ad/a0/12100d721273755db112241301dc2706902c4935ccd229956104de4e4fe7
  A /home/agent/.npm/_cacache/index-v5/ba
  A /home/agent/.npm/_cacache/index-v5/ba/90
  A /home/agent/.npm/_cacache/index-v5/ba/90/64812f284beb2b8c4701724562ca0d6bfcffff2df823665d9da695e7e688
  A /home/agent/.npm/_cacache/index-v5/bf
  A /home/agent/.npm/_cacache/index-v5/bf/53
  A /home/agent/.npm/_cacache/index-v5/bf/53/c643f047d385e390e4b7b4d29f5957a6bb84e9e3fdf1fa5f09c8fef4450c
  A /home/agent/.npm/_cacache/index-v5/d6
  A /home/agent/.npm/_cacache/index-v5/d6/e9
  A /home/agent/.npm/_cacache/index-v5/d6/e9/b19cf592f47249031b9cddb122db22785b2ec310b717fb089d7efb4b0059
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
  A /home/agent/.npm/_cacache/index-v5/f4/c5
  A /home/agent/.npm/_cacache/index-v5/f4/c5/11cd946eabff23dd99d3f5509f37bcff5270e518d3e9721e89851153f29f
  A /home/agent/.npm/_cacache/index-v5/fb
  A /home/agent/.npm/_cacache/index-v5/fb/82
  A /home/agent/.npm/_cacache/index-v5/fb/82/5df1718069e939cc0241a011755c5af560f9bb680f9c61593c9c9017a8e7
  A /home/agent/.npm/_cacache/tmp
  A /home/agent/.npm/_logs
  A /home/agent/.npm/_logs/2026-10-25T00_23_01_172Z-debug-0.log
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
  A /tmp/.bun-1001-ebbfe5f112d1e949.so
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
    [truncated: more than 300 lines]
