# Linked plugins and discovery

Plugins implement `CorePlugin` and return an immutable `CoreDescriptor` with
`id`, plugin `release`, Core `contract` version 2.0.0, exact-version
`provides`/`requires`, schemas, event-kind schema bindings and action declarations.
SchemaId and ActionId namespaces are globally unique across installed plugins.

Install providers before consumers. Pass `Bindings` from required CapabilityId to
provider PluginId; missing/wrong/extra bindings, incompatible exact versions and
cycles fail before installation commits. The callback cannot write another owner.
Providers cannot be removed while consumers remain installed. Remove consumers
first and use a fresh `owner()` token following restore.

Call `describe()` to inspect actual supported versions/limits, installed descriptors
and bindings, removal blockers, tick and the projected last accepted input. Core
does not advertise views/gameplay capabilities on its own. Capability labels have
no built-in semantic interpretation.

Create Scene/Entity identity and required records in `initialize`; use context
snapshots/actions and transactional commands in `tick`. Keep durable state in
records and random draws in `tx.draw()`. The example `examples/p2_public.rs`
demonstrates this without engine implementation access.

Restore does not deserialize executable plugin objects or call callbacks. Construct
the same registered plugins/bindings in a host with matching seed/content binding,
then restore save2. Descriptors or content mismatches reject atomically. Save1 is
explicitly a different version. Plugin callbacks must remain deterministic and
free of hidden mutable durable state to reproduce a continuation.

Native dynamic plugin loading, OS isolation and platform input mapping are outside
P2. The SDK contains ordinary linked Rust artifacts; no discovery scanner loads
arbitrary files or native DLLs.
