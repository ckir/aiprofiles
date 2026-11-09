custody: ci
run-id: 42654070717
run-url: https://github.com/ckir/aiprofiles/actions/runs/42654070717
harness-commit: 96c66274332c4054bab8794f8715ccc2e33604e1
---
install:
  npm install --global @openai/codex
exit-codes:
  install 0
  version 0
  help 0
  behaviour-env-CODEX_HOME 0
probe-exit: 0
version:
  codex-cli 0.167.0
version-extracted: 0.167.0
help:
  Codex CLI
  
  If no subcommand is specified, options will be forwarded to the interactive CLI.
  
  Usage: codex [OPTIONS] [PROMPT]
         codex [OPTIONS] <COMMAND> [ARGS]
  
  Commands:
    agents            Open the agent command center to browse and switch sessions
    exec              Run Codex non-interactively [aliases: e]
    review            Run a code review non-interactively
    login             Manage login
    logout            Remove stored authentication credentials
    mcp               Manage external MCP servers for Codex
    plugin            Manage Codex plugins
    app-server        [experimental] Run the app server or related tooling
    remote-control    [experimental] Manage the app-server daemon with remote control enabled
    completion        Generate shell completion scripts
    update            Update Codex to the latest version
    doctor            Diagnose local Codex installation, config, auth, and runtime health
    sandbox           Run commands within a Codex-provided sandbox
    debug             Debugging tools
    apply             Apply the latest diff produced by Codex agent as a `git apply` to your local
                      working tree [aliases: a]
    resume            Resume a previous interactive session (picker by default; use --last to continue
                      the most recent)
    queue             Queue a message for an existing session
    archive           Archive a saved session by id or session name
    delete            Permanently delete a saved session by id or session name
    migrate-rollouts  Inspect or migrate legacy local sessions to paginated thread history
    migrate-nfs-db    [experimental] Migrate configured SQLite databases to DELETE mode in place
    unarchive         Unarchive a saved session by id or session name
    fork              Fork a previous interactive session (picker by default; use --last to fork the
                      most recent)
    cloud             [EXPERIMENTAL] Browse tasks from Codex Cloud and apply changes locally
    exec-server       [EXPERIMENTAL] Run the standalone exec-server service
    features          Inspect feature flags
    help              Print this message or the help of the given subcommand(s)
  
  Arguments:
    [PROMPT]
            Optional user prompt to start the session
  
  Options:
    -c, --config <key=value>
            Override a configuration value that would otherwise be loaded from `~/.codex/config.toml`.
            Use a dotted path (`foo.bar.baz`) to override nested values. The `value` portion is parsed
            as TOML. If it fails to parse as TOML, the raw string is used as a literal.
            
            Examples: - `-c model="o3"` - `-c 'sandbox_permissions=["disk-full-read-access"]'` - `-c
            shell_environment_policy.inherit=all`
  
        --enable <FEATURE>
            Enable a feature (repeatable). Equivalent to `-c features.<name>=true`
  
        --disable <FEATURE>
            Disable a feature (repeatable). Equivalent to `-c features.<name>=false`
  
        --remote <ADDR>
            Connect the TUI to a remote app server endpoint.
            
            Accepted forms: `ws://host:port`, `wss://host:port`, `unix://`, or `unix://PATH`.
  
        --remote-auth-token-env <ENV_VAR>
            Name of the environment variable containing the bearer token to send to a remote app
            server websocket
  
        --strict-config
            Error out when config.toml contains fields that are not recognized by this version of
            Codex
  
    -i, --image <FILE>...
            Optional image(s) to attach to the initial prompt
  
    -m, --model <MODEL>
            Model the agent should use
  
        --oss
            Use open-source provider
  
        --local-provider <OSS_PROVIDER>
            Specify which local provider to use (lmstudio or ollama). If not specified with --oss,
            will use config default or show selection
  
    -p, --profile <CONFIG_PROFILE_V2>
            Layer $CODEX_HOME/<name>.config.toml on top of the base user config
  
    -s, --sandbox <SANDBOX_MODE>
            Select the sandbox policy to use when executing model-generated shell commands
            
            [possible values: read-only, workspace-write, danger-full-access]
  
        --approve-for-me
            Route approval requests through automatic review using the workspace-write sandbox
  
        --dangerously-bypass-approvals-and-sandbox
            Skip all confirmation prompts and execute commands without sandboxing. EXTREMELY
            DANGEROUS. Intended solely for running in environments that are externally sandboxed
  
        --dangerously-bypass-hook-trust
            Run enabled hooks without requiring persisted hook trust for this invocation. DANGEROUS.
            Intended only for automation that already vets hook sources
  
    -C, --cd <DIR>
            Tell the agent to use the specified directory as its working root
  
        --worktree
            Run the session in a new managed Git worktree
  
        --add-dir <DIR>
            Additional directories that should be writable alongside the primary workspace
  
    -a, --ask-for-approval <APPROVAL_POLICY>
            Configure when the model requires human approval before executing a command
  
            Possible values:
            - on-request: The model decides when to ask the user for approval
            - never:      Never ask for user approval Execution failures are immediately returned to
              the model
  
        --search
            Enable live web search. When enabled, the native Responses `web_search` tool is available
            to the model (no per‑call approval)
  
        --no-alt-screen
            Disable alternate screen mode
            
            Runs the TUI in inline mode, preserving terminal scrollback history.
  
        --no-daemon
            Run without the shared background server, even if it is already running
  
    -h, --help
            Print help (see a summary with '-h')
  
    -V, --version
            Print version
strings:
  # /home/agent/.local/bin/codex -> /home/agent/.local/lib/node_modules/@openai/codex/bin/codex.js
  # scanned 48 file(s), 458607228 byte(s) under /home/agent/.local/lib/node_modules/@openai/codex
  documented OPENAI_API_KEY: present
  documented OPENAI_BASE_URL: present
  documented CODEX_API_KEY: present
  #
  # variable-shaped tokens that name a credential or a location, documented or not
  AANAMEANYCAACDSCDNSKEYCNAMECSYNCDNSKEYDSHINFOHTTPSKEYMXNAPTRNSNSEC3NSEC3PARAMOPENPGPKEYOPTPTRRRSIGSIGSOASRVTXTU
  AANAMECAACDSCDNSKEYCNAMECSYNCDNSKEYDSHINFOHTTPSKEYMXNAPTRNSNSEC3NSEC3PARAMOPENPGPKEYPTRRRSIGSIGSOASRVTXTR
  ACTIONS_ID_TOKEN_REQUEST_URL
  ACTIONS_ID_TOKEN_REQUEST_URLHTTPS
  ACTIVE_WORKSPACENO_ACTIVE_WORKSPED_FOR_WORKSPACENOT_CONFIGURED_F
  AES_KEY
  AGENTS_MDCONFIGSKILLSPLUGINSMCP_SERVER_CONFIGSUBAGENTSHOOKSMEMORY
  AGENTS_MDCONFIGSKILLSPLUGINSMCP_SERVER_CONFIGSUBAGENTSHOOKSMEMORYE
  AGENTS_MDCONFIGSKILLSPLUGINSMCP_SERVER_CONFIGSUBAGENTSHOOKSMEMORYP
  ALGORITHMPASSWORD
  ALLOWED_INTERFACE_KEYS
  ALSA_CONFIG_DIR
  ALSA_CONFIG_PATH
  ANAMECDSCDNSKEYCNAMEDNSKEYDSHINFOHTTPSKEYNAPTRNSNSEC3NSEC3PARAMOPENPGPKEYPTRRRSIGSIGSOATXTN
  ANAMECNAMEACSYNCHINFOHTTPSMXNAPTROPENPGPKEYSOASRVSSHFPTXT
  ANAMECNAMECSYNCHINFOHTTPSNAPTRNSOPENPGPKEYPTRSOASSHFP
  ANYCDSCDNSKEYDNSKEYDSKEYNSEC3NSEC3PARAMRRSIGSIGC
  API_KEY
  API_KEYH
  API_KEYH1
  API_KEYH3N
  API_KEYH3O
  API_KEYH3V
  API_KEYL1
  API_KEYM1
  ATA_HOMEH
  AUTH
  AUTH1
  AUTHDATA
  AUTHORITY
  AUTHORITY1
  AUTHORITY_INFO_ACCESS
  AUTHORITY_KEYID
  AUTHORS
  AUTH_TOKEN_STDIN
  AUTH_TOKEN_UPDATES_STDINR
  AUTH_UPDATED
  AWS_ACCESS_KEY_ID
  AWS_ACCESS_KEY_IDAWS_SECRET_ACCESS_KEYAWS_SESSION_TOKENAWS_ACCOUNT_IDE
  AWS_ACCESS_KEY_IDAWS_SESSION_TOKENAWS_SECURITY_TOKENAWS_CA_BUNDLE
  AWS_AUTH_SCHEME_PREFERENCE
  AWS_BEARER_TOKEN_BEDROCK
  AWS_BEARER_TOKEN_BEDROCKAWS_ACCESS_KEY_IDAWS_SECRET_ACCESS_KEYN
  AWS_CONFIG_FILEAWS_SHARED_CREDENTIALS_FILE
  AWS_CONTAINER_AUTHORIZATION_TOKEN_FILEAWS_CONTAINER_AUTHORIZATION_TOKENECS
  AWS_CONTAINER_CREDENTIALS_RELATIVE_URIAWS_CONTAINER_CREDENTIALS_FULL_URI
  AWS_IGNORE_CONFIGURED_ENDPOINT_URLS
  AWS_PROFILE
  AWS_PROFILEAWS
  AWS_SECRET_ACCESS_KEY
  AWS_WEB_IDENTITY_TOKEN_FILEAWS_ROLE_ARNAWS_ROLE_SESSION_NAME
  BADVERSBADSIGBADKEYBADTIMEBADMODEBADNAMEBADALGBADCOOKIE
  BADVERSBADSIGBADKEYBADTIMEBADMODEBADNAMEBADALGBADCOOKIEI
  BARE_PUBKEY
  BAWS_BEARER_TOKEN_BEDROCKAWS_ACCESS_KEY_IDAWS_SECRET_ACCESS_KEYA
  BCONFIGURATION
  BEARER_TOKEN_ENV_VARO
  BRIPGREP_CONFIG_PATH
  CDSCDNSKEYDNSKEYDSKEYNSEC3NSEC3PARAMRRSIGSIG
  CHE_HOMEH
  CLIENT_EARLY_TRAFFIC_SECRET
  CLIENT_EARLY_TRAFFIC_SECRETCLIENT_HANDSHAKE_TRAFFIC_SECRETSERVER_HANDSHAKE_TRAFFIC_SECRETCLIENT_TRAFFIC_SECRET_0SERVER_TRAFFIC_SECRET_0EXPORTER_SECRET
  CLIENT_HANDSHAKE_TRAFFIC_SECRET
  CLIENT_SECRETOA
  CLIENT_TRAFFIC_SECRET_0
  CLIENT_TRAFFIC_SECRET_N
  CODEX_ACCESS_TOKEN
  CODEX_ACCESS_TOKENOPENAI_FEDERATION_RULE_IDOPENAI_IDENTITY_TOKEN_FILE
  CODEX_AGENT_IDENTITY_AUTHAPI_BASE_URLCODEX_AGENT_IDENTITY_JWKS_BASE_URL
  CODEX_AGENT_IDENTITY_JWKS_BASE_URLCODEX_AUTHAPI_BASE_URL
  CODEX_API_KEYCODEX_ACCESS_TOKEN
  CODEX_API_KEYOPENAI_API_KEYA
  CODEX_APPLY_PATCH_PRESERVE_LINE_ENDINGSCODEX_PERMISSION_PROFILE
  CODEX_AUTH
  CODEX_AUTHAPI_BASE_URL
  CODEX_CONNECTORS_TOKEN
  CODEX_EXEC_SERVER_NOISE_REGISTRY_URLCODEX_EXEC_SERVER_NOISE_ENVIRONMENT_IDCODEX_EXEC_SERVER_NOISE_AUTH_TOKENCODEX_EXEC_SERVER_NOISE_CHATGPT_ACCOUNT_ID
  CODEX_GITHUB_PERSONAL_ACCESS_TOKEN
  CODEX_HOME
  CODEX_HOMEC
  CODEX_HOMECODEX_CA_CERTIFICATESSL_CERT_FILEREQUESTS_CA_BUNDLECURL_CA_BUNDLENODE_EXTRA_CA_CERTSGIT_SSL_CAINFOCARGO_HTTP_CAINFOBUNDLE_SSL_CA_CERT
  CODEX_HOMECODEX_INSTALL_DEFER_SELECTIONCODEX_RELEASE
  CODEX_NETWORK_PROXY_BROKERED_CREDENTIALS
  CODEX_NETWORK_PROXY_CREDENTIAL_BROKER_ACTIVE
  CODEX_NETWORK_PROXY_CREDENTIAL_BROKER_ACTIVECODEX_NETWORK_PROXY_BROKERED_CREDENTIALS
  CODEX_NETWORK_PROXY_CREDENTIAL_BROKER_ACTIVECODEX_NETWORK_PROXY_SNAPSHOT_ORIGINAL_POSIX_ENVCODEX_NETWORK_PROXY_SNAPSHOT_BROKERED_UNSET_CODEX_NETWORK_PROXY_SNAPSHOT_BROKERED_VALUE_C
  CODEX_REFRESH_TOKEN_URL_OVERRIDE
  CODEX_REVOKE_TOKEN_URL_OVERRIDE
  CODEX_SHELL_SNAPSHOT_CONFIGURED_
  CODEX_SNAPSHOT_OVERRIDECODEX_NETWORK_PROXY_ACTIVECODEX_NETWORK_PROXY_CREDENTIAL_BROKER_ACTIVECODEX_NETWORK_PROXY_BROKERED_CREDENTIALSCODEX_NETWORK_ALLOW_LOCAL_BINDINGCODEX_NETWORK_PROXY_ATTRIBUTIONELECTRON_GET_USE_PROXYNODE_USE_ENV_PROXYHTTP_PROXYHTTPS_PROXY
  CODEX_SQLITE_HOME
  CODEX_TOOL_CALL_IDCODEX_EXEC_SERVER_NOISE_AUTH_TOKENNODE_REPL_AUTH_TOKENOPENAI_FEDERATION_RULE_IDOPENAI_IDENTITY_TOKEN_FILE
  CODEX_TUI_DISABLE_KEYBOARD_ENHANCEMENT
  COMPOSERARRANGERLYRICISTDIRECTORPRODUCERMIXED_BYKEYWORDSSYNOPSISSUBTITLELANGUAGESESSION
  CONFIG
  CONFIGURATION
  CONFIG_PROFILEL
  CONFIG_PROFILE_V2L
  CONFIG_SECCOMP
  CONFIG_SECCOMP_FILTER
  CREDENTIALS_FILEAWS_SHARED_CREDEE
  DEFAULT_FOREIGN_KEYS
  DEFAULT_PROFILEAWS_DEFAULT_PROF
  DH_PUBKEY
  DOCKER_HTTP_PROXNFIG_HTTPS_PROXYNPM_CONFIG_HTTPSONFIG_HTTP_PROXYNPM_CONFIG_HTTP_BUNDLE_HTTPS_PROXY
  DSA_PUBKEY
  EARLY_EXPORTER_SECRET
  ECRET_ACCESS_KEYAWS_SECRET_ACCES
  ECX_KEY
  EC_KEY
  EC_KEY_
  EC_KEY_O
  EC_PRIVATEKEY
  EC_PUBKEY
  EC_PUBLIC_KEY_P256_PKCS8_V1_TEMPLATE17
  EC_PUBLIC_KEY_P384_PKCS8_V1_TEMPLATE17
  ED25519_PUBKEY
  ED448_PUBKEY
  EMPTYAANAMEANYCAACDSCDNSKEYCNAMECSYNCDNSKEYDSHINFOHTTPSKEYMXNAPTRNSNSEC3NSEC3PARAMOPENPGPKEYOPTPTRRRSIGSIGSOASRVSSHFPTXTU
  ENTERPRISE_TOKENGH_ENTERPRISE_TOGITHUB_ENTERPRISREQUESTS_CA_BUNDLE
  ENUM_KEYS_CACHE_TYPE
  EPERMENOENTESRCHEINTREIOENXIOE2BIGENOEXECEBADFECHILDEAGAINENOMEMEACCESEFAULTENOTBLKEBUSYEEXISTEXDEVENODEVENOTDIREISDIREINVALENFILEEMFILEENOTTYETXTBSYEFBIGENOSPCESPIPEEROFSEMLINKEPIPEERANGEEDEADLKENAMETOOLONGENOLCKENOSYSENOTEMPTYELOOPENOMSGEIDRMECHRNGEL3HLTEL3RSTELNRNGEUNATCHENOCSIEL2HLTEBADEEBADREXFULLENOANOEBADRQCEBADSLTEBFONTENOSTRENODATAETIMEENOSRENONETENOPKGEREMOTEENOLINKESRMNTECOMMEPROTOEMULTIHOPEDOTDOTEBADMSGEOVERFLOWEBADFDEREMCHGELIBACCELIBBADELIBSCNELIBMAXEILSEQEUSERSEDESTADDRREQEPROTOTY  [line truncated]
  EVP_MAX_KEY_LENGTH
  EVP_PKEY
  EVP_PKEY2PKCS8
  EVP_PKEY_
  EVP_PKEY_CTX_
  EVP_PKEY_NONE
  EVP_RSA_PKEY_CTX_
  EVP_SKEY_
  EXECUTOR_CREDENTIAL_STDINR
  EXTENDED_KEY_USAGE
  FIG_HOMEH
  FIG_HOMEI3N
  FIG_HOMEL3B
  FINAL_SIZE_ERRORKEY_UPDATE_ERROR
  GH_TOKEN
  GH_TOKENH9
  GIO_USE_POWER_PROFILE_MONITOR
  GITHUB_TOKEN
  GITHUB_TOKENGH_ENTERPRISE_TOKENGITHUB_ENTERPRISE_TOKENOPENAI_BASE_URL
  GIT_AUTHOR_DATE
  GIT_AUTHOR_EMAILGIT_TRACE_PACKET
  GIT_AUTHOR_NAME
  GIT_CEILING_DIRECTORIESGIT_CONFIGGIT_CONFIG_PARAMETERSGIT_OBJECT_DIRECTORYGIT_DIRGIT_WORK_TREEGIT_IMPLICIT_WORK_TREEGIT_GRAFT_FILEGIT_INDEX_FILEGIT_NO_REPLACE_OBJECTSGIT_REPLACE_REF_BASEGIT_PREFIXGIT_COMMON_DIR
  GIT_COMMITTER_NAMEGIT_COMMITTER_EMAILGIT_AUTHOR_NAME
  GIT_COMMITTER_NAMEGIT_COMMITTER_EMAILGIT_CONFIG_NOSYSTEMGIT_HTTP_LOW_SPEED_LIMITGIT_HTTP_LOW_SPEED_TIMEGIT_HTTP_USER_AGENT
  GIT_CONFIGGIT_DISCOVERY_ACROSS_FILESYSTEMGIT_OBJECT_DIRECTORYGIT_COMMON_DIRGIT_CONFIG_SYSTEMGIT_CONFIG_NOSYSTEMGIT_CONFIG_PARAMETERSGIT_CONFIG_GLOBAL
  GIT_CONFIG_COUNT
  GIT_CONFIG_KEY_
  GIT_CONFIG_NOSYSTEMGIT_CONFIG_SYSTEMGIT_CONFIG_GLOBAL
  GIT_CONFIG_SYSTEMGIT_CONFIG_GLOBAL
  GIT_CONFIG_VALUE_
  GIT_HTTP_PROXY_AUTHMETHOD
  GIT_OPTIONAL_LOCKS0GIT_CEILING_DIRECTORIESGIT_COMMON_DIRGIT_CONFIGGIT_CONFIG_PARAMETERSGIT_DIRGIT_DISCOVERY_ACROSS_FILESYSTEMGIT_GRAFT_FILEGIT_IMPLICIT_WORK_TREEGIT_INDEX_FILEGIT_NAMESPACEGIT_OBJECT_DIRECTORYGIT_PREFIXGIT_REPLACE_REF_BASEGIT_WORK_TREE
  GIT_PAGERCODEX_PERMISSION_PROFILECODEX_VERSIONCODEX_APPLY_PATCH_PRESERVE_LINE_ENDINGSCODEX_PLUGIN_METRICS_OUTPUT
  GIX_CREDENTIALS_HELPER_STDERR
  GMTHOME
  GOOGLE_APPLICATION_CREDENTIALS
  GOOGLE_APPLICATION_CREDENTIALSE
  GSETTINGS_BACKEND
  GSETTINGS_SCHEMA_DIR
  GST_BUFFER_POOL_ACQUIRE_FLAG_KEY_UNIT
  GST_EVENT_RECONFIGURE
  GST_IS_ENCODING_CONTAINER_PROFILE
  GST_IS_ENCODING_PROFILE
  GST_IS_ENCODING_VIDEO_PROFILE
  GST_LIBRARY_ERROR_SETTINGS
  GST_NAVIGATION_EVENT_KEY_PRESS
  GST_NAVIGATION_EVENT_KEY_RELEASE
  GST_PAD_FLAG_NEED_RECONFIGURE
  GST_PAD_LINK_CHECK_NO_RECONFIGURE
  GST_RESOURCE_ERROR_NOT_AUTHORIZED
  GST_RESOURCE_ERROR_SETTINGS
  GST_RTP_PROFILE_AVP
  GST_RTP_PROFILE_AVPF
  GST_RTP_PROFILE_SAVP
  GST_RTP_PROFILE_SAVPF
  GST_RTP_PROFILE_UNKNOWN
  GST_SEEK_FLAG_KEY_UNIT
  GST_SEEK_FLAG_TRICKMODE_KEY_UNITS
  GST_SEGMENT_FLAG_TRICKMODE_KEY_UNITS
  GST_STREAM_ERROR_DECRYPT_NOKEY
  GST_VIDEO_CODEC_FRAME_FLAG_FORCE_KEYFRAME
  GST_VIDEO_CODEC_FRAME_FLAG_FORCE_KEYFRAME_HEADERS
  G_ASK_PASSWORD_ANONYMOUS_SUPPORTED
  G_ASK_PASSWORD_NEED_DOMAIN
  G_ASK_PASSWORD_NEED_PASSWORD
  G_ASK_PASSWORD_NEED_USERNAME
  G_ASK_PASSWORD_SAVING_SUPPORTED
  G_ASK_PASSWORD_TCRYPT
  G_CREDENTIALS_TYPE_APPLE_XUCRED
  G_CREDENTIALS_TYPE_FREEBSD_CMSGCRED
    [truncated: more than 200 lines]
baseline-env-CODEX_HOME:
  # /home/agent/probe-target
  d /home/agent/probe-target
  # /home/agent/.codex
  d /home/agent/.codex
  d /home/agent/.codex/tmp
  d /home/agent/.codex/tmp/arg0-linux
  d /home/agent/.codex/tmp/arg0-linux/codex-arg061l8VI
  f /home/agent/.codex/tmp/arg0-linux/codex-arg061l8VI/.lock
  l /home/agent/.codex/tmp/arg0-linux/codex-arg061l8VI/apply_patch
  l /home/agent/.codex/tmp/arg0-linux/codex-arg061l8VI/applypatch
  l /home/agent/.codex/tmp/arg0-linux/codex-arg061l8VI/codex-linux-sandbox
delta-env-CODEX_HOME:
  + d /home/agent/probe-target/tmp
  + d /home/agent/probe-target/tmp/arg0-linux
  + d /home/agent/probe-target/tmp/arg0-linux/codex-arg0MMcttu
  + f /home/agent/probe-target/tmp/arg0-linux/codex-arg0MMcttu/.lock
  + l /home/agent/probe-target/tmp/arg0-linux/codex-arg0MMcttu/apply_patch
  + l /home/agent/probe-target/tmp/arg0-linux/codex-arg0MMcttu/applypatch
  + l /home/agent/probe-target/tmp/arg0-linux/codex-arg0MMcttu/codex-linux-sandbox
container-delta:
  C /etc
  C /home/agent
  C /home
  A /home/agent/.codex
  A /home/agent/.codex/tmp
  A /home/agent/.codex/tmp/arg0-linux
  A /home/agent/.codex/tmp/arg0-linux/codex-arg061l8VI
  A /home/agent/.codex/tmp/arg0-linux/codex-arg061l8VI/.lock
  A /home/agent/.codex/tmp/arg0-linux/codex-arg061l8VI/apply_patch
  A /home/agent/.codex/tmp/arg0-linux/codex-arg061l8VI/applypatch
  A /home/agent/.codex/tmp/arg0-linux/codex-arg061l8VI/codex-linux-sandbox
  A /home/agent/.local
  A /home/agent/.local/bin
  A /home/agent/.local/bin/codex
  A /home/agent/.local/lib
  A /home/agent/.local/lib/node_modules
  A /home/agent/.local/lib/node_modules/@openai
  A /home/agent/.local/lib/node_modules/@openai/codex
  A /home/agent/.local/lib/node_modules/@openai/codex/README.md
  A /home/agent/.local/lib/node_modules/@openai/codex/bin
  A /home/agent/.local/lib/node_modules/@openai/codex/bin/codex.js
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/README.md
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/package.json
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/bin
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/bin/codex
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/bin/codex-code-mode-host
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-package.json
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-path
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-path/rg
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-resources
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-resources/bwrap
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-resources/voice
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-resources/voice/NOTICE.md
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-resources/voice/bin
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-resources/voice/bin/codex-voice-host
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-resources/voice/lib
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-resources/voice/lib/gstreamer-1.0
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-resources/voice/lib/gstreamer-1.0/libgstapp.so
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-resources/voice/lib/gstreamer-1.0/libgstaudioconvert.so
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-resources/voice/lib/gstreamer-1.0/libgstaudioresample.so
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-resources/voice/lib/gstreamer-1.0/libgstcoreelements.so
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-resources/voice/lib/gstreamer-1.0/libgstopus.so
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-resources/voice/lib/gstreamer-1.0/libgstrtp.so
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-resources/voice/lib/gstreamer-1.0/libgstrtpmanager.so
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-resources/voice/lib/libffi.so.8
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-resources/voice/lib/libgio-2.0.so.0
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-resources/voice/lib/libglib-2.0.so.0
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-resources/voice/lib/libgmodule-2.0.so.0
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-resources/voice/lib/libgobject-2.0.so.0
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-resources/voice/lib/libgstallocators-1.0.so.0
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-resources/voice/lib/libgstapp-1.0.so.0
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-resources/voice/lib/libgstaudio-1.0.so.0
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-resources/voice/lib/libgstbase-1.0.so.0
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-resources/voice/lib/libgstnet-1.0.so.0
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-resources/voice/lib/libgstpbutils-1.0.so.0
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-resources/voice/lib/libgstreamer-1.0.so.0
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-resources/voice/lib/libgstrtp-1.0.so.0
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-resources/voice/lib/libgsttag-1.0.so.0
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-resources/voice/lib/libgstvideo-1.0.so.0
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-resources/voice/lib/libintl.so.8
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-resources/voice/lib/libopus.so.0
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-resources/voice/lib/libpcre2-8.so.0
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-resources/voice/lib/libz.so.1
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-resources/voice/licenses
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-resources/voice/licenses/LGPL-2.1.txt
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-resources/voice/licenses/Opus.txt
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-resources/voice/licenses/PCRE2.md
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-resources/voice/licenses/libffi.txt
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-resources/voice/licenses/proxy-libintl.txt
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-resources/voice/licenses/sljit.txt
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-resources/voice/licenses/zlib.txt
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-resources/voice/manifest.json
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-resources/voice/runtime.json
  A /home/agent/.local/lib/node_modules/@openai/codex/node_modules/@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/codex-resources/voice/sources.json
  A /home/agent/.local/lib/node_modules/@openai/codex/package.json
  A /home/agent/.npm
  A /home/agent/.npm/_cacache
  A /home/agent/.npm/_cacache/content-v2
  A /home/agent/.npm/_cacache/content-v2/sha512
  A /home/agent/.npm/_cacache/content-v2/sha512/14
  A /home/agent/.npm/_cacache/content-v2/sha512/14/d7
  A /home/agent/.npm/_cacache/content-v2/sha512/14/d7/3b75a9df52783d4a9eb0f8587b9287cd1c766903a54cc8d296b16ad2dcec07972ebd832c1359e8aec2b50f5e9252868f037f8c6a46c1df1d13f38fbcd842
  A /home/agent/.npm/_cacache/content-v2/sha512/65
  A /home/agent/.npm/_cacache/content-v2/sha512/65/b3
  A /home/agent/.npm/_cacache/content-v2/sha512/65/b3/5a4268b16241d6a2ee70ba40b6d37eb31879f47cf2c69c6001703697fff4af892afa165d431679be9b1c1f69dc8af037660c295bf8d9f830fe693a19c817
  A /home/agent/.npm/_cacache/content-v2/sha512/d4
  A /home/agent/.npm/_cacache/content-v2/sha512/d4/54
  A /home/agent/.npm/_cacache/content-v2/sha512/d4/54/9081b4c8280753b9be4b9facf7e8d4f87970e60519229df8a0cf271b827f722d9749dd4764419aabd0dee39e5f3f0f5c714ff3f3dc597697b4b676645a14
  A /home/agent/.npm/_cacache/index-v5
  A /home/agent/.npm/_cacache/index-v5/67
  A /home/agent/.npm/_cacache/index-v5/67/35
  A /home/agent/.npm/_cacache/index-v5/67/35/41d81c92997638a9ccebb23019e1b5523284e23f03d8a23c4766a0e49ec0
  A /home/agent/.npm/_cacache/index-v5/68
  A /home/agent/.npm/_cacache/index-v5/68/55
  A /home/agent/.npm/_cacache/index-v5/68/55/5ad9506fe69fc99423224766c0ddd4f08db9ed23270e2ad8f69efc106774
  A /home/agent/.npm/_cacache/index-v5/f5
  A /home/agent/.npm/_cacache/index-v5/f5/dc
  A /home/agent/.npm/_cacache/index-v5/f5/dc/c4e113933dc9336af5ff67031ccabd30b60538026ed131d00e2087f9fd8a
  A /home/agent/.npm/_cacache/tmp
  A /home/agent/.npm/_logs
  A /home/agent/.npm/_logs/2026-11-09T16_23_42_003Z-debug-0.log
  A /home/agent/.npm/_update-notifier-last-checked
  A /home/agent/probe-target
  A /home/agent/probe-target/tmp
  A /home/agent/probe-target/tmp/arg0-linux
  A /home/agent/probe-target/tmp/arg0-linux/codex-arg0MMcttu
  A /home/agent/probe-target/tmp/arg0-linux/codex-arg0MMcttu/.lock
  A /home/agent/probe-target/tmp/arg0-linux/codex-arg0MMcttu/apply_patch
  A /home/agent/probe-target/tmp/arg0-linux/codex-arg0MMcttu/applypatch
  A /home/agent/probe-target/tmp/arg0-linux/codex-arg0MMcttu/codex-linux-sandbox
  C /home/probe
  A /home/probe/.probe-out
  A /home/probe/.probe-out/after-env-CODEX_HOME.txt
  A /home/probe/.probe-out/baseline-env-CODEX_HOME.txt
  A /home/probe/.probe-out/behaviour-env-CODEX_HOME.cmd
  A /home/probe/.probe-out/behaviour-env-CODEX_HOME.exit-code
  A /home/probe/.probe-out/behaviour-env-CODEX_HOME.txt
  A /home/probe/.probe-out/delta-env-CODEX_HOME.txt
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
  A /home/probe/.probe-state/pristine/1.tar
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
    [truncated: more than 300 lines]
