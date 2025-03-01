use crate::cloud::clients::certificate::CertificateClient;
use crate::cloud::model::text::certificate::{CertificateAddView, CertificateListView};
use crate::cloud::model::ProjectRef;
use crate::cloud::service::project::ProjectService;
use async_trait::async_trait;
use golem_cli::model::{GolemError, GolemResult, PathBufOrStdin};
use golem_cloud_client::model::CertificateRequest;
use std::fs::File;
use std::io;
use std::io::{BufReader, Read};
use std::sync::Arc;
use uuid::Uuid;

#[async_trait]
pub trait CertificateService {
    async fn get(
        &self,
        project_ref: ProjectRef,
        certificate_id: Option<Uuid>,
    ) -> Result<GolemResult, GolemError>;
    async fn add(
        &self,
        project_ref: ProjectRef,
        domain_name: String,
        certificate_body: PathBufOrStdin,
        certificate_private_key: PathBufOrStdin,
    ) -> Result<GolemResult, GolemError>;
    async fn delete(
        &self,
        project_ref: ProjectRef,
        certificate_id: Uuid,
    ) -> Result<GolemResult, GolemError>;
}

pub struct CertificateServiceLive {
    pub client: Box<dyn CertificateClient + Send + Sync>,
    pub projects: Arc<dyn ProjectService + Send + Sync>,
}

fn read_as_string<R: Read>(mut r: R, source: &str) -> Result<String, GolemError> {
    let mut result = String::new();

    r.read_to_string(&mut result)
        .map_err(|e| GolemError(format!("Failed to read {source} as String: ${e}")))?;

    Ok(result)
}

fn read_path_or_stdin_as_string(path_or_stdin: PathBufOrStdin) -> Result<String, GolemError> {
    match path_or_stdin {
        PathBufOrStdin::Path(path) => {
            let file = File::open(&path)
                .map_err(|e| GolemError(format!("Failed to open file {path:?}: {e}")))?;

            let reader = BufReader::new(file);

            read_as_string(reader, &format!("file `{path:?}`"))
        }
        PathBufOrStdin::Stdin => read_as_string(io::stdin(), "stdin"),
    }
}

#[async_trait]
impl CertificateService for CertificateServiceLive {
    async fn get(
        &self,
        project_ref: ProjectRef,
        certificate_id: Option<Uuid>,
    ) -> Result<GolemResult, GolemError> {
        let project_urn = self.projects.resolve_urn_or_default(project_ref).await?;

        let res = self
            .client
            .get(project_urn, certificate_id.as_ref())
            .await?;

        Ok(GolemResult::Ok(Box::new(CertificateListView(res))))
    }

    async fn add(
        &self,
        project_ref: ProjectRef,
        domain_name: String,
        certificate_body: PathBufOrStdin,
        certificate_private_key: PathBufOrStdin,
    ) -> Result<GolemResult, GolemError> {
        let project_urn = self.projects.resolve_urn_or_default(project_ref).await?;

        let request = CertificateRequest {
            project_id: project_urn.id.0,
            domain_name,
            certificate_body: read_path_or_stdin_as_string(certificate_body)?,
            certificate_private_key: read_path_or_stdin_as_string(certificate_private_key)?,
        };

        let res = self.client.create(request).await?;

        Ok(GolemResult::Ok(Box::new(CertificateAddView(res))))
    }

    async fn delete(
        &self,
        project_ref: ProjectRef,
        certificate_id: Uuid,
    ) -> Result<GolemResult, GolemError> {
        let project_urn = self.projects.resolve_urn_or_default(project_ref).await?;
        let res = self.client.delete(project_urn, &certificate_id).await?;
        Ok(GolemResult::Str(res))
    }
}
