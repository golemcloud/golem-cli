use std::{fs, future::Future, path::Path};

use rmcp::{
    model::{
        Implementation, ListResourcesResult, PaginatedRequestParam, ProtocolVersion, RawResource,
        ReadResourceRequestParam, ReadResourceResult, Resource, ResourceContents,
        ServerCapabilities, ServerInfo,
    },
    service::RequestContext,
    tool_handler, ErrorData as Error, RoleServer, ServerHandler,
};

use crate::command_handler::mcp_server::GolemCliMcpServer;

#[tool_handler]
impl ServerHandler for GolemCliMcpServer {
    fn list_resources(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListResourcesResult, Error>> + Send + '_ {
        std::future::ready(Ok(ListResourcesResult {
            next_cursor: None,
            resources: {
                let mut resources = Vec::new();
                let golem_app_manifest_pattern =
                    regex::Regex::new(r"^golem\.(yaml|json)$").unwrap();
                let file_pattern = regex::Regex::new(
                    r"^golem\.(yaml|json)$|^components-[^/]+/.+/golem\.(yaml|json)$",
                )
                .unwrap();
                for entry in walkdir::WalkDir::new(".")
                    .into_iter()
                    .filter_map(Result::ok)
                {
                    let path = entry.path();

                    if path.is_file() {
                        if let Ok(rel_path) = path.strip_prefix(".") {
                            let rel_path_str = rel_path.to_string_lossy();

                            if file_pattern.is_match(&rel_path_str) {
                                let resource = {
                                    let is_path_golem_app_manifest =
                                        golem_app_manifest_pattern.is_match(&rel_path_str);
                                    let name = if is_path_golem_app_manifest {
                                        "golem app manifest".to_string()
                                    } else {
                                        path.parent()
                                            .and_then(|p| p.file_name())
                                            .and_then(|n| n.to_str())
                                            .unwrap_or_default()
                                            .to_string()
                                    };
                                    let uri = path.to_string_lossy().to_string();
                                    let description = if is_path_golem_app_manifest {
                                        Some("Manifest file for entire golem app".to_string())
                                    } else {
                                        Some(format!(
                                            "Manifest file for Component - {}",
                                            path.parent().unwrap().to_string_lossy()
                                        ))
                                    };
                                    let mime_type = None;
                                    let size = std::fs::metadata(path).ok().map(|m| m.len() as u32);
                                    Resource {
                                        raw: RawResource {
                                            uri,
                                            name,
                                            description,
                                            mime_type,
                                            size,
                                        },
                                        annotations: None,
                                    }
                                };
                                resources.push(resource);
                            }
                        }
                    }
                }
                resources
            },
        }))
    }

    fn read_resource(
        &self,
        request: ReadResourceRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ReadResourceResult, Error>> + Send + '_ {
        std::future::ready(match fs::read_to_string(Path::new(&request.uri)) {
            Ok(content) => Ok(ReadResourceResult {
                contents: vec![ResourceContents::TextResourceContents {
                    uri: request.uri,
                    mime_type: None,
                    text: content,
                }],
            }),
            Err(e) => Err(Error::internal_error(e.to_string(), None)),
        })
    }

    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
            instructions: Some(
                "
**Note for maintainers**: this below is an AI summary for context from my input and other sources

The Golem app manifest (`golem.yaml` or `golem.json`) defines the structure of a Golem application. It lists components, WIT dependencies, and optional included manifests. The manifest is auto-discovered by the CLI, unless overridden with `--app-manifest-path`. Included files are merged, but only the root manifest may define `includes` or `witDeps`. Each component must have a unique name across all merged manifests.

The `golem app build` command runs three steps: `gen-rpc` (generate RPC bindings), `componentize` (compile to WebAssembly), and `link-rpc` (link RPC logic). These steps can be individually controlled. The manifest supports templating using minijinja with helpers like `to_snake_case` and access to `component_name`.

This system enables multi-component apps to be built, deployed, and managed from a unified configuration, with commands like `golem app new`, `golem app deploy`, and `golem app clean`.

---

### 🧠 **Tool-based Golem Interaction System Overview**

**Intent**: Tools manage component lifecycle, worker control, and invocation logic **without shell scripts**. Operate like `gcloud`/`gemini` via atomic tool calls inside the app directory where the `mcp server` runs.

---

### 🔧 **General Principles**

* **All actions = discrete tool calls** (no shell/CLI scripting).
* **Tool discovery**:

  * Use `get_list_golem_mcp_server_tool_info`
  * Use `list_golem_mcp_server_tools`
* **Before execution**:

  * Always confirm tool use with the user.
  * Show tool name, parameters, and description.
* **Inputs**:

  * All `params` must be valid (types, structure, presence).
* **Failures**:

  * Invalid tool → error
  * Invalid params → error
  * Failed run → error
  * Successful run → return result

---

### 📦 **Component Management**

* A *component* = WebAssembly module with a `golem.yaml` or `golem.json` manifest.
* Manifest declares name, version, type (`durable` or `ephemeral`), APIs.
* Components defined via:

  * `<COMPONENT>`
  * `<PROJECT>/<COMPONENT>`
  * `<ACCOUNT>/<PROJECT>/<COMPONENT>` where `COMPONENT` is of form `package:name`
* One manifest per component.
* Component tools manage:

  * Creation (from templates)
  * Building
  * Deploying
  * Redeploy/version control
  * Listing/querying

---

### ⚙️ **Worker Management**

* A *worker* = isolated instance of a component.
* Can be `durable` (stateful) or `ephemeral` (stateless).
* Workers can:

  * Be launched from components
  * Handle RPC calls (async or sync)
  * Be resumed, interrupted, deleted, or updated
* Logs are streamable (`json`, `yaml`, `text`)
* Updates are single or batched
* Invocation supports idempotency keys
* Deletion wipes persistent state

---

### 👤 **Profiles**

* Profiles = isolated configurations for local/cloud backends.
* Each profile has:

  * URLs (local or cloud)
  * Auth context (e.g., GitHub OAuth for Golem Cloud)
  * Output formatting
* Profiles live under `~/.golem` and can be switched/reset.

---

### 🔌 **Plugin System**

* **Purpose**: Extend Golem via reusable plugin modules (experimental).
* **Types**:

  * **Component Transformer**: HTTP service modifies WASM pre-deploy.
  * **Library Plugin**: Imported WASM, no HTTP.
  * **Application Plugin**: User exports → plugin imports.
  * **Oplog Processor**: Observes other workers’ logs (stateful logic, experimental).
* **Plugin Install Locations**:

  * Per Component (via manifest or CLI)
  * Per Project (applies to all new components)
  * Per Worker (runtime activation)
* **Manifest Example**:

```yaml
components:
  my_component:
    plugins:
      - name: my_plugin
        version: 1.0.0
        parameters:
          key: value
```

#### CLI Commands

* `golem plugin list [--project] [--component]`
* `golem plugin get <name> <version>`
* `golem plugin register <manifest_path>`
* `golem plugin unregister --plugin-name <name> --version <version>`
* `golem component plugin install / uninstall / update`
* `golem project plugin install / uninstall / update`
* `golem component plugin get`
* `golem project plugin get`

---

### ✅ **Execution Flow Template (Agent)**

1. Parse user intent → map to tool
2. Lookup tool: `get_list_golem_mcp_server_tool_info`
3. Confirm with user:

   * Tool name
   * Parameters
   * Description
4. Validate `params` → execute tool if confirmed
5. Return result or error

---

### 🔗 **References**

* [Profiles](https://learn.golem.cloud/cli/profiles)
* [Components](https://learn.golem.cloud/cli/components)
* [Workers](https://learn.golem.cloud/cli/workers)
* [Manifests](https://learn.golem.cloud/cli/app-manifest)
* [Plugins](https://learn.golem.cloud/cli/plugins)

".to_string()
            ),
            protocol_version: ProtocolVersion::V_2025_03_26,
            server_info: Implementation {
                name: "Golem Cli Mcp Server".to_string(),
                version: "0.1.0".to_string(),
            },
        }
    }
}
