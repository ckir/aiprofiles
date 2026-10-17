custody: ci
run-id: 39024332187
run-url: https://github.com/ckir/aiprofiles/actions/runs/39024332187
harness-commit: 16737cb79a0cde5bb698fae7784e69b9eeb6b551
---
install:
  npm install --global @continuedev/cli
exit-codes:
  install 0
  version 0
  help 0
  behaviour-flagfile---config 0
  candidate-1 0
  candidate-2 0
  candidate-3 0
  candidate-4 0
probe-exit: 0
version:
  1.5.47
version-extracted: 1.5.47
help:
  Usage: cn [options] [command] [prompt]
  
  Continue CLI - AI-powered development assistant. Starts an interactive session
  by default, use -p/--print for non-interactive output.
  
  Arguments:
    prompt                    Optional prompt to send to the assistant
  
  Options:
    -v, --version             Display version number
    --config <path>           Configuration file for the assistant (can be a file
                              path or hub slug)
    --org <slug>              Organization slug to use for this session (supported
                              only in headless mode)
    --readonly                Start in plan mode (read-only tools)
    --auto                    Start in auto mode (all tools allowed)
    --verbose                 Enable verbose logging
    --beta-status-tool        Enable beta status tool
    --rule <rule>             Add a rule (can be a file path, hub slug, or string
                              content). Can be specified multiple times. (default:
                              [])
    --mcp <slug>              Add an MCP server from the hub (slug in format
                              'owner/package'). Can be specified multiple times.
                              (default: [])
    --model <slug>            Add a model from the hub (slug in format
                              'owner/package'). Can be specified multiple times.
                              (default: [])
    --prompt <prompt>         Add to the initial user message (can be a file path,
                              hub slug, or string content). Can be specified
                              multiple times. (default: [])
    --allow <tool>            Allow specified tool (overrides default policies).
                              Can be specified multiple times. (default: [])
    --ask <tool>              Ask for permission before using specified tool
                              (overrides default policies). Can be specified
                              multiple times. (default: [])
    --exclude <tool>          Exclude specified tool from use (overrides default
                              policies). Can be specified multiple times.
                              (default: [])
    --agent <slug>            Load agent file from the hub (slug in format
                              'owner/package')
    -p, --print               Print response and exit (useful for pipes)
    --format <format>         Output format for headless mode (json). Only works
                              with -p/--print flag.
    --silent                  Strip <think></think> tags and excess whitespace
                              from output. Only works with -p/--print flag.
    --resume                  Resume from last session
    --fork <sessionId>        Fork from an existing session ID
    --beta-subagent-tool      Enable beta Subagent tool for invoking subagents
    -h, --help                display help for command
  
  Commands:
    ls [options]              List recent chat sessions and select one to resume
    checks [action] [pr-url]  Show CI check statuses for a PR
    review [options]          Run AI-powered reviews on your changes
strings:
  # /home/agent/.local/bin/cn -> /home/agent/.local/lib/node_modules/@continuedev/cli/dist/cn.js
  # scanned 564 file(s), 82660784 byte(s) under /home/agent/.local/lib/node_modules/@continuedev/cli
  documented CONTINUE_API_KEY: present
  documented ANTHROPIC_API_KEY: present
  documented OPENAI_API_KEY: present
  #
  # variable-shaped tokens that name a credential or a location, documented or not
  AFTER_DOCTYPE_PUBLIC_KEYWORD
  AFTER_DOCTYPE_SYSTEM_KEYWORD
  AI_GATEWAY_API_KEY
  ALL_SPECIAL_TOKENS
  ANTHROPIC_API_KEY
  ANTHROPIC_AUTH_TOKEN
  ANTHROPIC_TOKEN_MULTIPLIER
  ANY_KEY
  API_KEY
  API_KEY_AUTH
  API_KEY_WITH_CREDENTIALS
  APPLY_UNIQUE_TOKEN
  ATTR_MESSAGING_KAFKA_MESSAGE_KEY
  ATTR_MESSAGING_RABBITMQ_DESTINATION_ROUTING_KEY
  AT_KEYWORDS
  AUTH
  AUTHCODE_EXPIRED
  AUTHOR
  AUTHORITY
  AUTHORITY_REF
  AUTHORIZATION_CODE_CHALLENGE_METHOD
  AUTHORIZATION_CODE_RESPONSE_TYPE
  AUTHORIZED
  AUTHORS
  AUTH_CONFIG_PATH
  AUTH_HEADER
  AUTH_SCOPES
  AUTH_TYPE_UNSPECIFIED
  AWS_ACCESS_KEY
  AWS_ACCESS_KEY_ID
  AWS_AUTH_SCHEME_PREFERENCE
  AWS_BEARER_TOKEN_
  AWS_CONFIG_FILE
  AWS_CONTAINER_AUTHORIZATION_TOKEN
  AWS_CONTAINER_AUTHORIZATION_TOKEN_FILE
  AWS_CONTAINER_CREDENTIALS_FULL_URI
  AWS_CONTAINER_CREDENTIALS_RELATIVE_URI
  AWS_CREDENTIAL_EXPIRATION
  AWS_CREDENTIAL_SCOPE
  AWS_DEFAULTS_MODE_CONFIG
  AWS_PROFILE
  AWS_SECRET_ACCESS_KEY
  AWS_SECRET_KEY
  AWS_SESSION_TOKEN
  AWS_SHARED_CREDENTIALS_FILE
  AWS_WEB_IDENTITY_TOKEN_FILE
  BAGGAGE_KEY
  BAGGAGE_KEY_PAIR_SEPARATOR
  BARE_KEY
  BLOCK_AS_IMPLICIT_KEY
  BUILT_IN_KEYWORDS
  CERTIFICATE_CONFIGURATION_ENV_VARIABLE
  CLASS_KEYWORDS
  CLOUDSDK_CONFIG
  CLOUDSDK_CONFIG_DIRECTORY
  CL_AND_O_TOKEN_SPLIT_PATTERN
  COLOR_KEY
  COMMON_KEYWORDS
  COMPACTION_PROMPT_TOKENS
  CONFIG
  CONFIGURATION
  CONFIG_AV1_DECODER
  CONFIG_ENDPOINT_MODE_NAME
  CONFIG_ENDPOINT_NAME
  CONFIG_ENDPOINT_URL
  CONFIG_MAX_ATTEMPTS
  CONFIG_PATH
  CONFIG_PREFIX_SEPARATOR
  CONFIG_RETRY_MODE
  CONFIG_THREE_PASS
  CONFIG_USE_DUALSTACK_ENDPOINT
  CONFIG_USE_FIPS_ENDPOINT
  CONTEXTUAL_KEYWORDS
  CONTINUE_API_KEY
  CPP_KEYWORDS
  CREDENTIALS_CODE
  CREDENTIALS_ENV_VARS
  CREDENTIALS_ENV_VARS_STS_WEB_ID_TOKEN
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
  CREDENTIALS_STS_ASSUME_ROLE_WEB_ID
  CREDENTIAL_QUERY_PARAM
  C_KEYWORDS
  DB_CASSANDRA_KEYSPACE
  DCONFIG_TUNE_BUTTERAUGLI
  DCONFIG_TUNE_VMAF
  DEBUG_AUTH
  DEEPSEEK_API_KEY
  DEFAULT_AWS_REGIONAL_CREDENTIAL_VERIFICATION_URL
  DEFAULT_CONFIG_TS_CONTENTS
  DEFAULT_KEYWORD_SCOPE
  DEFAULT_MAX_TOKENS_RATIO
  DEFAULT_MAX_TOKEN_LIFETIME_SECS_
  DEFAULT_OAUTH_SCOPE
  DEFAULT_PROFILE
  DEFAULT_TOKEN_LIFESPAN
  DEFAULT_TOKEN_URL
  DESCENDANT_TOKEN
  DISCORD_KEY
  DOM_KEY_LOCATION_LEFT
  DOM_KEY_LOCATION_NUMPAD
  DOM_KEY_LOCATION_RIGHT
  DOM_KEY_LOCATION_STANDARD
  DOTENV_KEY
  DOTTED_KEY
  DOT_KEYWORD
  DUPLICATE_KEY
  ENDPOINT_CONFIG_OPTIONS
  ENDPOINT_MODE_CONFIG_OPTIONS
  ENV_CMDS_AUTH_TOKEN
  ENV_CONFIG_PATH
  ENV_CREDENTIALS_PATH
  ENV_CREDENTIAL_SCOPE
  ENV_KEY
  ENV_PROFILE
  ENV_SECRET
  ENV_TOKEN_FILE
  ERR_INVALID_HTTP_TOKEN
  EXTERNAL_ACCOUNT_AUTHORIZED_USER_TYPE
  FLEXIBLE_DESCENDANT_TOKEN
  FONTCONFIG_FILE
  FONTCONFIG_PATH
  FONTCONFIG_SYSROOT
  FONTCONFIG_USE_MMAP
  GATEWAY_AUTH_METHOD_HEADER
  GEMINI_API_KEY
  GEMINI_TOKEN_MULTIPLIER
  GITHUB_TOKEN
  GITLAB_TOKEN
  GIT_AUTHOR_EMAIL
  GIT_AUTHOR_NAME
  GLIBCONFIG_H__
  GLOBAL_OPENTELEMETRY_API_KEY
  GOOGLE_API_CERTIFICATE_CONFIG
  GOOGLE_API_KEY
  GOOGLE_API_KEY_HEADER
  GOOGLE_APPLICATION_CREDENTIALS
  GOOGLE_EXTERNAL_ACCOUNT_TOKEN_TYPE
  GOOGLE_GENERATIVE_AI_API_KEY
  GOOGLE_REVOKE_TOKEN_URL
  GOOGLE_SERVICE_ACCOUNT_AUTH
  GOOGLE_TOKEN_INFO_URL
  GOOGLE_TOKEN_URL
  GREPTILE_TOKEN
  G_ASK_PASSWORD_ANONYMOUS_SUPPORTED
  G_ASK_PASSWORD_NEED_DOMAIN
  G_ASK_PASSWORD_NEED_PASSWORD
  G_ASK_PASSWORD_NEED_USERNAME
  G_ASK_PASSWORD_SAVING_SUPPORTED
  G_ASK_PASSWORD_TCRYPT
  G_CREDENTIALS_TYPE_APPLE_XUCRED
  G_CREDENTIALS_TYPE_FREEBSD_CMSGCRED
  G_CREDENTIALS_TYPE_INVALID
  G_CREDENTIALS_TYPE_LINUX_UCRED
  G_CREDENTIALS_TYPE_NETBSD_UNPCBID
  G_CREDENTIALS_TYPE_OPENBSD_SOCKPEERCRED
  G_CREDENTIALS_TYPE_SOLARIS_UCRED
  G_CREDENTIALS_TYPE_WIN32_PID
  G_DBUS_CALL_FLAGS_ALLOW_INTERACTIVE_AUTHORIZATION
  G_DBUS_CONNECTION_FLAGS_AUTHENTICATION_ALLOW_ANONYMOUS
  G_DBUS_CONNECTION_FLAGS_AUTHENTICATION_CLIENT
  G_DBUS_CONNECTION_FLAGS_AUTHENTICATION_REQUIRE_SAME_USER
  G_DBUS_CONNECTION_FLAGS_AUTHENTICATION_SERVER
  G_DBUS_COOKIE_SHA1_KEYRING_DIR
  G_DBUS_COOKIE_SHA1_KEYRING_DIR_IGNORE_PERMISSION
  G_DBUS_ERROR_AUTH_FAILED
  G_DBUS_ERROR_SPAWN_CONFIG_INVALID
  G_DBUS_MESSAGE_FLAGS_ALLOW_INTERACTIVE_AUTHORIZATION
  G_DBUS_SERVER_FLAGS_AUTHENTICATION_ALLOW_ANONYMOUS
  G_DBUS_SERVER_FLAGS_AUTHENTICATION_REQUIRE_SAME_USER
  G_DRIVE_START_STOP_TYPE_PASSWORD
  G_IO_ERROR_PROXY_AUTH_FAILED
  G_IO_ERROR_PROXY_NEED_AUTH
  G_PASSWORD_SAVE_FOR_SESSION
  G_PASSWORD_SAVE_NEVER
  G_PASSWORD_SAVE_PERMANENTLY
  G_SETTINGS_BIND_DEFAULT
  G_SETTINGS_BIND_GET
  G_SETTINGS_BIND_GET_NO_CHANGES
  G_SETTINGS_BIND_INVERT_BOOLEAN
    [truncated: more than 200 lines]
baseline-flagfile---config:
  # /home/agent/probe-target
  d /home/agent/probe-target
  f /home/agent/probe-target/config.yaml
  # /home/agent/.continue
  d /home/agent/.continue
  d /home/agent/.continue/logs
  f /home/agent/.continue/logs/cn.log
delta-flagfile---config:
  (no change: nothing was created, moved or removed at the watched locations)
candidates:
  1: @none
  2: 
  3: # nothing
  4: {}
container-delta:
  C /etc
  C /home/agent
  C /home
  A /home/agent/.continue
  A /home/agent/.continue/logs
  A /home/agent/.continue/logs/cn.log
  A /home/agent/.local
  A /home/agent/.local/bin
  A /home/agent/.local/bin/cn
  A /home/agent/.local/lib
  A /home/agent/.local/lib/node_modules
  A /home/agent/.local/lib/node_modules/@continuedev
  A /home/agent/.local/lib/node_modules/@continuedev/cli
  A /home/agent/.local/lib/node_modules/@continuedev/cli/BUILD.md
  A /home/agent/.local/lib/node_modules/@continuedev/cli/CHANGELOG.md
  A /home/agent/.local/lib/node_modules/@continuedev/cli/README.md
  A /home/agent/.local/lib/node_modules/@continuedev/cli/build.mjs
  A /home/agent/.local/lib/node_modules/@continuedev/cli/demo.expect
  A /home/agent/.local/lib/node_modules/@continuedev/cli/dist
  A /home/agent/.local/lib/node_modules/@continuedev/cli/dist/cn.js
  A /home/agent/.local/lib/node_modules/@continuedev/cli/dist/index.js
  A /home/agent/.local/lib/node_modules/@continuedev/cli/dist/index.js.map
  A /home/agent/.local/lib/node_modules/@continuedev/cli/dist/meta.json
  A /home/agent/.local/lib/node_modules/@continuedev/cli/dist/xhr-sync-worker.js
  A /home/agent/.local/lib/node_modules/@continuedev/cli/docs
  A /home/agent/.local/lib/node_modules/@continuedev/cli/docs/artifact-uploads.md
  A /home/agent/.local/lib/node_modules/@continuedev/cli/docs/storage-sync.md
  A /home/agent/.local/lib/node_modules/@continuedev/cli/media
  A /home/agent/.local/lib/node_modules/@continuedev/cli/media/demo.gif
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/.bin
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/.bin/js-yaml
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/@img
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/@img/sharp-libvips-linux-x64
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/@img/sharp-libvips-linux-x64/README.md
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/@img/sharp-libvips-linux-x64/lib
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/@img/sharp-libvips-linux-x64/lib/glib-2.0
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/@img/sharp-libvips-linux-x64/lib/glib-2.0/include
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/@img/sharp-libvips-linux-x64/lib/glib-2.0/include/glibconfig.h
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/@img/sharp-libvips-linux-x64/lib/index.js
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/@img/sharp-libvips-linux-x64/lib/libvips-cpp.so.42
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/@img/sharp-libvips-linux-x64/package.json
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/@img/sharp-libvips-linux-x64/versions.json
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/@img/sharp-linux-x64
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/@img/sharp-linux-x64/LICENSE
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/@img/sharp-linux-x64/README.md
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/@img/sharp-linux-x64/lib
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/@img/sharp-linux-x64/lib/sharp-linux-x64.node
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/@img/sharp-linux-x64/package.json
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/argparse
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/argparse/CHANGELOG.md
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/argparse/LICENSE
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/argparse/README.md
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/argparse/argparse.js
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/argparse/lib
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/argparse/lib/sub.js
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/argparse/lib/textwrap.js
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/argparse/package.json
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/fdir
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/fdir/LICENSE
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/fdir/README.md
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/fdir/dist
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/fdir/dist/index.cjs
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/fdir/dist/index.d.cts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/fdir/dist/index.d.mts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/fdir/dist/index.mjs
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/fdir/package.json
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/find-up
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/find-up/index.d.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/find-up/index.js
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/find-up/license
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/find-up/package.json
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/find-up/readme.md
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/fzf
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/fzf/README.md
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/fzf/dist
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/fzf/dist/fzf.es.js
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/fzf/dist/fzf.umd.js
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/fzf/dist/types
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/fzf/dist/types/algo.d.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/fzf/dist/types/char.d.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/fzf/dist/types/extended.d.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/fzf/dist/types/finders.d.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/fzf/dist/types/main.d.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/fzf/dist/types/matchers.d.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/fzf/dist/types/normalize.d.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/fzf/dist/types/numerics.d.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/fzf/dist/types/pattern.d.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/fzf/dist/types/runes.d.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/fzf/dist/types/slab.d.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/fzf/dist/types/tiebreakers.d.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/fzf/dist/types/types.d.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/fzf/license.txt
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/fzf/package.json
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/js-yaml
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/js-yaml/LICENSE
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/js-yaml/README.md
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/js-yaml/bin
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/js-yaml/bin/js-yaml.js
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/js-yaml/dist
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/js-yaml/dist/js-yaml.js
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/js-yaml/dist/js-yaml.js.map
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/js-yaml/dist/js-yaml.min.js
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/js-yaml/dist/js-yaml.min.js.map
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/js-yaml/dist/js-yaml.mjs
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/js-yaml/dist/js-yaml.mjs.map
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/js-yaml/index.js
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/js-yaml/lib
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/js-yaml/lib/common.js
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/js-yaml/lib/dumper.js
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/js-yaml/lib/exception.js
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/js-yaml/lib/index_vite_proxy.tmp.mjs
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/js-yaml/lib/loader.js
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/js-yaml/lib/schema
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/js-yaml/lib/schema/core.js
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/js-yaml/lib/schema/default.js
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/js-yaml/lib/schema/failsafe.js
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/js-yaml/lib/schema/json.js
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/js-yaml/lib/schema.js
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/js-yaml/lib/snippet.js
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/js-yaml/lib/type
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/js-yaml/lib/type/binary.js
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/js-yaml/lib/type/bool.js
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/js-yaml/lib/type/float.js
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/js-yaml/lib/type/int.js
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/js-yaml/lib/type/map.js
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/js-yaml/lib/type/merge.js
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/js-yaml/lib/type/null.js
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/js-yaml/lib/type/omap.js
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/js-yaml/lib/type/pairs.js
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/js-yaml/lib/type/seq.js
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/js-yaml/lib/type/set.js
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/js-yaml/lib/type/str.js
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/js-yaml/lib/type/timestamp.js
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/js-yaml/lib/type.js
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/js-yaml/package.json
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/locate-path
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/locate-path/index.d.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/locate-path/index.js
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/locate-path/license
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/locate-path/package.json
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/locate-path/readme.md
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/p-limit
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/p-limit/index.d.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/p-limit/index.js
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/p-limit/license
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/p-limit/package.json
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/p-limit/readme.md
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/p-locate
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/p-locate/index.d.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/p-locate/index.js
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/p-locate/license
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/p-locate/package.json
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/p-locate/readme.md
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/unicorn-magic
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/unicorn-magic/default.d.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/unicorn-magic/default.js
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/unicorn-magic/license
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/unicorn-magic/node.d.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/unicorn-magic/node.js
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/unicorn-magic/package.json
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/unicorn-magic/readme.md
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/yocto-queue
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/yocto-queue/index.d.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/yocto-queue/index.js
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/yocto-queue/license
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/yocto-queue/package.json
  A /home/agent/.local/lib/node_modules/@continuedev/cli/node_modules/yocto-queue/readme.md
  A /home/agent/.local/lib/node_modules/@continuedev/cli/package.json
  A /home/agent/.local/lib/node_modules/@continuedev/cli/scripts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/scripts/install.ps1
  A /home/agent/.local/lib/node_modules/@continuedev/cli/scripts/install.sh
  A /home/agent/.local/lib/node_modules/@continuedev/cli/smoke-test.mjs
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/CLIPlatformClient.test.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/CLIPlatformClient.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/__mocks__
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/__mocks__/auth
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/__mocks__/auth/uriUtils.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/__mocks__/auth/workos.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/__mocks__/commands
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/__mocks__/commands/commands.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/__mocks__/config.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/__mocks__/configLoader.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/__mocks__/hooks
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/__mocks__/hooks/useService.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/__mocks__/logging.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/__mocks__/permissions
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/__mocks__/permissions/permissionsYamlLoader.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/__mocks__/permissions/precedenceResolver.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/__mocks__/services
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/__mocks__/services/BaseService.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/__mocks__/services/ServiceContainer.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/__mocks__/services/ServiceContainerContext.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/__mocks__/services/index.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/__mocks__/systemMessage.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/__mocks__/util
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/__mocks__/util/logger.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/__tests__
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/__tests__/CtrlCProcessHandling.test.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/args.test.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/asciiArt.test.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/asciiArt.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/auth
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/auth/authEnv.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/auth/uriUtils.test.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/auth/uriUtils.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/auth/workos.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/commands
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/commands/BaseCommandOptions.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/commands/chat.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/commands/checks.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/commands/commands.integration.test.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/commands/commands.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/commands/devbox-entrypoint.md
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/commands/init.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/commands/ls.test.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/commands/ls.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/commands/review
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/commands/review/ReviewProgress.tsx
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/commands/review/diffContext.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/commands/review/renderReport.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/commands/review/resolveReviews.test.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/commands/review/resolveReviews.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/commands/review/reviewWorker.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/commands/review/worktree.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/commands/review.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/commands/serve.helpers.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/commands/serve.initialPrompt.test.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/commands/serve.test.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/commands/serve.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/commands/stripThinkTags.test.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/compaction.infiniteLoop.test.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/compaction.pruneLastMessage.test.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/compaction.test.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/compaction.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/config.test.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/config.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/configLoader.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/constants
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/constants/session.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/continueSDK.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/e2e
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/e2e/basic-commands.test.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/e2e/config-switching.test.tsx
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/e2e/git-branch-display.test.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/e2e/headless-anthropic-api-key.test.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/e2e/headless-dynamic-responses.test.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/e2e/headless-json-format.test.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/e2e/headless-minimal.test.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/e2e/headless-missing-prompt.test.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/e2e/headless-mock-llm.test.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/e2e/headless-org-flag.test.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/e2e/headless-permission-error.test.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/e2e/headless-silent-flag.test.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/e2e/headless-simple.test.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/e2e/headless-tool-calls.test.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/e2e/local-config-switching.test.tsx
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/e2e/pipe-input-tui.test.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/e2e/resume-flag.test.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/e2e/spec.md
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/env.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/environment
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/environment/environmentHandler.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/fileTools.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/flags
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/flags/flagProcessor.test.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/flags/flagProcessor.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/flags/flagValidator.test.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/flags/flagValidator.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/hooks
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/hooks/HookService.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/hooks/fireHook.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/hooks/hookConfig.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/hooks/hookRunner.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/hooks/hooks.test.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/hooks/types.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/hooks/useService.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/hubLoader.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/index.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/infoScreen.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/init.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/integration
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/integration/model-persistence-e2e.test.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/integration/model-persistence-unauthenticated.test.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/integration/model-persistence-user-flow.test.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/integration/model-persistence.test.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/integration/rule-duplication.test.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/logger.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/logging.test.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/logging.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/mcp-flag.integration.test.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/messageConversion.test.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/onboarding.test.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/onboarding.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/permissions
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/permissions/README.md
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/permissions/defaultPolicies.test.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/permissions/defaultPolicies.ts
  A /home/agent/.local/lib/node_modules/@continuedev/cli/src/permissions/headlessPermissions.integration.test.ts
    [truncated: more than 300 lines]
