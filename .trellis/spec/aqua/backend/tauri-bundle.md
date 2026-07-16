# Tauri 2 Bundle: Resource Path Resolution

> How to locate bundled resources (connector.jar, builtin-biztypes.json) with correct dev/packaged path parity.

## Scenario: Resolving a bundled resource path

### 1. Scope / Trigger
- Trigger: any code that reads a file shipped as a Tauri bundle `resource` (infra integration).
- Applies to the `src-tauri` shell layer only. `aqua-core` must NOT depend on Tauri — paths are resolved in commands and passed in as `&str`.

### 2. Signatures
```rust
// src-tauri/src/commands/import.rs
use tauri::{AppHandle, Manager, Runtime};
use tauri::path::BaseDirectory;

fn connector_jar_path<R: Runtime>(app: &AppHandle<R>) -> Result<String, String> {
    let p = app.path()
        .resolve("resources/connector.jar", BaseDirectory::Resource)
        .map_err(|e| format!("定位 connector.jar 失败: {e}"))?;
    p.to_str().map(|s| s.to_string())
        .ok_or_else(|| "connector.jar 路径含非法字符".into())
}
```

### 3. Contracts
- `tauri.conf.json` `bundle.resources` entries keep their path prefix verbatim: `["resources/builtin-biztypes.json", "resources/connector.jar"]`.
- Packaged (macOS): file lands at `<app.app>/Contents/Resources/resources/<file>` — the `resources/` prefix is preserved.
- Dev: `BaseDirectory::Resource` resolves to `src-tauri/`, so `resolve("resources/<file>", Resource)` -> `src-tauri/resources/<file>` (same location, dev/packaged parity).
- The resolved path is passed to `aqua-core` as a plain `&str` (`create_driver(config, drivers_dir, connector_path)`); `aqua-core` never calls Tauri APIs.

### 4. Validation & Error Matrix
| Condition | Result |
|---|---|
| Packaged, resource exists | absolute path under resource_dir |
| Dev, `src-tauri/resources/<file>` exists | absolute path under src-tauri |
| `resolve` fails (rare) | `Err("定位 connector.jar 失败: ...")`; downstream JDBC spawn then fails with a JDK-misleading message — acceptable defensive fallback |
| Path contains non-UTF8 | `Err("...路径含非法字符")` |

### 5. Good / Base / Bad Cases
- **Good**: `resolve("resources/connector.jar", BaseDirectory::Resource)` — matches the `resources/` prefix in `bundle.resources`, dev/packaged identical.
- **Base**: `builtin.rs` resolves `resources/builtin-biztypes.json` the same way — the canonical pattern to copy.
- **Bad**: `app.path().resource_dir()?.join("connector.jar")` — drops the `resources/` prefix; packaged path is off by one level -> file not found.

### 6. Tests Required
- Integration (`--ignored`): `h2_test_connection` exercises `create_driver -> check_java -> spawn connector.jar -> H2` end-to-end with the resolved path.
- Manual E2E: `pnpm tauri build` -> mount dmg -> assert `aqua.app/Contents/Resources/resources/connector.jar` exists.

### 7. Wrong vs Correct
#### Wrong
```rust
// drops the "resources/" prefix; packaged path misses by one level
let jar = app.path().resource_dir()?.join("connector.jar");
```
#### Correct
```rust
// prefix matches bundle.resources entry; dev/packaged parity
let jar = app.path().resolve("resources/connector.jar", BaseDirectory::Resource)?;
```

---

## Convention: Built-in (read-only) vs external (writable) paths

**What**: Two distinct path roots, never mixed.
- Built-in bundle artifacts (`connector.jar`, `builtin-biztypes.json`) -> `BaseDirectory::Resource` (read-only after packaging).
- User-supplied external files (JDBC driver jars, `databases.json`) -> `app_data_dir()` (writable; `commands/database.rs:drivers_dir()`).

**Why**: Packaged resources are read-only; users must be able to add/remove their own JDBC drivers at runtime. Mixing them puts user files in a read-only location.

**Example**:
```rust
// built-in, read-only
let jar = app.path().resolve("resources/connector.jar", BaseDirectory::Resource)?;
// external, writable
let drivers = app.path().app_data_dir()?.join("drivers");
```

**Related**: `docs/packaging.md`, `commands/database.rs`.

---

## Pattern: `--if-missing` for heavy dev prerequisites

**Problem**: `beforeDevCommand` runs on every `tauri dev`. A heavy build step (Maven `package`, several seconds) on every dev launch is wasteful, but skipping it entirely means a fresh clone can't connect to JDBC data sources until the developer remembers to build manually.

**Solution**: Wrap the heavy build in an `--if-missing` script that runs the full build only when the artifact is absent; otherwise a cheap `existsSync` check exits 0.

```jsonc
// tauri.conf.json
"beforeDevCommand": "pnpm build:connector:if-missing && pnpm --filter @aqua/app dev"
```
```js
// scripts/build-connector.mjs
if (existsSync(target) && !process.argv.includes('--force')) {
  console.log('已存在,跳过'); process.exit(0);
}
execSync('mvn -q package', { cwd: connectorDir, stdio: 'inherit' });
// copy target/connector.jar -> src-tauri/resources/connector.jar
```

**Why**: Fresh clone -> builds once automatically; subsequent dev launches skip Maven. `beforeBuildCommand` still runs the full build (no `--if-missing`) so releases always ship a fresh jar.

**Related**: `docs/packaging.md` §7.
