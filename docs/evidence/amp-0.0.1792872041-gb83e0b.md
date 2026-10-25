custody: ci
run-id: 40146391660
run-url: https://github.com/ckir/aiprofiles/actions/runs/40146391660
harness-commit: e51b7714ef11ba55e69db6cadbb4fd1bda676a00
---
install:
  npm install --global @ampcode/cli
exit-codes:
  install 0
  version 0
  help 0
  behaviour-flagfile---settings-file 0
  candidate-1 0
  candidate-2 0
  candidate-3 0
probe-exit: 0
version:
  0.0.1792872041-gb83e0b (released 2026-10-24T20:00:41.000Z, 4h ago)
version-extracted: 0.0.1792872041-gb83e0b
help:
  Amp CLI
  
  Usage: amp [options] [command]
  
  Commands:
  
    version                              Print the version number and exit
    logout                               Remove a saved Amp account
    account                              Manage saved Amp accounts
      list                               List saved accounts
      switch                             Choose the account for future CLI commands
    login                                Log in to Amp
    orb                                  Commands for Amp-managed orbs
      provider                           Run self-hosted orbs from inside your network by pulling lifecycle jobs from Amp
                                         over HTTPS
      bootstrap                          Connect this orb to its thread actor
      system-metrics                     Print historical orb system metrics for a thread URL or ID
      portal                             Register a portal for an existing process and print its URL; use "amp orb service
                                         start" for long-lived servers
        access                           Set portal access
      service                            [alias: services] Manage long-lived services (dev servers, watchers) as
                                         supervised processes that survive Amp CLI updates and restart on failure
        ensure                           Ensure every service declared in .amp/services.yaml is running and exposed at its
                                         portal URLs
        start                            Start an undeclared ad-hoc supervised service, e.g.: amp orb service start
                                         preview --command 'pnpm dev' --portal
        restart                          Restart a declared service from .amp/services.yaml or reload an existing ad-hoc
                                         service
        stop                             Stop a service
        status                           Show supervisor status for a service
        logs                             Show recent logs for a service
        list                             List orb services
      simulator                          Work with the iOS Simulator on macOS
        serve                            Serve one booted iOS Simulator as an interactive web page
      desktop                            Work with the orb Desktop shown in the Desktop tab of the thread
        ensure                           Start the Desktop and its Google Chrome if needed and print the Chrome DevTools
                                         endpoint for agent-browser --cdp
      restart-processes                  Refresh environment variables and restart all processes in an orb thread
      id-token                           Mint a short-lived OIDC token with a caller-chosen audience and print it to
                                         stdout
      redact-output                      Copy stdin to stdout with secrets redacted: known token formats, the thread's
                                         secret values, and the values of the environment variables named in
                                         AMP_SECRET_ENV_NAMES (comma-separated)
    portal                               Commands for accessing Amp portal URLs
      login-url                          Mint a single-use login URL (valid ~60s) that signs a non-browser client into an
                                         Amp portal URL; fetching it once sets the portal session cookie and redirects to
                                         the portal page
    net                                  Amp Net: send an orb’s outbound traffic through an exit node on your own machine
                                         (for example a residential connection)
      exit-node                          Run this machine as an exit node for your orbs: it appears in your runner list
                                         and any orb thread you own can route through it with `amp net route`
      route                              In an orb: send this orb’s traffic through one of your exit nodes (by ID or
                                         hostname). With no argument, list available exit nodes.
      status                             In an orb: show whether Amp Net is configured and which exit node is connected
      ping                               In an orb: measure the round trip from this orb to the connected exit node
      run                                In an orb: run one command with its network traffic sent through the exit node,
                                         without changing orb-wide configuration
    runner                               Inspect and change the amp --no-tui runners on this machine
      list                               List the running runners on this machine and the directories they serve
      dirs                               Change the directories a running runner serves, without restarting it
        add                              Start serving one or more directories; they are served again after the runner
                                         restarts
        remove                           Stop serving one or more directories added with `amp runner dirs add`
        list                             List the directories a running runner serves
      desktop                            Inspect or allow desktop sharing for --no-tui --desktop runners
        status                           Show desktop sharing readiness (macOS permissions, Linux programs)
        allow                            Request Screen Recording or Accessibility permission
    clone                                Clone an Amp-hosted Project, Personal Plugins or Skills repository, or Workspace
                                         Plugins or Skills repository
    git-setup                            Configure the Amp Git credential helper
    files                                Download and unpublish Amp files
      get                                Download a private Amp attachment
      delete                             Unpublish a public artifact you published, so its URL stops working
    top                                  Show live active threads
    last                                 [alias: l] Continue the last thread
    report-bug                           [alias: report, diagnostics] Report a bug with diagnostics from the amp --no-tui
                                         runner in this directory
    threads                              [alias: t, thread] Manage threads
      new                                [alias: n] Create a new thread
      continue                           [alias: c] Continue an existing thread
      list                               [alias: l, ls] List all threads
      usage                              Show usage information for a thread
      context                            [alias: analyze-context] Analyze the current context for a thread
      visibility                         [alias: v] Show or set default visibility for this repository
      search                             [alias: find] Search threads
      color                              Set a thread color
      label                              Add labels to a thread
      share                              [alias: s] Share a thread
        multiplayer                      Turn multiplayer on or off
          on                             Turn multiplayer on for the default duration (1 week)
          off                            Turn multiplayer off
          ttl                            Turn multiplayer on for a specific duration
      import                             Import conversations from other coding agents as read-only Amp threads
        detect                           Show which coding agents have conversations on this machine
        codex                            Import Codex conversations from this machine
        claude                           [alias: claude-code] Import Claude Code sessions from this machine
        opencode                         Import OpenCode sessions from this machine
      rename                             [alias: r] Rename a thread
      archive                            Archive a thread
      delete                             Delete a thread
      markdown                           [alias: md] Render thread as markdown
      export                             Export a thread as JSON
      raw                                [alias: raw-thread] Export raw actor thread data as JSON
    tools                                [alias: tool] Tool management commands
      list                               [alias: ls] List all active tools (including MCP tools)
      show                               Show details about an active tool
    plugins                              [alias: plugin] Manage project, system, personal, and workspace plugins
      add                                Install a plugin from a URL
      remove                             [alias: rm] Remove an installed plugin
      update                             Update installed plugins or one imported shared plugin
      list                               [alias: ls] Load and list plugins in this command, not in a running Amp session
      import                             Import a shared personal plugin into a plugin repository checkout
      repositories                       [alias: repos] List available Personal Plugins and Workspace Plugins repositories
      show-agent-options                 Show models and built-in tool names available to plugin agents
      exec                               Execute a plugin with a given event
      show-docs                          Output the plugin API documentation understood by this version of Amp
    skill                                [alias: skills] Manage skills from shared references, Git repositories, or local
                                         paths
      add                                Install skills from a source
      list                               [alias: ls] List all available skills
      remove                             [alias: rm] Remove an installed skill
      info                               Show information about a skill
      repositories                       [alias: repos] List available Personal Skills and Workspace Skills repositories
    permissions                          [alias: permission] Manage permissions
      list                               [alias: ls] List permissions
      test                               Test permissions
      edit                               Edit permissions
      add                                Add permission rule
    mcp                                  Manage MCP servers
      add                                Add an MCP server configuration
      list                               List available MCP servers
      remove                             Remove an MCP server configuration
      remote                             Manage MCP servers saved to ampcode.com
        list                             List stored remote MCP servers
        add                              Add a stored remote MCP server
        update                           Update a stored remote MCP server
        remove                           Remove a stored remote MCP server
        check                            Check a stored remote MCP server
        tools                            List tools for a stored remote MCP server
        login                            Log in to a stored remote MCP server
        logout                           Log out of a stored remote MCP server
      oauth                              Manage OAuth credentials for MCP servers in local CLI settings
        login                            Register OAuth client credentials for an MCP server
        logout                           Remove OAuth credentials for an MCP server
        status                           Show OAuth status for an MCP server
      doctor                             Check MCP server status
      approve                            Approve a workspace MCP server
    config                               Manage Amp configuration
      edit                               Open the Amp settings file in $EDITOR
      keymap                             List command keymap entries
      model-providers                    Manage model provider keys and subscriptions
        list                             List model provider keys and subscriptions in priority order
        activate                         Activate a model provider key (subscriptions other than ChatGPT replace the
                                         active connection of the same type)
        deactivate                       Deactivate a model provider key
        set-priority                     Change where a connection sits in the model routing precedence order
        show                             Show complete non-secret model provider connection data as JSON
        delete                           Delete a model provider key or subscription
        test                             Test a model provider key or subscription
        check-access                     Check which model provider serves a model by running a small inference
        add-router                       Add an AI model router and select the models routed through it
        edit-router                      Edit an AI model router
        setup-guide                      Explain how to obtain the credential an AI model router needs
        edit-key                         Edit a model provider API key name or model mapping
        add-chatgpt-subscription         Link your ChatGPT subscription to Amp as a model provider by signing in with
                                         ChatGPT in your browser
        add-github-copilot-subscription  Connect your personal GitHub Copilot subscription in the browser
      vocabulary                         Manage personal dictation vocabulary
        show                             Show personal dictation vocabulary (default)
        add                              Add terms without replacing existing vocabulary
        set                              Replace personal dictation vocabulary; omit terms to clear it
    domains                              Manage custom domains for sites
      list                               List custom domains, statuses, and required DNS records
      add                                Add a custom domain
      check                              Request an immediate DNS and provisioning status check
      remove                             [alias: rm] Remove a custom domain
    projects                             Manage Amp projects
      create                             Create an Amp-hosted project or map an existing Git repository
      update                             Update project settings by ID, [@]namespace/name, or repository
      list                               List projects accessible to you
      status                             Show the project matched by this directory's git remotes
      get                                Get a project by ID, [@]namespace/name, or repository
      snapshots                          Manage a project’s shared orb snapshots
        list                             List shared orb snapshots by project and orb size
        delete                           [alias: rm] Delete all shared orb snapshots, or one orb size with --resource
      gallery-media                      Manage a project’s Amp Store gallery media
        get                              Print a project’s Amp Store gallery media as JSON
        set                              Set a project’s Amp Store gallery media from a JSON array of
                                         {"type":"image"|"video","url":"<Amp artifact URL>","width":N,"height":N} items
      delete                             [alias: rm] Delete a project by ID, [@]namespace/name, or repository
    oauth-clients                        Manage workspace External API OAuth clients
      list                               List OAuth clients and client IDs
      create                             Create a Sign in with Amp OAuth client
      edit                               Edit a Sign in with Amp OAuth client
    secrets                              Manage environment variables and secrets for orbs and perma-orbs
      list                               List secret and environment variable names
      history                            Show the change history of secrets and environment variables, including deletions
                                         (values are never shown)
      get                                Get an environment variable value (secret values are never returned)
      set                                Add or update a secret or environment variable
    [truncated: more than 200 lines]
strings:
  # /home/agent/.local/bin/amp -> /home/agent/.local/lib/node_modules/@ampcode/cli/bin/amp
  # scanned 10 file(s), 322179688 byte(s) under /home/agent/.local/lib/node_modules/@ampcode/cli
  documented AMP_API_KEY: present
  documented AMP_URL: present
  documented AMP_SETTINGS_FILE: present
  #
  # variable-shaped tokens that name a credential or a location, documented or not
  ABORT_ERRERR_ACCESS_DENIEDERR_AMBIGUOUS_ARGUMENTERR_ARG_NOT_ITERABLEERR_ASSERTIONERR_ASYNC_CALLBACKERR_ASYNC_TYPEERR_BODY_ALREADY_USEDERR_BORINGSSLERR_BROTLI_INVALID_PARAMERR_BUFFER_OUT_OF_BOUNDSERR_BUFFER_TOO_LARGEERR_CHILD_PROCESS_IPC_REQUIREDERR_CHILD_PROCESS_STDIO_MAXBUFFERERR_CLOSED_MESSAGE_PORTERR_CONSOLE_WRITABLE_STREAMERR_CONSTRUCT_CALL_INVALIDERR_CONSTRUCT_CALL_REQUIREDERR_CRYPTO_CUSTOM_ENGINE_NOT_SUPPORTEDERR_CRYPTO_ECDH_INVALID_FORMATERR_CRYPTO_ECDH_INVALID_PUBLIC_KEYERR_CRYPTO_HASH_F  [line truncated]
  ACCESS_KEY_ID
  ADDRCONFIG
  AES_KEY_SETUP_FAILED
  AFTER_DOCTYPE_PUBLIC_KEYWORD
  AFTER_DOCTYPE_SYSTEM_KEYWORD
  AGENT_PROXY_AUTH_TOKEN
  AI_GATEWAY_API_KEY
  AMP_API_KEY
  AMP_DEPLOYMENT_DATA_DIRECTORY
  AMP_DISABLE_AMP_COAUTHOR_TRAILER
  AMP_DISABLE_SECRET_REDACTION
  AMP_GITHUB_BOOTSTRAP_TOKEN
  AMP_GIT_CREDENTIAL_HOSTS
  AMP_HEADLESS_OAUTH
  AMP_HOME
  AMP_INITIAL_AGENT_MODE_KEY
  AMP_KEYRING_ENTRY_CLASS__
  AMP_MAC_SIDECAR_GATEWAY_TOKEN
  AMP_ORB_PROVIDER_TOKEN
  AMP_PORTAL_HAIRPIN_KEY
  AMP_SECRET_ENV_NAMES
  AMP_SETTINGS_FILE
  AMP_SIGNING_KEYCHAIN
  AMP_WORKLOAD_IDENTITY_REQUEST_TOKEN
  AMP_WORKLOAD_IDENTITY_TOKEN
  ANDROID_HOME
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
  ANY_KEY__
  API_KEY_URL
  APPLE_SIGNING_P12_PASSWORD
  ASC_KEY_ID
  ASC_KEY_PATH
  ASC_PRIVATE_KEY
  ASC_PRIVATE_KEY_B64
  AUTH
  AUTHDATA
  AUTHENTICATED
  AUTHENTICATING
  AUTHH
  AUTHORITY
  AUTHORITY1
  AUTHORITY_INFO_ACCESS
  AUTHORITY_KEYID
  AUTHORIZATION
  AUTHORIZATION_EXPIRED
  AUTHORIZED
  AUTHP
  AUTH_METHODS
  AWS_ACCESS_KEY_ID
  AWS_ACCESS_KEY_IDS3_SECRET_ACCESS_KEYAWS_SECRET_ACCESS_KEYS3_REGIONAWS_REGIONS3_ENDPOINTAWS_ENDPOINTS3_BUCKETAWS_BUCKETAWS_SESSION_TOKEN
  AWS_BEARER_TOKEN_BEDROCK
  AWS_CONFIG_FILE
  AWS_CONTAINER_AUTHORIZATION_TOKEN
  AWS_CONTAINER_AUTHORIZATION_TOKEN_FILE
  AWS_CONTAINER_CREDENTIALS_FULL_URI
  AWS_CONTAINER_CREDENTIALS_RELATIVE_URI
  AWS_PROFILE
  AWS_SECRET_ACCESS_KEY
  AWS_SESSION_TOKEN
  AWS_SHARED_CREDENTIALS_FILE
  AWS_WEB_IDENTITY_TOKEN_FILE
  BAD_KEY_LENGTH
  BAD_PASSWORD_READ
  BAD_SRTP_PROTECTION_PROFILE_LIST
  BASETEN_API_KEY
  BLOCK_AS_IMPLICIT_KEY
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
  CCR_SESSION_PROFILE
  CERTIFICATE_AND_PRIVATE_KEY_MISMATCH
  CERTIFICATE_HAS_NO_KEYID
  CLAUDE_AI_AUTHORIZE_URL
  CLAUDE_BG_AUTH_SNAPSHOT_PATH
  CLAUDE_BG_CLAIM_AUTH
  CLAUDE_BG_PTY_AUTH
  CLAUDE_BG_RV_AUTH
  CLAUDE_BG_SOCKET_TOKENS_PATH
  CLAUDE_BRIDGE_OAUTH_TOKEN
  CLAUDE_CODE_AGENT_PROXY_GIT_CONFIG
  CLAUDE_CODE_API_KEY_FILE_DESCRIPTOR
  CLAUDE_CODE_API_KEY_HELPER_TTL_MS
  CLAUDE_CODE_ARTIFACTS_API_TOKEN
  CLAUDE_CODE_AUTH_FAIL_EXIT_MS
  CLAUDE_CODE_BRIDGE_CHILD_MACHINE_SETTINGS
  CLAUDE_CODE_CLIENT_KEY
  CLAUDE_CODE_CLIENT_KEY_PASSPHRASE
  CLAUDE_CODE_CUSTOM_OAUTH_URL
  CLAUDE_CODE_DESIGN_OAUTH_CLIENT_ID
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
  CLAUDE_CODE_IDLE_TOKEN_THRESHOLD
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
  CLAUDE_CODE_REMOTE_SETTINGS_PATH
  CLAUDE_CODE_REMOTE_SETTINGS_POLL_MS
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
  CLIENT_TRAFFIC_SECRET_0
  CLOUDFLARE_API_TOKEN
  CLOUDSDK_AUTH_ACCESS_TOKEN
  CLOUDSDK_CONFIG
  CMAKE_CXX_COMPILER_LAUNCHERCMAKE_C_COMPILER_LAUNCHERNODE_TLS_REJECT_UNAUTHORIZEDBUN_CONFIG_NO_CLEAR_TERMINAL_ON_RELOADCITDDIUMGITHUB_ACTIONSJENKINS_URL
  CODEX_HOME
  CONFIG
  CONNECTION_TOKEN_CHARS
  CONSOLE_AUTHORIZE_URL
  CREATE_OAUTH_CLIENT_APPS
  CURSOR_USER_DATA_DIR
  CUSTOM_URL_API_KEY
  DASHSCOPE_API_KEY
  DATA_TOO_LARGE_FOR_KEY_SIZE
  DATA_TOO_SMALL_FOR_KEY_SIZE
  DEEPSEEK_API_KEY
  DEFAULT_SETTINGS_ENABLE_CONNECT_PROTOCOL
  DEFAULT_SETTINGS_ENABLE_PUSH
  DEFAULT_SETTINGS_HEADER_TABLE_SIZE
  DEFAULT_SETTINGS_INITIAL_WINDOW_SIZE
    [truncated: more than 200 lines]
baseline-flagfile---settings-file:
  # /home/agent/probe-target
  d /home/agent/probe-target
  f /home/agent/probe-target/settings.json
  # /home/agent/.config/amp
  (absent)
  # /home/agent/.local/share/amp
  (absent)
delta-flagfile---settings-file:
  + d /home/agent/.local/share/amp
  + f /home/agent/.local/share/amp/accounts.json
  + f /home/agent/.local/share/amp/device-id.json
  - (absent)
candidates:
  1: @none
  2: 
  3: {}
container-delta:
  C /etc
  C /home/agent
  C /home
  A /home/agent/.cache
  A /home/agent/.cache/amp
  A /home/agent/.cache/amp/logs
  A /home/agent/.cache/amp/logs/cli.log
  A /home/agent/.local
  A /home/agent/.local/bin
  A /home/agent/.local/bin/amp
  A /home/agent/.local/lib
  A /home/agent/.local/lib/node_modules
  A /home/agent/.local/lib/node_modules/@ampcode
  A /home/agent/.local/lib/node_modules/@ampcode/cli
  A /home/agent/.local/lib/node_modules/@ampcode/cli/LICENSE.md
  A /home/agent/.local/lib/node_modules/@ampcode/cli/README.md
  A /home/agent/.local/lib/node_modules/@ampcode/cli/bin
  A /home/agent/.local/lib/node_modules/@ampcode/cli/bin/amp
  A /home/agent/.local/lib/node_modules/@ampcode/cli/bin/amp.exe
  A /home/agent/.local/lib/node_modules/@ampcode/cli/cli-wrapper.cjs
  A /home/agent/.local/lib/node_modules/@ampcode/cli/install.cjs
  A /home/agent/.local/lib/node_modules/@ampcode/cli/node_modules
  A /home/agent/.local/lib/node_modules/@ampcode/cli/node_modules/@ampcode
  A /home/agent/.local/lib/node_modules/@ampcode/cli/node_modules/@ampcode/cli-linux-x64
  A /home/agent/.local/lib/node_modules/@ampcode/cli/node_modules/@ampcode/cli-linux-x64/LICENSE.md
  A /home/agent/.local/lib/node_modules/@ampcode/cli/node_modules/@ampcode/cli-linux-x64/README.md
  A /home/agent/.local/lib/node_modules/@ampcode/cli/node_modules/@ampcode/cli-linux-x64/amp
  A /home/agent/.local/lib/node_modules/@ampcode/cli/node_modules/@ampcode/cli-linux-x64/package.json
  A /home/agent/.local/lib/node_modules/@ampcode/cli/package.json
  A /home/agent/.local/share
  A /home/agent/.npm
  A /home/agent/.npm/_cacache
  A /home/agent/.npm/_cacache/content-v2
  A /home/agent/.npm/_cacache/content-v2/sha512
  A /home/agent/.npm/_cacache/content-v2/sha512/0d
  A /home/agent/.npm/_cacache/content-v2/sha512/0d/d3
  A /home/agent/.npm/_cacache/content-v2/sha512/0d/d3/c0e781b60e7b1e3195b99fa886b0f9db7c0e9d5142d7fb41ce0b331aefca95ae859679f543a0b401ca69fe3ae008cee19deb8e25d4363d7b106d63bfb58a
  A /home/agent/.npm/_cacache/content-v2/sha512/1a
  A /home/agent/.npm/_cacache/content-v2/sha512/1a/2f
  A /home/agent/.npm/_cacache/content-v2/sha512/1a/2f/e98194c225bf4ddcdc3b9ab5cc213bfde4b514de80e5b05f941e3e95b216e391541ed5d9cf0a2e51897943bea1104dc7e6bdadd9efb093e942bcec3e15a3
  A /home/agent/.npm/_cacache/content-v2/sha512/53
  A /home/agent/.npm/_cacache/content-v2/sha512/53/13
  A /home/agent/.npm/_cacache/content-v2/sha512/53/13/ab1bcfe59c37d1fe112208749537e5a586d755c890e0397169073b0752d5ba0d93403b7b7d950daa1afe0ebf3134e834d03ae3d0860cccfa150cf47da763
  A /home/agent/.npm/_cacache/content-v2/sha512/62
  A /home/agent/.npm/_cacache/content-v2/sha512/62/69
  A /home/agent/.npm/_cacache/content-v2/sha512/62/69/933e0a6d7bbbf990a486772dacdd18c275dd66f86fdd011caa0031e1e824a7816da6a01185605abf1a685a4bf9328098b072e94617eea596d255b1c1fc4f
  A /home/agent/.npm/_cacache/content-v2/sha512/92
  A /home/agent/.npm/_cacache/content-v2/sha512/92/59
  A /home/agent/.npm/_cacache/content-v2/sha512/92/59/402c145c108fb23576944c447854b6b11af46751d2b65a5d6c87cc377072f0244d6515be2a57253b92e6a58c68fc721aa53420ca0ebf8d6c3602424e5214
  A /home/agent/.npm/_cacache/content-v2/sha512/ad
  A /home/agent/.npm/_cacache/content-v2/sha512/ad/52
  A /home/agent/.npm/_cacache/content-v2/sha512/ad/52/f1aecfd76d68cf74f29af708d7c29565d7888073e384901620fd0ba9f8d12ee8fc47485b99f1851895767e38c668216ab787a386884e44bd74eeb8fe6e9c
  A /home/agent/.npm/_cacache/content-v2/sha512/bc
  A /home/agent/.npm/_cacache/content-v2/sha512/bc/bb
  A /home/agent/.npm/_cacache/content-v2/sha512/bc/bb/73ca3282edea5c53c72427a6a2fb61a3346051893a41591efe3ef98aae52d286cb2399370ff8b7d84565fc941589695c3e9b3150cd1d0ae35fdb82023eab
  A /home/agent/.npm/_cacache/content-v2/sha512/c9
  A /home/agent/.npm/_cacache/content-v2/sha512/c9/24
  A /home/agent/.npm/_cacache/content-v2/sha512/c9/24/ed63286daab5847a6d0d6d5ee0fc7682cf7d934bad0870f1deccf07bc1337ed0f41315a5ce1c9f0bcebc73fe39da65ec6d3ec4fd3544a17d20148b22ac1a
  A /home/agent/.npm/_cacache/index-v5
  A /home/agent/.npm/_cacache/index-v5/02
  A /home/agent/.npm/_cacache/index-v5/02/c5
  A /home/agent/.npm/_cacache/index-v5/02/c5/faf7106c9d6d6c8cfd8b0738f17d2542ca24dcaeaac5dd58e3d06b24e3ff
  A /home/agent/.npm/_cacache/index-v5/58
  A /home/agent/.npm/_cacache/index-v5/58/54
  A /home/agent/.npm/_cacache/index-v5/58/54/93b57b7c79bc66e9fad7e090c2fb89bea26c300fcee09b388ceeaed7c85c
  A /home/agent/.npm/_cacache/index-v5/5b
  A /home/agent/.npm/_cacache/index-v5/5b/eb
  A /home/agent/.npm/_cacache/index-v5/5b/eb/72be895465ac289bdf331f992c36a93f794f4d8c44d6414d14dab5ddfc9c
  A /home/agent/.npm/_cacache/index-v5/76
  A /home/agent/.npm/_cacache/index-v5/76/02
  A /home/agent/.npm/_cacache/index-v5/76/02/93ce72222d5fa288c6d90864b9299280ef622785251ff182468386cde194
  A /home/agent/.npm/_cacache/index-v5/77
  A /home/agent/.npm/_cacache/index-v5/77/24
  A /home/agent/.npm/_cacache/index-v5/77/24/df76a2b9b633b3f526674deb2ec505282e884baca150cd35977d75d18b7b
  A /home/agent/.npm/_cacache/index-v5/9f
  A /home/agent/.npm/_cacache/index-v5/9f/27
  A /home/agent/.npm/_cacache/index-v5/9f/27/1e8d43a2512db858e6b27eaa62ffc5f2300bd1237375b268fcabc67fc170
  A /home/agent/.npm/_cacache/index-v5/a4
  A /home/agent/.npm/_cacache/index-v5/a4/81
  A /home/agent/.npm/_cacache/index-v5/a4/81/7abf083b4d21ce2767622c6a519ab4c4fd0a36bc8e705abf9a9e2b8ca9e6
  A /home/agent/.npm/_cacache/index-v5/a9
  A /home/agent/.npm/_cacache/index-v5/a9/fb
  A /home/agent/.npm/_cacache/index-v5/a9/fb/35f53a704aeb87f34674a61ae401938206d576db49770f67854ee84e9cb8
  A /home/agent/.npm/_cacache/tmp
  A /home/agent/.npm/_logs
  A /home/agent/.npm/_logs/2026-10-25T00_22_59_503Z-debug-0.log
  A /home/agent/.npm/_update-notifier-last-checked
  A /home/agent/probe-target
  A /home/agent/probe-target/settings.json
  C /home/probe
  A /home/probe/.probe-out
  A /home/probe/.probe-out/after-flagfile---settings-file.txt
  A /home/probe/.probe-out/baseline-flagfile---settings-file.txt
  A /home/probe/.probe-out/behaviour-flagfile---settings-file.cmd
  A /home/probe/.probe-out/behaviour-flagfile---settings-file.exit-code
  A /home/probe/.probe-out/behaviour-flagfile---settings-file.txt
  A /home/probe/.probe-out/candidate-1.cmd
  A /home/probe/.probe-out/candidate-1.exit-code
  A /home/probe/.probe-out/candidate-1.txt
  A /home/probe/.probe-out/candidate-2.cmd
  A /home/probe/.probe-out/candidate-2.exit-code
  A /home/probe/.probe-out/candidate-2.txt
  A /home/probe/.probe-out/candidate-3.cmd
  A /home/probe/.probe-out/candidate-3.exit-code
  A /home/probe/.probe-out/candidate-3.txt
  A /home/probe/.probe-out/candidates.txt
  A /home/probe/.probe-out/delta-flagfile---settings-file.txt
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
  A /home/probe/.probe-state/pristine/2.absent
  A /home/probe/.probe-state/pristine/index
  A /home/probe/.probe-state/steps
  A /src
  C /tmp
  A /tmp/.bun-1001-42512e2cf721069e.node
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
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/56a6346a
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/57a28ded
  A /tmp/node-compile-cache/v24.21.0-x64-964aae3f-1001/5839ec17
    [truncated: more than 300 lines]
