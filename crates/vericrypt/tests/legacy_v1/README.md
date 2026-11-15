These integration tests belong to the retired v1 `scan` pipeline. They are kept for reference but are NOT built:
Cargo only compiles files directly inside `tests/`, and `network_integration_test.rs` no longer compiled (it used a removed `ScanArgs` field).
The v2 engine is tested by `tests/engine_pipeline.rs` and `tests/engine_cli.rs`. Delete this folder once the v1 modules are removed (workstream W1).
