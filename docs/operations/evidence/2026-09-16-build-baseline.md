<!-- darkstar-header-v1 -->
<!-- po co: build/test baseline 2026-09-16 -->
<!-- nie wolno: zmieniac sieci/firewall/tailscale/headscale/hotspot poza zakresem tasku -->
<!-- autor: Marcin -->
<!-- powstal: 2026-09-16 -->
<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: GPT-5 Codex
TIMESTAMP: 2026-09-16T05:40:27.495+00:00
REASON FOR CREATION: Utrwalenie zweryfikowanej bazy build/test dla workspace zgodnie z taskiem niefunkcjonalnym. Zawiera dokladne komendy i dokladny output przed i po poprawkach.
SYSTEM PART: Darkstar / evidence
GITHUB METADATA: jpytka666-jpg/polip-agi
==========================================
-->

# Build/test baseline — 2026-09-16

## Environment

```text
rustc 1.98.1 (48a229cea 2026-09-01)
cargo 1.98.1 (797e8a9bc 2026-08-05)
node v22.23.2
npm 10.9.8
```

## Scope notes

- Frontend `package.json` does **not** define a `typecheck` script.
- Defined and executed frontend checks were: `npm ci`, `npm run lint`, `npm run test:pin`, `npm run test:git`.

## Commands and exact output — BEFORE

### `cargo build --workspace --all-targets`

```text
    Updating crates.io index
    Updating git repository `https://github.com/jpytka666-jpg/wpc-engine.git`
 Downloading crates ...
  Downloaded futures-core v0.3.34
  Downloaded futures-sink v0.3.34
  Downloaded futures-task v0.3.34
  Downloaded http-range-header v0.4.2
  Downloaded httpdate v1.0.3
  Downloaded ident_case v1.0.1
  Downloaded atomic-waker v1.1.2
  Downloaded base64 v0.13.1
  Downloaded crossbeam-utils v0.8.22
  Downloaded darling_macro v0.20.11
  Downloaded derive_builder_macro v0.20.2
  Downloaded either v1.18.0
  Downloaded errno v0.3.14
  Downloaded getrandom v0.2.17
  Downloaded http-body-util v0.1.5
  Downloaded num-complex v0.4.6
  Downloaded percent-encoding v2.3.2
  Downloaded aho-corasick v1.1.5
  Downloaded autocfg v1.5.1
  Downloaded axum-core v0.5.6
  Downloaded cfg-if v1.0.4
  Downloaded derive_builder_core v0.20.2
  Downloaded find-msvc-tools v0.1.12
  Downloaded fnv v1.0.7
  Downloaded form_urlencoded v1.2.2
  Downloaded http-body v1.1.0
  Downloaded httparse v1.10.1
  Downloaded lazy_static v1.5.0
  Downloaded itoa v1.0.18
  Downloaded pastey v0.2.3
  Downloaded pin-project-lite v0.2.17
  Downloaded ppv-lite86 v0.2.21
  Downloaded tracing-core v0.1.36
  Downloaded tracing-log v0.2.0
  Downloaded log v0.4.34
  Downloaded macro_rules_attribute v0.2.3
  Downloaded matrixmultiply v0.3.11
  Downloaded ort-sys v2.0.0-rc.13
  Downloaded parking_lot v0.12.5
  Downloaded paste v1.0.15
  Downloaded pkg-config v0.3.34
  Downloaded proc-macro2 v1.0.107
  Downloaded shlex v2.0.1
  Downloaded signal-hook-registry v1.4.8
  Downloaded slab v0.4.12
  Downloaded smallvec v1.15.2
  Downloaded strsim v0.11.1
  Downloaded sync_wrapper v1.0.2
  Downloaded thiserror v2.0.20
  Downloaded thiserror-impl v1.0.69
  Downloaded unicase v2.9.0
  Downloaded unicode-ident v1.0.24
  Downloaded bytes v1.12.1
  Downloaded cc v1.4.5
  Downloaded crossbeam-epoch v0.9.20
  Downloaded darling v0.20.11
  Downloaded hyper-util v0.1.20
  Downloaded libloading v0.9.0
  Downloaded lock_api v0.4.14
  Downloaded macro_rules_attribute-proc_macro v0.2.3
  Downloaded memchr v2.8.3
  Downloaded monostate-impl v0.1.18
  Downloaded num-integer v0.1.47
  Downloaded num-traits v0.2.19
  Downloaded once_cell v1.21.4
  Downloaded onig v6.5.3
  Downloaded parking_lot_core v0.9.12
  Downloaded quote v1.0.47
  Downloaded rand v0.8.8
  Downloaded serde_derive v1.0.229
  Downloaded tracing-subscriber v0.3.23
  Downloaded unicode_categories v0.1.1
  Downloaded bitflags v2.13.1
  Downloaded crossbeam-deque v0.8.7
  Downloaded darling_core v0.20.11
  Downloaded derive_builder v0.20.2
  Downloaded futures-channel v0.3.34
  Downloaded getrandom v0.4.3
  Downloaded http v1.5.0
  Downloaded matchers v0.2.0
  Downloaded tower v0.5.3
  Downloaded uuid v1.26.0
  Downloaded axum v0.8.9
  Downloaded itertools v0.12.1
  Downloaded ort v2.0.0-rc.13
  Downloaded rayon v1.12.0
  Downloaded tower-http v0.6.11
  Downloaded esaxx-rs v0.1.10
  Downloaded futures-util v0.3.34
  Downloaded hyper v1.11.0
  Downloaded itertools v0.11.0
  Downloaded mio v1.2.2
  Downloaded nom v7.1.3
  Downloaded syn v2.0.119
  Downloaded spm_precompiled v0.1.4
  Downloaded regex v1.13.1
  Downloaded tokio v1.53.1
  Downloaded tokio-macros v2.7.2
  Downloaded regex-syntax v0.8.11
  Downloaded thiserror-impl v2.0.20
  Downloaded tower-layer v0.3.3
  Downloaded unicode-segmentation v1.13.3
  Downloaded libc v0.2.189
  Downloaded onig_sys v69.9.3
  Downloaded serde_json v1.0.151
  Downloaded socket2 v0.6.5
  Downloaded zerocopy v0.8.56
  Downloaded matchit v0.8.4
  Downloaded mime v0.3.17
  Downloaded rand_chacha v0.3.1
  Downloaded rand_core v0.6.4
  Downloaded rawpointer v0.2.1
  Downloaded regex-automata v0.4.18
  Downloaded syn v3.0.4
  Downloaded tokenizers v0.20.4
  Downloaded monostate v0.1.18
  Downloaded wordnet-lemmatizer v0.1.0
  Downloaded thread_local v1.1.10
  Downloaded rayon-core v1.13.0
  Downloaded serde_path_to_error v0.1.20
  Downloaded tokio-util v0.7.19
  Downloaded ndarray v0.17.2
  Downloaded rayon-cond v0.3.0
  Downloaded ryu v1.0.23
  Downloaded minimal-lexical v0.2.1
  Downloaded scopeguard v1.2.0
  Downloaded serde_urlencoded v0.7.1
  Downloaded tower-service v0.3.3
  Downloaded tokio-stream v0.1.19
  Downloaded zmij v1.0.23
  Downloaded serde_core v1.0.229
  Downloaded unicode-normalization-alignments v0.1.12
  Downloaded thiserror v1.0.69
  Downloaded serde v1.0.229
  Downloaded mime_guess v2.0.5
  Downloaded tracing v0.1.44
  Downloaded sharded-slab v0.1.7
  Downloaded nu-ansi-term v0.50.3
  Downloaded tracing-attributes v0.1.31
   Compiling proc-macro2 v1.0.107
   Compiling quote v1.0.47
   Compiling unicode-ident v1.0.24
   Compiling libc v0.2.189
   Compiling cfg-if v1.0.4
   Compiling itoa v1.0.18
   Compiling serde_core v1.0.229
   Compiling memchr v2.8.3
   Compiling serde v1.0.229
   Compiling zmij v1.0.23
   Compiling smallvec v1.15.2
   Compiling syn v3.0.4
   Compiling syn v2.0.119
   Compiling serde_json v1.0.151
   Compiling once_cell v1.21.4
   Compiling thiserror v2.0.20
   Compiling log v0.4.34
   Compiling aho-corasick v1.1.5
   Compiling regex-syntax v0.8.11
   Compiling serde_derive v1.0.229
   Compiling thiserror-impl v2.0.20
   Compiling regex-automata v0.4.18
   Compiling lazy_static v1.5.0
   Compiling bitflags v2.13.1
   Compiling autocfg v1.5.1
   Compiling crossbeam-utils v0.8.22
   Compiling strsim v0.11.1
   Compiling fnv v1.0.7
   Compiling crossbeam-epoch v0.9.20
   Compiling ident_case v1.0.1
   Compiling darling_core v0.20.11
   Compiling num-traits v0.2.19
   Compiling crossbeam-deque v0.8.7
   Compiling find-msvc-tools v0.1.12
   Compiling shlex v2.0.1
   Compiling zerocopy v0.8.56
   Compiling cc v1.4.5
   Compiling either v1.18.0
   Compiling rayon-core v1.13.0
   Compiling pkg-config v0.3.34
   Compiling darling_macro v0.20.11
   Compiling onig_sys v69.9.3
   Compiling darling v0.20.11
   Compiling matrixmultiply v0.3.11
   Compiling getrandom v0.2.17
   Compiling rand_core v0.6.4
   Compiling derive_builder_core v0.20.2
   Compiling thiserror v1.0.69
   Compiling paste v1.0.15
   Compiling rawpointer v0.2.1
   Compiling minimal-lexical v0.2.1
   Compiling ort-sys v2.0.0-rc.13
   Compiling esaxx-rs v0.1.10
   Compiling nom v7.1.3
   Compiling rayon v1.12.0
   Compiling ppv-lite86 v0.2.21
   Compiling derive_builder_macro v0.20.2
   Compiling num-integer v0.1.47
   Compiling rand_chacha v0.3.1
   Compiling num-complex v0.4.6
   Compiling itertools v0.11.0
   Compiling darkstar-recall v0.1.0 (/home/runner/work/polip-agi/polip-agi/crates/darkstar-recall)
   Compiling thiserror-impl v1.0.69
   Compiling monostate-impl v0.1.18
   Compiling unicode-segmentation v1.13.3
   Compiling macro_rules_attribute-proc_macro v0.2.3
   Compiling base64 v0.13.1
   Compiling pastey v0.2.3
   Compiling spm_precompiled v0.1.4
   Compiling rayon-cond v0.3.0
   Compiling monostate v0.1.18
   Compiling ndarray v0.17.2
   Compiling macro_rules_attribute v0.2.3
   Compiling rand v0.8.8
   Compiling derive_builder v0.20.2
   Compiling itertools v0.12.1
   Compiling regex v1.13.1
   Compiling unicode-normalization-alignments v0.1.12
   Compiling libloading v0.9.0
   Compiling pin-project-lite v0.2.17
   Compiling unicode_categories v0.1.1
   Compiling darkstar-shadow v0.1.0 (/home/runner/work/polip-agi/polip-agi/crates/darkstar-shadow)
   Compiling wordnet-lemmatizer v0.1.0
   Compiling cbms-writing v0.1.0 (https://github.com/jpytka666-jpg/wpc-engine.git?rev=bad82f2e5f11973ceedf067d368f6983b093ea3d#bad82f2e)
   Compiling bytes v1.12.1
   Compiling parking_lot_core v0.9.12
   Compiling futures-core v0.3.34
   Compiling tracing-core v0.1.36
   Compiling ort v2.0.0-rc.13
   Compiling scopeguard v1.2.0
   Compiling lock_api v0.4.14
   Compiling tracing-attributes v0.1.31
   Compiling errno v0.3.14
   Compiling signal-hook-registry v1.4.8
   Compiling parking_lot v0.12.5
   Compiling tokio-macros v2.7.2
   Compiling tracing v0.1.44
   Compiling socket2 v0.6.5
   Compiling mio v1.2.2
   Compiling tokio v1.53.1
   Compiling http v1.5.0
   Compiling getrandom v0.4.3
   Compiling httparse v1.10.1
   Compiling http-body v1.1.0
   Compiling tower-service v0.3.3
   Compiling uuid v1.26.0
   Compiling httpdate v1.0.3
   Compiling tower-layer v0.3.3
   Compiling unicase v2.9.0
   Compiling futures-task v0.3.34
   Compiling percent-encoding v2.3.2
   Compiling slab v0.4.12
   Compiling mime v0.3.17
   Compiling futures-util v0.3.34
   Compiling mime_guess v2.0.5
   Compiling http-body-util v0.1.5
   Compiling futures-channel v0.3.34
   Compiling sync_wrapper v1.0.2
   Compiling atomic-waker v1.1.2
   Compiling futures-sink v0.3.34
   Compiling form_urlencoded v1.2.2
   Compiling darkstar-core v0.1.0 (/home/runner/work/polip-agi/polip-agi/crates/darkstar-core)
   Compiling ryu v1.0.23
   Compiling serde_urlencoded v0.7.1
   Compiling axum-core v0.5.6
   Compiling hyper v1.11.0
   Compiling tokio-util v0.7.19
   Compiling tower v0.5.3
   Compiling hyper-util v0.1.20
   Compiling tracing-log v0.2.0
   Compiling matchers v0.2.0
   Compiling sharded-slab v0.1.7
   Compiling serde_path_to_error v0.1.20
   Compiling thread_local v1.1.10
   Compiling nu-ansi-term v0.50.3
   Compiling http-range-header v0.4.2
   Compiling matchit v0.8.4
   Compiling tower-http v0.6.11
   Compiling tracing-subscriber v0.3.23
   Compiling axum v0.8.9
   Compiling tokio-stream v0.1.19
   Compiling darkstar-server v0.1.0 (/home/runner/work/polip-agi/polip-agi/crates/darkstar-server)
   Compiling onig v6.5.3
   Compiling tokenizers v0.20.4
   Compiling darkstar-embed v0.1.0 (/home/runner/work/polip-agi/polip-agi/crates/darkstar-embed)
error[E0599]: no method named `plain_id_for_symbol` found for struct `Vocabulary<'b>` in the current scope
  --> crates/darkstar-embed/src/cbms_ids.rs:93:19
   |
93 |             vocab.plain_id_for_symbol(unit).map_err(|e| {
   |                   ^^^^^^^^^^^^^^^^^^^ method not found in `Vocabulary<'_>`

For more information about this error, try `rustc --explain E0599`.
error: could not compile `darkstar-embed` (lib) due to 1 previous error
warning: build failed, waiting for other jobs to finish...

```

### `cargo test --workspace`

```text
   Compiling darkstar-embed v0.1.0 (/home/runner/work/polip-agi/polip-agi/crates/darkstar-embed)
   Compiling darkstar-seed v0.1.0 (/home/runner/work/polip-agi/polip-agi/crates/darkstar-seed)
error[E0599]: no method named `plain_id_for_symbol` found for struct `Vocabulary<'b>` in the current scope
  --> crates/darkstar-embed/src/cbms_ids.rs:93:19
   |
93 |             vocab.plain_id_for_symbol(unit).map_err(|e| {
   |                   ^^^^^^^^^^^^^^^^^^^ method not found in `Vocabulary<'_>`

error[E0599]: no method named `plain_id_for_symbol` found for struct `cbms_writing::Vocabulary<'b>` in the current scope
  --> crates/darkstar-embed/src/cbms_ids.rs:93:19
   |
93 |             vocab.plain_id_for_symbol(unit).map_err(|e| {
   |                   ^^^^^^^^^^^^^^^^^^^ method not found in `cbms_writing::Vocabulary<'_>`

For more information about this error, try `rustc --explain E0599`.
error: could not compile `darkstar-embed` (lib) due to 1 previous error
warning: build failed, waiting for other jobs to finish...
error: could not compile `darkstar-embed` (lib test) due to 1 previous error

```

### `cd /home/runner/work/polip-agi/polip-agi/frontend && npm ci`

```text

added 172 packages, and audited 173 packages in 6s

42 packages are looking for funding
  run `npm fund` for details

found 0 vulnerabilities

```

### `cd /home/runner/work/polip-agi/polip-agi/frontend && npm run lint`

```text

> frontend@0.0.0 lint
> eslint .


```

### `cd /home/runner/work/polip-agi/polip-agi/frontend && npm run test:pin`

```text

> frontend@0.0.0 test:pin
> node --experimental-strip-types --disable-warning=ExperimentalWarning --test tests/operatorPin.test.ts

TAP version 13
# Subtest: operator pin starts with exactly four cells
ok 1 - operator pin starts with exactly four cells
  ---
  duration_ms: 1.425349
  type: 'test'
  ...
# Subtest: incomplete operator pin does not authorize a request
ok 2 - incomplete operator pin does not authorize a request
  ---
  duration_ms: 0.26159
  type: 'test'
  ...
# Subtest: complete operator pin gets one bearer scheme prefix
ok 3 - complete operator pin gets one bearer scheme prefix
  ---
  duration_ms: 0.162183
  type: 'test'
  ...
# Subtest: pasted operator pin fills no more than four cells
ok 4 - pasted operator pin fills no more than four cells
  ---
  duration_ms: 0.171791
  type: 'test'
  ...
# Subtest: editing one operator pin cell preserves the other cells
ok 5 - editing one operator pin cell preserves the other cells
  ---
  duration_ms: 0.198772
  type: 'test'
  ...
1..5
# tests 5
# suites 0
# pass 5
# fail 0
# cancelled 0
# skipped 0
# todo 0
# duration_ms 105.016357

```

### `cd /home/runner/work/polip-agi/polip-agi/frontend && npm run test:git`

```text

> frontend@0.0.0 test:git
> node --experimental-strip-types --disable-warning=ExperimentalWarning --test tests/gitOverview.test.ts

TAP version 13
# Subtest: missing Git endpoint becomes an unavailable view instead of a raw error
ok 1 - missing Git endpoint becomes an unavailable view instead of a raw error
  ---
  duration_ms: 22.39249
  type: 'test'
  ...
# Subtest: Git overview is read from the authenticated darkstar-server endpoint
ok 2 - Git overview is read from the authenticated darkstar-server endpoint
  ---
  duration_ms: 2.084538
  type: 'test'
  ...
1..2
# tests 2
# suites 0
# pass 2
# fail 0
# cancelled 0
# skipped 0
# todo 0
# duration_ms 126.826376

```

## Commands and exact output — AFTER

### `cargo build --workspace --all-targets`

```text
   Compiling darkstar-embed v0.1.0 (/home/runner/work/polip-agi/polip-agi/crates/darkstar-embed)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.90s

```

### `cargo test --workspace`

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.18s
     Running unittests src/lib.rs (target/debug/deps/darkstar_core-9b63b1f38a01aa94)

running 84 tests
test capability_gate::tests::execute_requires_trusted_approval ... ok
test capability_index::tests::returns_empty_index_for_empty_registry ... ok
test capability_gate::tests::missing_capability_never_reaches_execution_permission ... ok
test capability_selector::tests::prefers_requested_runtime_and_platform ... ok
test capability_gate::tests::read_capability_is_allowed_after_selection ... ok
test capability_index::tests::indexes_capabilities_from_registry ... ok
test capability_selector::tests::unknown_capability_returns_no_candidates ... ok
test capability_selector::tests::selection_is_deterministic ... ok
test context_client::tests::falls_back_to_the_local_leg_when_e_is_silent ... ok
test context_client::tests::each_leg_uses_the_api_version_its_server_speaks ... ok
test context_client::tests::falls_back_to_the_share_when_the_local_leg_is_silent ... ok
test context_client::tests::fails_closed_when_both_legs_are_silent ... ok
test context_client::tests::health_reports_each_leg_separately ... ok
test context_client::tests::limit_is_respected ... ok
test context_client::tests::lists_collections_from_the_preferred_leg ... ok
test context_client::tests::search_filters_collection_names_without_touching_the_store ... ok
test discovery::tests::discovers_and_registers_manifest ... ok
test gateway_module::tests::descriptor_carries_no_secret_fields ... ok
test gateway_module::tests::descriptor_declares_the_planned_capabilities ... ok
test discovery::tests::empty_directory_discovers_nothing ... ok
test gateway_module::tests::mutating_commands_are_refused_while_unimplemented ... ok
test gateway_module::tests::registers_once_and_rejects_a_duplicate ... ok
test gateway_provider::tests::abandoned_subnet_from_host_is_rejected_by_validation ... ok
test gateway_provider::tests::counts_only_downstream_neighbours ... ok
test gateway_provider::tests::nmcli_blocked_still_yields_status_from_ip ... ok
test gateway_provider::tests::profile_that_is_not_active_reports_offline ... ok
test gateway_provider::tests::reads_live_gateway_shape_from_host_output ... ok
test gateway_provider::tests::shared_method_without_address_is_degraded ... ok
test gateway_provider::tests::subnet_is_derived_from_address_and_prefix ... ok
test gateway_provider::tests::without_nmcli_and_without_address_it_is_offline ... ok
test gateway_status::tests::abandoned_subnet_is_rejected ... ok
test gateway_status::tests::live_configuration_is_accepted ... ok
test gateway_status::tests::loopback_downstream_address_is_rejected ... ok
test gateway_status::tests::path_like_interface_is_rejected ... ok
test gateway_status::tests::public_downstream_subnet_is_rejected ... ok
test gateway_status::tests::same_interface_needs_preflight_approval ... ok
test gateway_status::tests::status_round_trips_through_json ... ok
test gateway_status::tests::unspecified_bind_is_rejected ... ok
test module_execution::tests::approved_module_command_reaches_provider ... ok
test module_execution::tests::execute_without_trusted_approval_stops_before_provider ... ok
test module_execution::tests::missing_session_capability_stops_before_provider ... ok
test module_execution::tests::provider_receives_only_authorized_command ... ok
test module_provider::tests::dry_run_provider_maps_all_lifecycle_commands_to_states ... ok
test module_provider::tests::execute_is_authorized_before_reaching_provider ... ok
test module_provider::tests::execute_without_approval_cannot_create_authorized_command ... ok
test gateway_module::tests::only_inspection_is_implemented_today ... ok
test module_registry::tests::registry_is_deterministic_and_stateful ... ok
test module_state::tests::command_round_trips_as_json ... ok
test module_state::tests::commands_expose_control_capabilities ... ok
test network_topology::tests::aions_is_the_only_protected_endpoint ... ok
test network_topology::tests::first_azure_deployment_has_three_edge_roles ... ok
test network_topology::tests::layer_serializes_under_the_new_name ... ok
test module_provider::tests::missing_capability_cannot_create_authorized_command ... ok
test network_topology::tests::legacy_sheriff_layer_still_deserializes ... ok
test network_topology::tests::reference_path_is_ordered ... ok
test network_topology::tests::legacy_node_id_resolves_to_warlock ... ok
test orchestrator::tests::blocked_module_is_never_activated ... ok
test orchestrator::tests::in_flight_and_failed_states_are_not_replanned ... ok
test orchestrator::tests::offline_module_gets_start_command ... ok
test orchestrator::tests::planned_command_converts_to_execution_request ... ok
test orchestrator::tests::ready_module_can_be_stopped ... ok
test orchestrator::tests::running_module_uses_restart_to_return_to_ready ... ok
test orchestrator::tests::unsupported_ready_to_running_transition_is_not_invented ... ok
test plugin::tests::manifest_round_trips_as_json ... ok
test plugin_host::tests::unknown_api_version_is_rejected ... ok
test plugin_host::tests::valid_manifest_is_accepted ... ok
test policy::tests::execution_without_trusted_approval_stops_at_policy ... ok
test module_registry::tests::duplicates_are_rejected ... ok
test network_topology::tests::only_one_perimeter_node_exists ... ok
test policy::tests::missing_capability_is_denied_even_with_approval ... ok
test policy::tests::reads_are_allowed_when_capability_is_present ... ok
test registry::tests::duplicate_plugin_name_is_rejected ... ok
test registry::tests::invalid_manifest_is_rejected_before_storage ... ok
test registry::tests::register_and_lookup_work ... ok
test round_table::tests::human_and_agent_have_distinct_capability_scopes ... ok
test policy::tests::model_cannot_forge_approval_through_request_data ... ok
test service_status::tests::empty_output_is_an_error_not_an_assumption ... ok
test service_status::tests::the_read_path_exposes_no_lifecycle_command ... okPython 3.12.3

test stdio::tests::adapter_reports_stdio_transport ... ok
test service_status::tests::parses_every_word_systemctl_actually_prints ... ok
test system_graph::tests::headplane_node_reads_headscale_only_no_egress_or_control_edge ... ok
test system_graph::tests::json_round_trip_preserves_metadata ... ok
test system_graph::tests::snapshot_contains_darkstar_core_and_external_systems ... ok
test stdio::tests::python_plugin_round_trip_works ... ok

test result: ok. 84 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s

     Running tests/memory_store.rs (target/debug/deps/memory_store-0e9c288a299bb19a)

running 1 test
test stores_and_reads_memory_with_session_isolation ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/service_status_read.rs (target/debug/deps/service_status_read-9cd50cdea9949eeb)

running 8 tests
test active_unit_answering_on_8001_reads_as_running ... ok
test active_unit_that_stopped_answering_is_not_reported_as_running ... ok
test activating_unit_reads_as_starting ... ok
test inactive_unit_reads_as_offline_and_puls_is_not_probed ... ok
test unrecognised_systemctl_output_is_an_error_not_a_guess ... ok
test failed_unit_reads_as_failed ... ok
test chroma_e_copy_preset_targets_the_unit_and_endpoint_measured_on_cbms ... ok
test status_read_issues_only_read_commands ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running unittests src/lib.rs (target/debug/deps/darkstar_embed-16dc4fd751294a07)

running 26 tests
test cbms_ids::tests::frontend_to_cbms_memorilo_full_sequence ... ok
test cbms_ids::tests::adversarial_distinct_eo_do_not_collide_via_glyph_space ... ok
test cbms_ids::tests::frontend_to_cbms_kodo_matches_writer_baseline ... ok
test cbms_ids::tests::frontend_to_cbms_memoro_full_sequence ... ok
test cbms_ids::tests::glyph_text_is_not_a_bridge_to_kodo_id ... ok
test cbms_ids::tests::verified_morpheme_vectors_use_plain_ids_in_order ... ok
test eo_morph::tests::applies_real_compound_flag_contract ... ok
test eo_morph::tests::artifact_examples_if_configured ... ok
test eo_morph::tests::parses_stems_and_affixes ... ok
test eo_morph::tests::reverses_strip_and_checks_literal_condition ... ok
test eo_morph::tests::zero_surface_affix_satisfies_needaffix_without_empty_morpheme ... ok
test noworodek::tests::description_has_unit_length ... ok
test cbms_ids::tests::frontend_to_cbms_cxifro_full_sequence ... ok
test noworodek::tests::missing_parts_are_reported_clearly ... ok
test noworodek::tests::legacy_u16_parser_is_named_and_separate ... ok
test noworodek::tests::parse_cbms_ids_u32_rejects_empty ... ok
test noworodek::tests::parse_cbms_ids_u32_rejects_odd_lengths ... ok
test noworodek::tests::parse_cbms_ids_u32_roundtrips_including_above_u16 ... ok
test noworodek::tests::symbols_with_no_description_are_skipped ... ok
test noworodek::tests::unknown_symbols_do_not_change_the_description ... ok
test tests::all_padding_gives_zeros_instead_of_dividing_by_zero ... ok
test noworodek::tests::embed_bounds_reject_id_at_or_above_vocab ... ok
test tests::result_has_unit_length ... ok
test tests::missing_model_directory_is_reported_clearly ... ok
test tests::truncated_output_does_not_panic ... ok
test tests::padding_tokens_do_not_dilute_the_average ... ok

test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running unittests src/bin/eo_morph_conformance.rs (target/debug/deps/eo_morph_conformance-19cf7fd79efc896b)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/bin/espdic.rs (target/debug/deps/espdic-7b4916c6fe295749)

running 6 tests
test tests::dalekie_znaczenie_tez_jest_kandydatem ... ok
test tests::naglowek_nie_jest_haslem ... ok
test tests::objasnienie_w_nawiasie_nie_jest_haslem ... ok
test tests::zachowuje_wszystkie_znaczenia ... ok
test tests::code_ma_dwoch_kandydatow ... ok
test tests::zapisuje_pozycje_i_dlugosc_listy ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/bin/implant.rs (target/debug/deps/implant-db8bf193e8831ea6)

running 10 tests
test tests::cbms_rows_large_slot_exceeds_u16 ... ok
test tests::cbms_rows_slot0_and_slot1 ... ok
test tests::ids_do_not_depend_on_embed_success ... ok
test tests::implant_rows_writes_both_plain_and_spaced ... ok
test tests::book_fixture_non_slot_lines_do_not_advance_symbol_slot ... ok
test tests::oob_vocab_fails_loud_not_silent_skip ... ok
test tests::projection_is_repeatable ... ok
test tests::projection_returns_unit_length ... ok
test tests::several_skipped_lines_do_not_desync_ids ... ok
test tests::projection_keeps_similar_things_similar ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running unittests src/bin/lathe.rs (target/debug/deps/lathe-446b45ccc55b5929)

running 12 tests
test tests::informacja_i_information_daja_to_samo ... ok
test tests::funkcja_i_function_daja_to_samo ... ok
test tests::forma_bez_koncowki_nie_przechodzi ... ok
test tests::c_zamienia_sie_zaleznie_od_nastepnej_litery ... ok
test tests::nieznane_zostaje_surowe_i_oznaczone ... ok
test tests::podwojone_spolgloski_znikaja ... ok
test tests::odrzuca_litery_spoza_alfabetu ... ok
test tests::slowo_bez_rozpoznanej_koncowki_idzie_do_niewiadomych ... ok
test tests::slowo_rodzime_wychodzi_bledne_i_to_jest_znane ... ok
test tests::slownik_bije_regule ... ok
test tests::wektor_i_vector_daja_to_samo ... ok
test tests::wpis_na_siebie_to_nie_trafienie ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running unittests src/bin/morph_coverage.rs (target/debug/deps/morph_coverage-5ccd07793fe0299e)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/bin/recall_ask.rs (target/debug/deps/recall_ask-f91223615dfdb7ac)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/bin/recall_find.rs (target/debug/deps/recall_find-e2346c513b6ee277)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/bin/reflex_store.rs (target/debug/deps/reflex_store-985b70d4a8b847c0)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/bin/spine.rs (target/debug/deps/spine-37b92af94b119a33)

running 4 tests
test tests::brak_trafien_to_sprawa_dla_mozgu ... ok
test tests::wyrazny_odstep_daje_odruch ... ok
test tests::odstep_liczony_w_obrebie_toru ... ok
test tests::wysoka_ocena_bez_odstepu_to_nie_odruch ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/bin/spine_loop.rs (target/debug/deps/spine_loop-1a625036060d237a)

running 9 tests
test tests::identyczne_bloki_nie_daja_roznicy ... ok
test tests::liczy_decyzje_o_danej_tresci ... ok
test tests::material_bierze_potwierdzone ... ok
test tests::material_pomija_niepotwierdzone ... ok
test tests::nieznany_blok_to_brak_nie_zgadywanie ... ok
test tests::przewaga_porazek_wyklucza_z_materialu ... ok
test tests::wstawienie_na_gorze_to_jedna_zmiana ... ok
test tests::zmiana_linii_to_usuniecie_i_dodanie ... ok
test tests::wyciaga_blok_liczac_klamry ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/bin/tokenize.rs (target/debug/deps/tokenize-c6f1630aa794ade8)

running 9 tests
test tests::canonical_book_error_does_not_emit_ids ... ok
test tests::canonical_empty_text_rejected_before_book_load ... ok
test tests::linia_bez_znaku_rownosci_jest_pomijana ... ok
test tests::pusty_znak_nie_tworzy_pojecia ... ok
test tests::canonical_output_matches_frontend_u32 ... ok
test tests::czyta_ksiege_pomijajac_naglowek ... ok
test tests::raw_exact_candidates_preserve_meanings_and_match_cbms ... ok
test tests::wordnet_morphy_lemmas_reach_exact_espdic ... ok
test tests::sentence_tokens_normalize_preserve_order_and_expose_oov ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.62s

     Running tests/chroma_parity.rs (target/debug/deps/chroma_parity-29e2d491a98c6afb)

running 3 tests
test the_engine_fits_the_shadow_socket ... ok
test empty_and_blank_text_behave_predictably ... ok
test vectors_match_what_chroma_computed ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/shadow_wiring.rs (target/debug/deps/shadow_wiring-b30ce876921b4168)

running 1 test
test the_student_watches_and_the_answer_stays_the_master_s ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/darkstar_recall-1a17fbf69ef0a857)

running 2 tests
test tests::missing_file_is_not_a_panic ... ok
test tests::env_value_is_read_by_exact_key ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/bin/recall_migrate.rs (target/debug/deps/recall_migrate-e6fecbdd99daedb3)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/bin/bak_pak_map_of_bak_files.rs (target/debug/deps/bak_pak_map_of_bak_files-478f6c17a24c0192)

running 11 tests
test tests::czas_liczony_poprawnie ... ok
test tests::identyczne_daja_pusto ... ok
test tests::kod_szukajacy_kluczy_nie_jest_sekretem ... ok
test tests::nazwa_bez_wartosci_nie_jest_sekretem ... ok
test tests::lapie_doslowne_przypisanie ... ok
test tests::lapie_znane_ksztalty_kluczy ... ok
test tests::nie_blokuje_poprawnego_odczytu_klucza ... ok
test tests::odrzuca_co_nie_ma_znacznika ... ok
test tests::rok_przestepny ... ok
test tests::rozklada_nazwe_kopii ... ok
test tests::wstawienie_linii_to_jedna_zmiana ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/main.rs (target/debug/deps/darkstar_seed-cafb60c5018f2049)

running 6 tests
test tests::frequent_symbols_get_a_louder_row ... ok
test tests::nwrd_round_trips_through_disk ... ok
test tests::scaling_touches_whole_rows_and_only_them ... ok
test tests::zero_strength_changes_nothing ... ok
test tests::wrong_magic_is_refused ... ok
test tests::no_multiplier_can_blow_the_weights_up ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running unittests src/bin/noworodek_embed.rs (target/debug/deps/noworodek_embed-38bfede393a6cd3c)

running 5 tests
test tests::empty_rows_are_skipped_too ... ok
test tests::identical_sentences_are_fully_aligned ... ok
test tests::hidden_size_is_read_from_the_data ... ok
test tests::result_has_unit_length ... ok
test tests::unknown_symbols_are_skipped_not_counted_as_zero ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/bin/nwrd_diff.rs (target/debug/deps/nwrd_diff-a0d66413463270e4)

running 3 tests
test tests::change_is_measured_where_it_happened ... ok
test tests::identical_weights_show_no_change ... ok
test tests::names_shorten_to_something_readable ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/main.rs (target/debug/deps/darkstar_server-1ae2715d8ee270a0)

running 14 tests
test http::run_stream::tests::subscribers_receive_published_run_event ... ok
test http::tests::demo_run_requires_bearer_token ... ok
test http::tests::live_events_require_bearer_token ... ok
test http::tests::health_is_public ... ok
test http::system_graph_view::tests::control_room_contains_operator_ui_anchors ... ok
test http::tests::session_creation_requires_bearer_token ... ok
test http::tests::system_graph_requires_bearer_token ... ok
test http::tests::valid_token_starts_demo_run ... ok
test tests::core_api_version_is_present ... ok
test http::tests::valid_token_creates_scoped_session ... ok
test tests::loopback_only_configuration_does_not_bind_twice ... ok
test http::tests::system_graph_is_available_with_valid_token ... ok
test tests::every_bind_address_names_one_interface ... ok
test tests::lan_bind_keeps_the_loopback_socket_alive ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/context_http.rs (target/debug/deps/context_http-60b6f514200eb6d5)

running 6 tests
test both_legs_default_to_chroma_v2_matching_the_live_host ... ok
test missing_token_is_refused ... ok
test both_legs_silent_gives_service_unavailable ... ok
test health_reports_both_legs ... ok
test search_returns_matching_collections ... ok
test mutating_methods_are_not_registered ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/frontend_http.rs (target/debug/deps/frontend_http-b454674ad51f685b)

running 14 tests
test http::system_graph_view::tests::control_room_contains_operator_ui_anchors ... ok
test http::tests::demo_run_requires_bearer_token ... ok
test api_stays_closed_while_world_landing_is_open ... ok
test http::run_stream::tests::subscribers_receive_published_run_event ... ok
test http::tests::health_is_public ... ok
test assets_path_serves_built_file ... ok
test http::tests::session_creation_requires_bearer_token ... ok
test http::tests::live_events_require_bearer_token ... ok
test http::tests::system_graph_requires_bearer_token ... ok
test http::tests::valid_token_starts_demo_run ... ok
test http::tests::valid_token_creates_scoped_session ... ok
test http::tests::system_graph_is_available_with_valid_token ... ok
test root_serves_control_room_html ... ok
test world_landing_is_served_without_a_token ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/gateway_http.rs (target/debug/deps/gateway_http-082ba9a9b7267df8)

running 5 tests
test unconfigured_token_fails_closed ... ok
test get_returns_live_gateway_status ... ok
test missing_token_is_refused ... ok
test unreadable_host_reports_service_unavailable ... ok
test mutating_methods_are_not_registered ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/git_http.rs (target/debug/deps/git_http-217fb8aa4a6b9d23)

running 4 tests
test overview_requires_the_operator_token ... ok
test missing_git_reports_service_unavailable_without_raw_command_output ... ok
test mutating_methods_are_not_registered ... ok
test overview_returns_structured_branch_state_and_ten_commits ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/headscale_http.rs (target/debug/deps/headscale_http-f1ad43e47179daaa)

running 5 tests
test a_missing_api_key_is_reported_rather_than_faked_as_zero_nodes ... ok
test health_and_nodes_are_read_together ... ok
test mutating_methods_are_not_registered ... ok
test headscale_down_reports_service_unavailable_without_raw_detail ... ok
test the_operator_token_is_still_required ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/loopback_http.rs (target/debug/deps/loopback_http-96884814a53393f3)

running 6 tests
test a_wrong_token_from_loopback_is_not_replaced ... ok
test gateway_address_does_not_receive_the_loopback_token ... ok
test loopback_without_authorization_is_allowed ... ok
test address_outside_loopback_still_needs_the_token ... ok
test an_unknown_peer_address_is_not_trusted ... ok
test without_a_configured_token_loopback_opens_nothing ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/memory_http.rs (target/debug/deps/memory_http-14749508766a1221)

running 16 tests
test http::system_graph_view::tests::control_room_contains_operator_ui_anchors ... ok
test http::run_stream::tests::subscribers_receive_published_run_event ... ok
test http::tests::health_is_public ... ok
test http::tests::demo_run_requires_bearer_token ... ok
test http::tests::live_events_require_bearer_token ... ok
test http::tests::session_creation_requires_bearer_token ... ok
test http::tests::system_graph_requires_bearer_token ... ok
test http::tests::system_graph_is_available_with_valid_token ... ok
test http::tests::valid_token_starts_demo_run ... ok
test http::tests::valid_token_creates_scoped_session ... ok
test memory_routes_require_authentication ... ok
test missing_api_token_fails_closed ... ok
test module_action_denies_missing_capability ... ok
test session_can_write_and_read_memory ... ok
test module_action_accepts_authorized_start_request ... ok
test module_action_requires_bearer_token ... ok

test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/world_http.rs (target/debug/deps/world_http-8800cb3ef1ef8817)

running 6 tests
test headplane_probe_target_is_pinned_to_loopback_never_all_interfaces ... ok
test mutating_methods_are_not_registered ... ok
test context_status_never_carries_collection_names_or_note_content ... ok
test public_get_reports_four_fresh_read_only_probes ... ok
test context_is_up_when_either_leg_answers_down_when_neither_does ... ok
test blocking_probes_do_not_starve_the_request_executor ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running unittests src/lib.rs (target/debug/deps/darkstar_shadow-ebbc5da5b92b7b0d)

running 14 tests
test journal::tests::missing_parent_directory_is_created ... ok
test journal::tests::reopening_appends_instead_of_erasing ... ok
test journal::tests::each_observation_is_exactly_one_line ... ok
test record::tests::identical_vectors_are_fully_aligned ... ok
test record::tests::action_round_trips_and_keeps_its_kind ... ok
test record::tests::opposite_vectors_are_fully_opposed ... ok
test record::tests::mismatched_dimensions_give_no_answer_instead_of_a_fake_one ... ok
test record::tests::record_round_trips_through_jsonl ... ok
test tests::a_failing_shadow_is_recorded_not_hidden ... ok
test tests::different_dimensions_are_recorded_without_a_fabricated_score ... ok
test tests::a_shadow_that_crashes_does_not_bring_the_system_down ... ok
test tests::agent_actions_are_recorded_even_without_a_shadow ... ok
test tests::without_a_shadow_nothing_is_observed ... ok
test tests::the_answer_comes_from_the_live_model_only ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests darkstar_core

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests darkstar_embed

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests darkstar_recall

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests darkstar_shadow

running 1 test
test crates/darkstar-shadow/src/lib.rs - ShadowedEmbedder (line 74) - compile fail ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s


```

### `cd /home/runner/work/polip-agi/polip-agi/frontend && npm ci`

```text

added 172 packages, and audited 173 packages in 3s

42 packages are looking for funding
  run `npm fund` for details

found 0 vulnerabilities

```

### `cd /home/runner/work/polip-agi/polip-agi/frontend && npm run lint`

```text

> frontend@0.0.0 lint
> eslint .


```

### `cd /home/runner/work/polip-agi/polip-agi/frontend && npm run test:pin`

```text

> frontend@0.0.0 test:pin
> node --experimental-strip-types --disable-warning=ExperimentalWarning --test tests/operatorPin.test.ts

TAP version 13
# Subtest: operator pin starts with exactly four cells
ok 1 - operator pin starts with exactly four cells
  ---
  duration_ms: 2.073791
  type: 'test'
  ...
# Subtest: incomplete operator pin does not authorize a request
ok 2 - incomplete operator pin does not authorize a request
  ---
  duration_ms: 0.344625
  type: 'test'
  ...
# Subtest: complete operator pin gets one bearer scheme prefix
ok 3 - complete operator pin gets one bearer scheme prefix
  ---
  duration_ms: 0.228758
  type: 'test'
  ...
# Subtest: pasted operator pin fills no more than four cells
ok 4 - pasted operator pin fills no more than four cells
  ---
  duration_ms: 0.255679
  type: 'test'
  ...
# Subtest: editing one operator pin cell preserves the other cells
ok 5 - editing one operator pin cell preserves the other cells
  ---
  duration_ms: 0.289502
  type: 'test'
  ...
1..5
# tests 5
# suites 0
# pass 5
# fail 0
# cancelled 0
# skipped 0
# todo 0
# duration_ms 190.298088

```

### `cd /home/runner/work/polip-agi/polip-agi/frontend && npm run test:git`

```text

> frontend@0.0.0 test:git
> node --experimental-strip-types --disable-warning=ExperimentalWarning --test tests/gitOverview.test.ts

TAP version 13
# Subtest: missing Git endpoint becomes an unavailable view instead of a raw error
ok 1 - missing Git endpoint becomes an unavailable view instead of a raw error
  ---
  duration_ms: 56.379721
  type: 'test'
  ...
# Subtest: Git overview is read from the authenticated darkstar-server endpoint
ok 2 - Git overview is read from the authenticated darkstar-server endpoint
  ---
  duration_ms: 5.913297
  type: 'test'
  ...
1..2
# tests 2
# suites 0
# pass 2
# fail 0
# cancelled 0
# skipped 0
# todo 0
# duration_ms 239.543643

```

## Files changed and reason

1. `/home/runner/work/polip-agi/polip-agi/crates/darkstar-embed/src/cbms_ids.rs`
   - Fixed compilation failure caused by use of removed/non-existent `Vocabulary::plain_id_for_symbol`.
   - Replaced with existing APIs: strict symbol presence check + single-id encode validation.
   - Updated test fallback so `cbms_ids` tests are hermetic when `NOWORODEK_BOOK`/`CBMS_BOOK` is absent.

2. `/home/runner/work/polip-agi/polip-agi/crates/darkstar-embed/src/bin/tokenize.rs`
   - Fixed test defects where unit tests hard-failed without `NOWORODEK_BOOK`/`CBMS_BOOK`.
   - Added hermetic fallback test book path/content for tests only.
   - Kept runtime behavior and public CLI/API unchanged; only test setup/expectations were stabilized.

3. `/home/runner/work/polip-agi/polip-agi/docs/operations/evidence/2026-09-16-build-baseline.md`
   - Added required evidence document with exact commands, exact output, environment, and change accounting.

## Still failing / skipped / unverified

- Still failing: **none** for required commands in scope.
- Skipped: no frontend `typecheck` script exists in `frontend/package.json`; therefore no standalone `npm run typecheck` command was available to execute.
- Unverified: CI on GitHub Actions for this exact commit is not included in this local evidence run; local workspace commands above are fully verified.
