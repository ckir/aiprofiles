custody: ci
run-id: 40146391660
run-url: https://github.com/ckir/aiprofiles/actions/runs/40146391660
harness-commit: e51b7714ef11ba55e69db6cadbb4fd1bda676a00
---
install:
  npm install --global @google/gemini-cli
exit-codes:
  install 0
  version 0
  help 0
  behaviour-env-GEMINI_CLI_HOME 0
probe-exit: 0
version:
  0.65.0
version-extracted: 0.65.0
help:
  Usage: gemini [options] [command]
  
  Gemini CLI - Defaults to interactive mode. Use -p/--prompt for non-interactive (headless) mode.
  
  Commands:
    gemini mcp                   Manage MCP servers
    gemini extensions <command>  Manage Gemini CLI extensions.  [aliases: extension]
    gemini skills <command>      Manage agent skills.  [aliases: skill]
    gemini hooks <command>       Manage Gemini CLI hooks.  [aliases: hook]
    gemini gemma                 Manage local Gemma model routing
    gemini [query..]             Launch Gemini CLI  [default]
  
  Positionals:
    query  Initial prompt. Runs in interactive mode by default; use -p/--prompt for non-interactive.
  
  Options:
    -d, --debug                     Run in debug mode (open debug console with F12)  [boolean] [default: false]
    -m, --model                     Model  [string]
    -p, --prompt                    Run in non-interactive (headless) mode with the given prompt. Appended to input on stdin (if any).  [string]
    -i, --prompt-interactive        Execute the provided prompt and continue in interactive mode  [string]
        --skip-trust                Trust the current workspace for this session.  [boolean] [default: false]
    -w, --worktree                  Start Gemini in a new git worktree. If no name is provided, one is generated automatically.  [string]
    -s, --sandbox                   Run in sandbox?  [boolean]
    -y, --yolo                      Automatically accept all actions (aka YOLO mode, see https://www.youtube.com/watch?v=xvFZjo5PgG0 for more details)?  [boolean] [default: false]
        --approval-mode             Set the approval mode: default (prompt for approval), auto_edit (auto-approve edit tools), yolo (auto-approve all tools), plan (read-only mode)  [string] [choices: "default", "auto_edit", "yolo", "plan"]
        --policy                    Additional policy files or directories to load (comma-separated or multiple --policy)  [array]
        --admin-policy              Additional admin policy files or directories to load (comma-separated or multiple --admin-policy)  [array]
        --acp                       Starts the agent in ACP mode  [boolean]
        --experimental-acp          Starts the agent in ACP mode (deprecated, use --acp instead)  [boolean]
        --allowed-mcp-server-names  Allowed MCP server names  [array]
        --allowed-tools             [DEPRECATED: Use Policy Engine instead See https://geminicli.com/docs/core/policy-engine] Tools that are allowed to run without confirmation  [array]
    -e, --extensions                A list of extensions to use. If not provided, all extensions are used.  [array]
    -l, --list-extensions           List all available extensions and exit.  [boolean]
    -r, --resume                    Resume a previous session. Use "latest" for most recent or index number (e.g. --resume 5)  [string]
        --session-file              Load a session from a JSON file  [string]
        --session-id                Start a new session with a manually provided UUID.  [string]
        --list-sessions             List available sessions for the current project and exit.  [boolean]
        --delete-session            Delete a session by index number (use --list-sessions to see available sessions).  [string]
        --include-directories       Additional directories to include in the workspace (comma-separated or multiple --include-directories)  [array]
        --screen-reader             Enable screen reader mode for accessibility.  [boolean]
    -o, --output-format             The format of the CLI output.  [string] [choices: "text", "json", "stream-json"]
        --raw-output                Disable sanitization of model output (e.g. allow ANSI escape sequences). WARNING: This can be a security risk if the model output is untrusted.  [boolean]
        --accept-raw-output-risk    Suppress the security warning when using --raw-output.  [boolean]
    -v, --version                   Show version number  [boolean]
    -h, --help                      Show help  [boolean]
strings:
  # /home/agent/.local/bin/gemini -> /home/agent/.local/lib/node_modules/@google/gemini-cli/bundle/gemini.js
  # scanned 825 file(s), 102571893 byte(s) under /home/agent/.local/lib/node_modules/@google/gemini-cli
  documented GEMINI_API_KEY: present
  documented GOOGLE_API_KEY: present
  documented GOOGLE_APPLICATION_CREDENTIALS: present
  documented GEMINI_CLI_HOME: present
  #
  # variable-shaped tokens that name a credential or a location, documented or not
  ADDRESS_HOME_ADMIN_LEVEL2
  ADDRESS_HOME_APT
  ADDRESS_HOME_APT_NUM
  ADDRESS_HOME_APT_TYPE
  ADDRESS_HOME_BETWEEN_STREETS
  ADDRESS_HOME_BETWEEN_STREETS_1
  ADDRESS_HOME_BETWEEN_STREETS_2
  ADDRESS_HOME_BETWEEN_STREETS_OR_LANDMARK
  ADDRESS_HOME_CITY
  ADDRESS_HOME_COUNTRY
  ADDRESS_HOME_DEPENDENT_LOCALITY
  ADDRESS_HOME_FLOOR
  ADDRESS_HOME_HOUSE_NUMBER
  ADDRESS_HOME_LANDMARK
  ADDRESS_HOME_LINE1
  ADDRESS_HOME_LINE2
  ADDRESS_HOME_OVERFLOW
  ADDRESS_HOME_STATE
  ADDRESS_HOME_STREET_ADDRESS
  ADDRESS_HOME_STREET_LOCATION
  ADDRESS_HOME_STREET_NAME
  ADDRESS_HOME_SUBPREMISE
  ADDRESS_HOME_ZIP
  AGENT_CONFIG_FIELDS
  AGENT_LABEL_KEY
  ALLOWED_TELEMETRY_ENV_KEYS
  ANY_KEY
  API_KEY
  API_KEY_AUTH
  API_KEY_WITH_CREDENTIALS
  ASCII_TOKENS_PER_CHAR
  ATTR_ASPNETCORE_USER_IS_AUTHENTICATED
  AT_KEYWORDS
  AUTH
  AUTHENTICATED_EXTENDED_CARD_NOT_CONFIGURED
  AUTHOR
  AUTHORED_DEPLOYED_GROUPING
  AUTHORIZATION_CODE_CHALLENGE_METHOD
  AUTHORIZATION_CODE_RESPONSE_TYPE
  AUTHORIZED
  AUTHORS
  AUTH_CODE
  AUTH_ENV_VAR_WHITELIST
  AUTH_REQUIRED_MSG
  AUTH_TYPE_UNSPECIFIED
  AWS_ACCESS_KEY_ID
  AWS_SECRET_ACCESS_KEY
  AWS_SESSION_TOKEN
  B3_DEBUG_FLAG_KEY
  BAGGAGE_KEY
  BAGGAGE_KEY_PAIR_SEPARATOR
  BARE_KEY
  BASE_SEATBELT_PROFILE
  BBKEYND
  BENIGN_SUBCOMMAND_TOKENS
  BINARY_INJECTION_KEY
  BKEYEQCACQ
  BLOCK_AS_IMPLICIT_KEY
  BROWSER_HOME
  BROWSER_PROFILE_DIR
  BROWSER_PROFILE_DIR2
  BUILTIN_SEATBELT_PROFILES
  BUILTIN_SEATBELT_PROFILE_CONTENTS
  BUILT_IN_KEYWORDS
  CERTIFICATE_CONFIGURATION_ENV_VARIABLE
  CHANNEL_ARGS_CONFIG_SELECTOR_KEY
  CHROME_CONFIG_HOME
  CI_TOKEN
  CLASS_KEYWORDS
  CLIENT_KEY
  CLOUDSDK_CONFIG
  CLOUDSDK_CONFIG_DIRECTORY
  CLOUD_RUN_CONFIG_ENV
  COMMON_KEYWORDS
  COMPRESSION_FAILED_INFLATED_TOKEN_COUNT
  COMPRESSION_FAILED_TOKEN_COUNT_ERROR
  COMPRESSION_FUNCTION_RESPONSE_TOKEN_BUDGET
  CONFIGURABLE
  CONFIGURABLE_FUNCTION_NAME
  CONFIGURABLE_LENGTH
  CONFIGURATION_NAME
  CONFIG_ERROR_INTERACTIVE_MODE
  CONFIG_ERROR_MODE_REQUIRED
  CONFIG_ERROR_UNKNOWN_OPTION
  CONFIG_REGEX
  CONNECTION_TOKEN_CHARS
  CONTEXTUAL_KEYWORDS
  CONVERSATIONAL_PARAM_KEYS
  CORECLR_PROFILER_PATH
  CPP_KEYWORDS
  CPU_PROFILE
  CREDENTIAL
  CRUX_API_KEY
  C_KEYWORDS
  DATA_PRODUCER_KEY
  DEBUG_AUTH
  DEFAULT_ACTIVITY_CONFIG
  DEFAULT_API_KEY_ENTRY
  DEFAULT_AWS_REGIONAL_CREDENTIAL_VERIFICATION_URL
  DEFAULT_CHARS_PER_TOKEN
  DEFAULT_COMPRESSION_TOKEN_THRESHOLD
  DEFAULT_CONFIG
  DEFAULT_CONFIG_PARAMETERS
  DEFAULT_FAST_ACK_MODEL_CONFIG_KEY
  DEFAULT_KEYWORD_SCOPE
  DEFAULT_LOGGER_CONFIG
  DEFAULT_LOGGER_CONFIGURATOR
  DEFAULT_MAX_TOKEN_LIFETIME_SECS_
  DEFAULT_MIN_PRUNABLE_TOKENS_THRESHOLD
  DEFAULT_MODEL_CONFIGS
  DEFAULT_OAUTH_SCOPE
  DEFAULT_PROJECT_ID_TOKEN
  DEFAULT_TOKEN_LIFESPAN
  DEFAULT_TOKEN_LIMIT
  DEFAULT_TOKEN_URL
  DIAGNOSTIC_INFO_KEY
  DOTENV_KEY
  DOTTED_KEY
  DOT_KEYWORD
  DUPLICATE_KEY
  EBCAEBAQEQEQALIANBKEYNBSADQS9GDQM
  EMPTY_TOKENS
  ENHANCED_KEY
  ERROR_NO_TOKEN
  ERR_INVALID_HTTP_TOKEN
  EVENT_API_KEY_UPDATED
  EVENT_CLI_CONFIG
  EVENT_KEYCHAIN_AVAILABILITY
  EVENT_TOKEN_STORAGE_INITIALIZATION
  EXTENSIONS_CONFIG_FILENAME
  EXTENSION_SETTINGS_FILENAME
  EXTERNAL_ACCOUNT_AUTHORIZED_USER_TYPE
  EXTERNAL_API_KEY
  EXTERNAL_SERVICE_TOKEN
  FATAL_AUTHENTICATION_ERROR
  FATAL_CONFIG_ERROR
  FILE_PATH_KEYS
  FOCUS_UI_ENABLED_STATE_KEY
  FORCE_GENERIC_KEYBINDING_HINTS
  GEMINI_API_KEY
  GEMINI_API_KEY_AUTH_MECHANISM
  GEMINI_CLI_API_RESPONSE_CACHED_TOKEN_COUNT
  GEMINI_CLI_API_RESPONSE_INPUT_TOKEN_COUNT
  GEMINI_CLI_API_RESPONSE_OUTPUT_TOKEN_COUNT
  GEMINI_CLI_API_RESPONSE_THINKING_TOKEN_COUNT
  GEMINI_CLI_API_RESPONSE_TOOL_TOKEN_COUNT
  GEMINI_CLI_AUTH_OVERRIDE
  GEMINI_CLI_AUTH_TYPE
  GEMINI_CLI_COMPRESSION_TOKENS_AFTER
  GEMINI_CLI_COMPRESSION_TOKENS_BEFORE
  GEMINI_CLI_HOME
  GEMINI_CLI_IDE_AUTH_TOKEN
  GEMINI_CLI_KEYCHAIN_AVAILABLE
  GEMINI_CLI_KEY_UNKNOWN
  GEMINI_CLI_START_SESSION_API_KEY_ENABLED
  GEMINI_CLI_SYSTEM_SETTINGS_PATH
  GEMINI_CLI_TOKEN_STORAGE_FORCED
  GEMINI_CLI_TOKEN_STORAGE_TYPE
  GEMINI_CLI_TOOL_OUTPUT_MASKING_TOKENS_AFTER
  GEMINI_CLI_TOOL_OUTPUT_MASKING_TOKENS_BEFORE
  GEMINI_CLI_TOOL_OUTPUT_MASKING_TOTAL_PRUNABLE_TOKENS
  GEMINI_CLI_USER_SETTINGS
  GEMINI_CONTEXT_CALIBRATE_TOKEN_CALCULATIONS
  GEMINI_DEFAULT_AUTH_TYPE
  GEMINI_TELEMETRY_USE_CLI_AUTH
  GEMMA_4_TOKEN_LIMIT
  GEN_AI_CLIENT_TOKEN_USAGE
  GEN_AI_USAGE_INPUT_TOKENS
  GEN_AI_USAGE_OUTPUT_TOKENS
  GITHUB_PERSONAL_ACCESS_TOKEN
  GITHUB_TOKEN
  GIT_AUTHOR_EMAIL
  GIT_AUTHOR_NAME
  GIT_CONFIG_
  GIT_CONFIG_COUNT
  GIT_CONFIG_GLOBAL
  GIT_CONFIG_KEY_
  GIT_CONFIG_KEY_0
  GIT_CONFIG_KEY_1
  GIT_CONFIG_KEY_2
  GIT_CONFIG_KEY_3
  GIT_CONFIG_KEY_4
  GIT_CONFIG_KEY_5
  GIT_CONFIG_KEY_6
  GIT_CONFIG_NOSYSTEM
  GIT_CONFIG_PARAMETERS
  GIT_CONFIG_SYSTEM
  GIT_CONFIG_VALUE_
  GIT_CONFIG_VALUE_0
  GIT_CONFIG_VALUE_1
  GIT_CONFIG_VALUE_2
  GIT_CONFIG_VALUE_3
    [truncated: more than 200 lines]
baseline-env-GEMINI_CLI_HOME:
  # /home/agent/probe-target
  d /home/agent/probe-target
  # /home/agent/.gemini
  d /home/agent/.gemini
  f /home/agent/.gemini/projects.json.0fb4a987-94e8-489c-a903-49016553c1e3.tmp
  f /home/agent/.gemini/projects.json.17e85aaa-86d8-44b6-8109-e16707bf94ae.tmp
  f /home/agent/.gemini/projects.json.825d2a6d-e04b-410d-bbeb-7f1d7cd83f71.tmp
  f /home/agent/.gemini/projects.json.b09d4d1b-1dce-428e-9660-d79966fffe82.tmp
delta-env-GEMINI_CLI_HOME:
  + d /home/agent/probe-target/.gemini
  + d /home/agent/probe-target/.gemini/history
  + d /home/agent/probe-target/.gemini/history/work
  + d /home/agent/probe-target/.gemini/tmp
  + d /home/agent/probe-target/.gemini/tmp/work
  + f /home/agent/probe-target/.gemini/history/work/.project_root
  + f /home/agent/probe-target/.gemini/projects.json
  + f /home/agent/probe-target/.gemini/tmp/work/.project_root
container-delta:
  C /etc
  C /home/agent
  C /home
  A /home/agent/.cache
  A /home/agent/.cache/node-gyp
  A /home/agent/.cache/node-gyp/24.21.0
  A /home/agent/.cache/node-gyp/24.21.0/include
  A /home/agent/.cache/node-gyp/24.21.0/include/node
  A /home/agent/.cache/node-gyp/24.21.0/include/node/common.gypi
  A /home/agent/.cache/node-gyp/24.21.0/include/node/config.gypi
  A /home/agent/.cache/node-gyp/24.21.0/include/node/cppgc
  A /home/agent/.cache/node-gyp/24.21.0/include/node/cppgc/allocation.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/cppgc/common.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/cppgc/cross-thread-persistent.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/cppgc/custom-space.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/cppgc/default-platform.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/cppgc/explicit-management.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/cppgc/garbage-collected.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/cppgc/heap-consistency.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/cppgc/heap-handle.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/cppgc/heap-state.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/cppgc/heap-statistics.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/cppgc/heap.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/cppgc/internal
  A /home/agent/.cache/node-gyp/24.21.0/include/node/cppgc/internal/api-constants.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/cppgc/internal/atomic-entry-flag.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/cppgc/internal/base-page-handle.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/cppgc/internal/caged-heap-local-data.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/cppgc/internal/caged-heap.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/cppgc/internal/compiler-specific.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/cppgc/internal/conditional-stack-allocated.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/cppgc/internal/finalizer-trait.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/cppgc/internal/gc-info.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/cppgc/internal/logging.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/cppgc/internal/member-storage.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/cppgc/internal/name-trait.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/cppgc/internal/persistent-node.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/cppgc/internal/pointer-policies.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/cppgc/internal/write-barrier.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/cppgc/liveness-broker.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/cppgc/macros.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/cppgc/member.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/cppgc/name-provider.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/cppgc/object-size-trait.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/cppgc/persistent.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/cppgc/platform.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/cppgc/prefinalizer.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/cppgc/process-heap-statistics.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/cppgc/sentinel-pointer.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/cppgc/source-location.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/cppgc/testing.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/cppgc/trace-trait.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/cppgc/type-traits.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/cppgc/visitor.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/js_native_api.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/js_native_api_types.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/libplatform
  A /home/agent/.cache/node-gyp/24.21.0/include/node/libplatform/libplatform-export.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/libplatform/libplatform.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/libplatform/v8-tracing.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/node.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/node_api.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/node_api_types.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/node_buffer.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/node_object_wrap.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/node_version.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/aes.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/crypto
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/crypto/buildinf.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/include
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/include/crypto
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/include/crypto/bn_conf.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/include/crypto/dso_conf.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/include/internal
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/include/internal/param_names.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/include/openssl
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/include/openssl/asn1.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/include/openssl/asn1t.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/include/openssl/bio.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/include/openssl/cmp.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/include/openssl/cms.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/include/openssl/comp.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/include/openssl/conf.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/include/openssl/configuration.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/include/openssl/core_names.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/include/openssl/crmf.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/include/openssl/crypto.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/include/openssl/ct.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/include/openssl/err.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/include/openssl/ess.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/include/openssl/fipskey.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/include/openssl/lhash.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/include/openssl/ocsp.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/include/openssl/opensslv.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/include/openssl/pkcs12.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/include/openssl/pkcs7.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/include/openssl/safestack.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/include/openssl/srp.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/include/openssl/ssl.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/include/openssl/ui.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/include/openssl/x509.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/include/openssl/x509_acert.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/include/openssl/x509_vfy.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/include/openssl/x509v3.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/include/progs.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/providers
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/providers/common
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/providers/common/include
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/providers/common/include/prov
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/providers/common/include/prov/der_digests.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/providers/common/include/prov/der_dsa.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/providers/common/include/prov/der_ec.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/providers/common/include/prov/der_ecx.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/providers/common/include/prov/der_ml_dsa.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/providers/common/include/prov/der_rsa.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/providers/common/include/prov/der_slh_dsa.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/providers/common/include/prov/der_sm2.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm/providers/common/include/prov/der_wrap.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/crypto
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/crypto/buildinf.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/include
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/include/crypto
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/include/crypto/bn_conf.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/include/crypto/dso_conf.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/include/internal
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/include/internal/param_names.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/include/openssl
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/include/openssl/asn1.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/include/openssl/asn1t.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/include/openssl/bio.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/include/openssl/cmp.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/include/openssl/cms.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/include/openssl/comp.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/include/openssl/conf.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/include/openssl/configuration.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/include/openssl/core_names.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/include/openssl/crmf.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/include/openssl/crypto.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/include/openssl/ct.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/include/openssl/err.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/include/openssl/ess.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/include/openssl/fipskey.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/include/openssl/lhash.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/include/openssl/ocsp.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/include/openssl/opensslv.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/include/openssl/pkcs12.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/include/openssl/pkcs7.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/include/openssl/safestack.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/include/openssl/srp.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/include/openssl/ssl.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/include/openssl/ui.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/include/openssl/x509.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/include/openssl/x509_acert.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/include/openssl/x509_vfy.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/include/openssl/x509v3.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/include/progs.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/providers
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/providers/common
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/providers/common/include
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/providers/common/include/prov
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/providers/common/include/prov/der_digests.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/providers/common/include/prov/der_dsa.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/providers/common/include/prov/der_ec.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/providers/common/include/prov/der_ecx.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/providers/common/include/prov/der_ml_dsa.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/providers/common/include/prov/der_rsa.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/providers/common/include/prov/der_slh_dsa.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/providers/common/include/prov/der_sm2.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/asm_avx2/providers/common/include/prov/der_wrap.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/crypto
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/crypto/buildinf.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/include
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/include/crypto
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/include/crypto/bn_conf.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/include/crypto/dso_conf.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/include/internal
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/include/internal/param_names.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/include/openssl
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/include/openssl/asn1.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/include/openssl/asn1t.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/include/openssl/bio.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/include/openssl/cmp.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/include/openssl/cms.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/include/openssl/comp.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/include/openssl/conf.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/include/openssl/configuration.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/include/openssl/core_names.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/include/openssl/crmf.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/include/openssl/crypto.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/include/openssl/ct.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/include/openssl/err.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/include/openssl/ess.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/include/openssl/fipskey.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/include/openssl/lhash.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/include/openssl/ocsp.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/include/openssl/opensslv.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/include/openssl/pkcs12.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/include/openssl/pkcs7.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/include/openssl/safestack.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/include/openssl/srp.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/include/openssl/ssl.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/include/openssl/ui.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/include/openssl/x509.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/include/openssl/x509_acert.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/include/openssl/x509_vfy.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/include/openssl/x509v3.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/include/progs.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/providers
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/providers/common
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/providers/common/include
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/providers/common/include/prov
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/providers/common/include/prov/der_digests.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/providers/common/include/prov/der_dsa.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/providers/common/include/prov/der_ec.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/providers/common/include/prov/der_ecx.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/providers/common/include/prov/der_ml_dsa.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/providers/common/include/prov/der_rsa.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/providers/common/include/prov/der_slh_dsa.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/providers/common/include/prov/der_sm2.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86/no-asm/providers/common/include/prov/der_wrap.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/crypto
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/crypto/buildinf.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/include
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/include/crypto
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/include/crypto/bn_conf.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/include/crypto/dso_conf.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/include/internal
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/include/internal/param_names.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/include/openssl
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/include/openssl/asn1.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/include/openssl/asn1t.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/include/openssl/bio.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/include/openssl/cmp.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/include/openssl/cms.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/include/openssl/comp.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/include/openssl/conf.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/include/openssl/configuration.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/include/openssl/core_names.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/include/openssl/crmf.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/include/openssl/crypto.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/include/openssl/ct.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/include/openssl/err.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/include/openssl/ess.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/include/openssl/fipskey.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/include/openssl/lhash.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/include/openssl/ocsp.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/include/openssl/opensslv.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/include/openssl/pkcs12.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/include/openssl/pkcs7.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/include/openssl/safestack.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/include/openssl/srp.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/include/openssl/ssl.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/include/openssl/ui.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/include/openssl/x509.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/include/openssl/x509_acert.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/include/openssl/x509_vfy.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/include/openssl/x509v3.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/include/progs.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/providers
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/providers/common
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/providers/common/include
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/providers/common/include/prov
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/providers/common/include/prov/der_digests.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/providers/common/include/prov/der_dsa.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/providers/common/include/prov/der_ec.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/providers/common/include/prov/der_ecx.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/providers/common/include/prov/der_ml_dsa.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/providers/common/include/prov/der_rsa.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/providers/common/include/prov/der_slh_dsa.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/providers/common/include/prov/der_sm2.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm/providers/common/include/prov/der_wrap.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm_avx2
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm_avx2/crypto
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm_avx2/crypto/buildinf.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm_avx2/include
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm_avx2/include/crypto
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm_avx2/include/crypto/bn_conf.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm_avx2/include/crypto/dso_conf.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm_avx2/include/internal
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm_avx2/include/internal/param_names.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm_avx2/include/openssl
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm_avx2/include/openssl/asn1.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm_avx2/include/openssl/asn1t.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm_avx2/include/openssl/bio.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm_avx2/include/openssl/cmp.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm_avx2/include/openssl/cms.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm_avx2/include/openssl/comp.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm_avx2/include/openssl/conf.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm_avx2/include/openssl/configuration.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm_avx2/include/openssl/core_names.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm_avx2/include/openssl/crmf.h
  A /home/agent/.cache/node-gyp/24.21.0/include/node/openssl/archs/BSD-x86_64/asm_avx2/include/openssl/crypto.h
    [truncated: more than 300 lines]
