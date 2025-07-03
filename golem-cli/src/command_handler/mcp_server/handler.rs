use std::{fs, future::Future, path::Path };

use rmcp::{
    model::{
        Implementation, ListResourcesResult, PaginatedRequestParam, ProtocolVersion, RawResource,
        ReadResourceRequestParam, ReadResourceResult, Resource, ResourceContents,
        ServerCapabilities, ServerInfo,
    },
    service::RequestContext,
    tool_handler, Error, RoleServer, ServerHandler,
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
        std::future::ready(
            match fs::read_to_string(Path::new(&request.uri)) {
                Ok(content) => Ok(ReadResourceResult {
                    contents: vec![ResourceContents::TextResourceContents {
                        uri: request.uri,
                        mime_type: None,
                        text: content,
                    }],
                }),
                Err(e) => Err(Error::internal_error(e.to_string(), None)),
            },
        )
    }

    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
            instructions: Some(
                "Help user to perform what normally they can directly do using golem cli".to_string(),
            ),
            protocol_version: ProtocolVersion::V_2025_03_26,
            server_info: Implementation {
                name: "Golem Cli Mcp Server".to_string(),
                version: "0.1.0".to_string(),
            },
        }
    }
}
