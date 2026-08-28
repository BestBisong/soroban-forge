# Privacy and Telemetry

`soroban-forge` collects no telemetry. It does not send usage analytics, crash
reports, command arguments, project contents, identifiers, or any other data to
the maintainers or to an analytics service.

Some commands make network requests only to perform an operation the user
explicitly requested, such as cloning a remote template, checking testnet RPC
connectivity, funding an identity with friendbot, or fetching deployed contract
Wasm for verification. These requests are functional, not telemetry. Pass
`--offline` to disable all such network access.

If telemetry is ever introduced, it will be strictly opt-in and disabled by
default. The project will document what is collected, its purpose, destination,
retention period, and how to revoke consent before asking users to enable it.
An upgrade will never silently enroll an existing installation.
