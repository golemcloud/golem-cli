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
                Always prefer to use tools not commands line scripts or manual commands.
                Tools Introduction
                Helps interacting with Golem. It allows users to upload their components, launch new workers based on these components and call functions on running workers, etc..,

                Call get_list_golem_mcp_server_tool_info, list_golem_mcp_server_tools tools to know detailed info about available tools before making a call.
                Ask user confirmation before making a call to any tool. very import. Show the user the tool name, arguments, and description.

                User uses these tools like they normally use gemini cli but via claude code or similar tools inside the golem app directory where the mcp server is running.
                
                📎 NOTES:
                    - User requests should be interpreted and mapped internally to tool calls.
                    - AI agents must ensure the `params` object includes all required keys and correct types and number of arguments.
                    - If the user requests a tool that is not available, return an error.
                    - If the user requests a tool that is available, but with incorrect parameters, return an error.
                    - If the user requests a tool that is available, but with correct parameters, execute the tool and return the result.
                    - If the user requests a tool that is available, but with correct parameters, but the tool execution fails, return an error.
                    - If the user requests a tool that is available, but with correct parameters, and the tool execution succeeds, return the result.
                    - No interactive shell exists. Every action is an atomic tool invocation. Ask for confirmation before calling a tool
                    - Golem app, golem app has components, components have manifests, manifests are yaml or json files, manifests are used to describe the component and its dependencies, 
                    - manifests are used to launch the component, manifests are used to manage the components
                    - components are defined or resolved in three ways:
                            /// Accepted formats:
                            ///   - <COMPONENT>
                            ///   - <PROJECT>/<COMPONENT>
                            ///   - <ACCOUNT>/<PROJECT>/<COMPONENT>
                            /// WHERE COMPONENT is the name of form <package>:<name>, e.g. golem:my_component
                    - each account of user can have multiple projects, each project can have multiple components
                    - one manifest file for each component, manifest file is named golem.yaml or golem.json
                    - one manifest for app/project.

                    For more details about Golem CLI tools, see the documentation at learn.golem.cloud/cli (Note: below are generated with ai)
                    1. profiles - 
                        Golem CLI profiles system enables isolated configurations for different Golem environments (e.g., cloud vs. local). 
                        It supports interactive and scripted setup, profile switching, authentication for cloud access via GitHub OAuth, and output formatting. 
                        Profiles are stored in ~/.golem and can be reset by deleting that directory. Useful for managing multi-environment workflows via named profiles. 
                        Full CLI command reference and usage examples are available here: learn.golem.cloud/cli/profiles.

                    2. components:
                        Golem CLI components are WebAssembly modules you build, deploy, and version. 
                        You can generate components from templates, build them locally, deploy to Golem, and inspect metadata like exports and version. 
                        Deployments are durable by default but can be set as ephemeral. 
                        Versioning enables updates and redeploys without breaking references. 
                        Components can be listed, retrieved by name/version, and connected to worker logic. 
                        Use golem component, golem app, and related subcommands for management.

                        More details: learn.golem.cloud/cli/components

                    3. workers - 
                        Golem CLI workers are isolated WebAssembly instances that can be created (durable or ephemeral), listed, inspected, invoked (async or sync), updated, suspended, resumed, or deleted. 
                        Each worker has state, metadata, and logs. 
                        Invocations support idempotency keys, async queuing, and argument passing. Updates can target single or multiple workers via filters. 
                        Logs are streamable in various formats. Durable workers persist state; ephemeral ones restart stateless. 
                        Deletion wipes state. Listing is slow at scale.
                        More details: learn.golem.cloud/cli/workers

                End.
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
