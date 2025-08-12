# Resources: Manifest Discovery and Reading

Targets
- `golem.yaml` (primary manifest). Keep room to add more recognized files later.

Discovery strategy
- Current working directory.
- Ancestors up to filesystem root.
- One-level children (directories only) to avoid deep scans.

Implementation
- Canonicalize and deduplicate file paths.
- Register each as an MCP resource with a stable URI (e.g., `file://...`) and title.
- `resources.read` returns file content with MIME `application/yaml`.

Reusability
- Optionally reuse parts of `ApplicationContext` discovery where lightweight.
- Keep listing cheap and robust; avoid heavy parsing during listing. 