use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use strum_macros::Display;

// NonSuccessfulExit is used to signal that an error got resolved with hints or error messages
// already on the command line, thus nothing should be printed in the main error handler,
// but should return non-successful exit code from the process.
#[derive(Debug)]
pub struct NonSuccessfulExit;

impl Display for NonSuccessfulExit {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        //NOP
        Ok(())
    }
}

impl Error for NonSuccessfulExit {}

// Errors that should be handled by the command handler with showing hints or error messages
#[derive(Debug, Display)]
pub enum HintError {
    NoApplicationManifestFound,
    ExpectedCloudProfile, // TODO: maybe add project name for hints?
}

impl Error for HintError {}

pub mod service {
    use bytes::Bytes;
    use golem_client::model::{
        GolemError, GolemErrorComponentDownloadFailed, GolemErrorComponentParseFailed,
        GolemErrorFailedToResumeWorker, GolemErrorFileSystemError,
        GolemErrorGetLatestVersionOfComponentFailed, GolemErrorInitialComponentFileDownloadFailed,
        GolemErrorInterrupted, GolemErrorInvalidRequest, GolemErrorInvalidShardId,
        GolemErrorPromiseAlreadyCompleted, GolemErrorPromiseDropped, GolemErrorPromiseNotFound,
        GolemErrorRuntimeError, GolemErrorUnexpectedOplogEntry, GolemErrorUnknown,
        GolemErrorValueMismatch, GolemErrorWorkerAlreadyExists, GolemErrorWorkerCreationFailed,
        GolemErrorWorkerNotFound,
    };
    use golem_common::model::{PromiseId, WorkerId};
    use golem_wasm_rpc_stubgen::log::LogColorize;
    use itertools::Itertools;
    use reqwest::StatusCode;
    use std::error::Error;
    use std::fmt::{Display, Formatter};

    #[derive(Debug)]
    pub struct ServiceErrorResponse {
        status_code: u16,
        message: String,
    }

    pub trait HasServiceName {
        fn service_name() -> &'static str;
    }

    #[derive(Debug)]
    pub struct ServiceError {
        service_name: &'static str,
        kind: ServiceErrorKind,
    }

    #[derive(Debug)]
    pub enum ServiceErrorKind {
        ErrorResponse(ServiceErrorResponse),
        ReqwestError(reqwest::Error),
        ReqwestHeaderError(reqwest::header::InvalidHeaderValue),
        SerdeError(serde_json::Error),
        UnexpectedResponse { status_code: u16, payload: Bytes },
    }

    impl Display for ServiceError {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            fn format_status_code(status_code: u16) -> String {
                match StatusCode::from_u16(status_code) {
                    Ok(status_code) => status_code.to_string(),
                    Err(_) => status_code.to_string(),
                }
            }

            match &self.kind {
                ServiceErrorKind::ErrorResponse(response) => {
                    write!(
                        f,
                        "{} Service - Error: {}, {}",
                        self.service_name.log_color_highlight(),
                        format_status_code(response.status_code).log_color_error(),
                        response.message.log_color_warn()
                    )
                }
                ServiceErrorKind::ReqwestError(error) => {
                    write!(
                        f,
                        "{} Service - Network Error: {}",
                        self.service_name.log_color_highlight(),
                        error.to_string().log_color_warn()
                    )
                }
                ServiceErrorKind::ReqwestHeaderError(error) => {
                    write!(
                        f,
                        "{} Service - Header Error: {}",
                        self.service_name.log_color_highlight(),
                        error.to_string().log_color_warn()
                    )
                }
                ServiceErrorKind::SerdeError(error) => {
                    write!(
                        f,
                        "{} Service - Serialization Error: {}",
                        self.service_name.log_color_highlight(),
                        error.to_string().log_color_warn()
                    )
                }
                ServiceErrorKind::UnexpectedResponse {
                    status_code,
                    payload,
                } => {
                    write!(
                        f,
                        "{} Service - Unexpected Response Error: {}, {}",
                        self.service_name.log_color_highlight(),
                        format_status_code(*status_code).log_color_error(),
                        String::from_utf8_lossy(payload)
                            .to_string()
                            .log_color_warn()
                    )
                }
            }
        }
    }

    impl Error for ServiceError {}

    impl<T> From<golem_client::Error<T>> for ServiceError
    where
        T: Into<ServiceErrorResponse> + HasServiceName,
    {
        fn from(error: golem_client::Error<T>) -> Self {
            ServiceError {
                service_name: T::service_name(),
                kind: match error {
                    golem_client::Error::Item(error) => {
                        ServiceErrorKind::ErrorResponse(error.into())
                    }
                    golem_client::Error::Reqwest(error) => ServiceErrorKind::ReqwestError(error),
                    golem_client::Error::ReqwestHeader(error) => {
                        ServiceErrorKind::ReqwestHeaderError(error)
                    }
                    golem_client::Error::Serde(error) => ServiceErrorKind::SerdeError(error),
                    golem_client::Error::Unexpected { code, data } => {
                        ServiceErrorKind::UnexpectedResponse {
                            status_code: code,
                            payload: data,
                        }
                    }
                },
            }
        }
    }

    impl<T> From<golem_cloud_client::Error<T>> for ServiceError
    where
        T: Into<ServiceErrorResponse> + HasServiceName,
    {
        fn from(error: golem_cloud_client::Error<T>) -> Self {
            ServiceError {
                service_name: T::service_name(),
                kind: match error {
                    golem_cloud_client::Error::Item(error) => {
                        ServiceErrorKind::ErrorResponse(error.into())
                    }
                    golem_cloud_client::Error::Reqwest(error) => {
                        ServiceErrorKind::ReqwestError(error)
                    }
                    golem_cloud_client::Error::ReqwestHeader(error) => {
                        ServiceErrorKind::ReqwestHeaderError(error)
                    }
                    golem_cloud_client::Error::Serde(error) => ServiceErrorKind::SerdeError(error),
                    golem_cloud_client::Error::Unexpected { code, data } => {
                        ServiceErrorKind::UnexpectedResponse {
                            status_code: code,
                            payload: data,
                        }
                    }
                },
            }
        }
    }

    pub trait AnyhowMapServiceError<R> {
        fn map_service_error(self) -> anyhow::Result<R>;
    }

    impl<R, E> AnyhowMapServiceError<R> for Result<R, golem_client::Error<E>>
    where
        ServiceError: From<golem_client::Error<E>>,
    {
        fn map_service_error(self) -> anyhow::Result<R> {
            self.map_err(|err| ServiceError::from(err).into())
        }
    }

    impl<R, E> AnyhowMapServiceError<R> for Result<R, golem_cloud_client::Error<E>>
    where
        ServiceError: From<golem_cloud_client::Error<E>>,
    {
        fn map_service_error(self) -> anyhow::Result<R> {
            self.map_err(|err| ServiceError::from(err).into())
        }
    }

    impl HasServiceName for golem_client::api::ComponentError {
        fn service_name() -> &'static str {
            "Component"
        }
    }

    impl From<golem_client::api::ComponentError> for ServiceErrorResponse {
        fn from(value: golem_client::api::ComponentError) -> Self {
            match value {
                golem_client::api::ComponentError::Error400(errors) => ServiceErrorResponse {
                    status_code: 400,
                    message: errors.errors.join("\n"),
                },
                golem_client::api::ComponentError::Error401(error) => ServiceErrorResponse {
                    status_code: 401,
                    message: error.error,
                },
                golem_client::api::ComponentError::Error403(error) => ServiceErrorResponse {
                    status_code: 403,
                    message: error.error,
                },
                golem_client::api::ComponentError::Error404(error) => ServiceErrorResponse {
                    status_code: 404,
                    message: error.error,
                },
                golem_client::api::ComponentError::Error409(error) => ServiceErrorResponse {
                    status_code: 409,
                    message: error.error,
                },
                golem_client::api::ComponentError::Error500(error) => ServiceErrorResponse {
                    status_code: 500,
                    message: error.error,
                },
            }
        }
    }

    impl HasServiceName for golem_cloud_client::api::ComponentError {
        fn service_name() -> &'static str {
            "Cloud Component"
        }
    }

    impl From<golem_cloud_client::api::ComponentError> for ServiceErrorResponse {
        fn from(value: golem_cloud_client::api::ComponentError) -> Self {
            match value {
                golem_cloud_client::api::ComponentError::Error400(errors) => ServiceErrorResponse {
                    status_code: 400,
                    message: errors.errors.join("\n"),
                },
                golem_cloud_client::api::ComponentError::Error401(error) => ServiceErrorResponse {
                    status_code: 401,
                    message: error.error,
                },
                golem_cloud_client::api::ComponentError::Error403(error) => ServiceErrorResponse {
                    status_code: 403,
                    message: error.error,
                },
                golem_cloud_client::api::ComponentError::Error404(error) => ServiceErrorResponse {
                    status_code: 404,
                    message: error.error,
                },
                golem_cloud_client::api::ComponentError::Error409(error) => ServiceErrorResponse {
                    status_code: 409,
                    message: error.error,
                },
                golem_cloud_client::api::ComponentError::Error500(error) => ServiceErrorResponse {
                    status_code: 500,
                    message: error.error,
                },
            }
        }
    }

    impl HasServiceName for golem_client::api::WorkerError {
        fn service_name() -> &'static str {
            "Worker"
        }
    }

    impl From<golem_client::api::WorkerError> for ServiceErrorResponse {
        fn from(value: golem_client::api::WorkerError) -> Self {
            match value {
                golem_client::api::WorkerError::Error400(errors) => ServiceErrorResponse {
                    status_code: 400,
                    message: errors.errors.join("\n"),
                },
                golem_client::api::WorkerError::Error401(error) => ServiceErrorResponse {
                    status_code: 401,
                    message: error.error,
                },
                golem_client::api::WorkerError::Error403(error) => ServiceErrorResponse {
                    status_code: 403,
                    message: error.error,
                },
                golem_client::api::WorkerError::Error404(error) => ServiceErrorResponse {
                    status_code: 404,
                    message: error.error,
                },
                golem_client::api::WorkerError::Error409(error) => ServiceErrorResponse {
                    status_code: 409,
                    message: error.error,
                },
                golem_client::api::WorkerError::Error500(error) => ServiceErrorResponse {
                    status_code: 500,
                    message: display_golem_error(error.golem_error),
                },
            }
        }
    }

    impl HasServiceName for golem_cloud_client::api::WorkerError {
        fn service_name() -> &'static str {
            "Cloud Worker"
        }
    }

    impl From<golem_cloud_client::api::WorkerError> for ServiceErrorResponse {
        fn from(value: golem_cloud_client::api::WorkerError) -> Self {
            match value {
                golem_cloud_client::api::WorkerError::Error400(errors) => ServiceErrorResponse {
                    status_code: 400,
                    message: errors.errors.join("\n"),
                },
                golem_cloud_client::api::WorkerError::Error401(error) => ServiceErrorResponse {
                    status_code: 401,
                    message: error.error,
                },
                golem_cloud_client::api::WorkerError::Error403(error) => ServiceErrorResponse {
                    status_code: 403,
                    message: error.error,
                },
                golem_cloud_client::api::WorkerError::Error404(error) => ServiceErrorResponse {
                    status_code: 404,
                    message: error.error,
                },
                golem_cloud_client::api::WorkerError::Error409(error) => ServiceErrorResponse {
                    status_code: 409,
                    message: error.error,
                },
                golem_cloud_client::api::WorkerError::Error500(error) => ServiceErrorResponse {
                    status_code: 500,
                    message: display_golem_error(error.golem_error),
                },
            }
        }
    }

    impl HasServiceName for golem_cloud_client::api::ProjectError {
        fn service_name() -> &'static str {
            "Cloud Project"
        }
    }

    impl From<golem_cloud_client::api::ProjectError> for ServiceErrorResponse {
        fn from(value: golem_cloud_client::api::ProjectError) -> Self {
            match value {
                golem_cloud_client::api::ProjectError::Error400(errors) => ServiceErrorResponse {
                    status_code: 400,
                    message: errors.errors.join("\n"),
                },
                golem_cloud_client::api::ProjectError::Error401(error) => ServiceErrorResponse {
                    status_code: 401,
                    message: error.error,
                },
                golem_cloud_client::api::ProjectError::Error403(error) => ServiceErrorResponse {
                    status_code: 403,
                    message: error.error,
                },
                golem_cloud_client::api::ProjectError::Error404(error) => ServiceErrorResponse {
                    status_code: 404,
                    message: error.error,
                },
                golem_cloud_client::api::ProjectError::Error500(error) => ServiceErrorResponse {
                    status_code: 500,
                    message: error.error,
                },
            }
        }
    }

    // TODO: re-add callstack highlighting here?
    pub fn display_golem_error(error: GolemError) -> String {
        match error {
            GolemError::InvalidRequest(GolemErrorInvalidRequest { details }) => {
                format!("Invalid request: {details}")
            }
            GolemError::WorkerAlreadyExists(GolemErrorWorkerAlreadyExists { worker_id }) => {
                format!(
                    "Worker already exists: {}",
                    display_worker_id(worker_id).log_color_highlight()
                )
            }
            GolemError::WorkerNotFound(GolemErrorWorkerNotFound { worker_id }) => {
                format!(
                    "Worker not found: {}",
                    display_worker_id(worker_id).log_color_highlight()
                )
            }
            GolemError::WorkerCreationFailed(GolemErrorWorkerCreationFailed {
                worker_id,
                details,
            }) => {
                format!(
                    "Failed to create worker {}: {}",
                    display_worker_id(worker_id).log_color_highlight(),
                    details
                )
            }
            GolemError::FailedToResumeWorker(inner) => {
                let GolemErrorFailedToResumeWorker { worker_id, reason } = *inner;
                format!(
                    "Failed to resume worker {}: {}",
                    display_worker_id(worker_id).log_color_highlight(),
                    display_golem_error(reason).log_color_warn()
                )
            }
            GolemError::ComponentDownloadFailed(GolemErrorComponentDownloadFailed {
                component_id,
                reason,
            }) => {
                format!(
                    "Failed to download component {}@{}: {}",
                    component_id.component_id.to_string().log_color_highlight(),
                    component_id.version.to_string().log_color_highlight(),
                    reason.log_color_warn()
                )
            }
            GolemError::ComponentParseFailed(GolemErrorComponentParseFailed {
                component_id,
                reason,
            }) => {
                format!(
                    "Failed to parse component {}@{}: {}",
                    component_id.component_id.to_string().log_color_highlight(),
                    component_id.version.to_string().log_color_highlight(),
                    reason.log_color_warn()
                )
            }
            GolemError::GetLatestVersionOfComponentFailed(
                GolemErrorGetLatestVersionOfComponentFailed {
                    component_id,
                    reason,
                },
            ) => {
                format!(
                    "Failed to get latest version of component {}: {}",
                    component_id.to_string().log_color_highlight(),
                    reason.log_color_warn()
                )
            }
            GolemError::PromiseNotFound(GolemErrorPromiseNotFound { promise_id }) => {
                format!(
                    "Promise not found: {}",
                    display_promise_id(promise_id).log_color_highlight()
                )
            }
            GolemError::PromiseDropped(GolemErrorPromiseDropped { promise_id }) => {
                format!(
                    "Promise dropped: {}",
                    display_promise_id(promise_id).log_color_highlight()
                )
            }
            GolemError::PromiseAlreadyCompleted(GolemErrorPromiseAlreadyCompleted {
                promise_id,
            }) => {
                format!(
                    "Promise already completed: {}",
                    display_promise_id(promise_id).log_color_highlight()
                )
            }
            GolemError::Interrupted(GolemErrorInterrupted {
                recover_immediately,
            }) => {
                if recover_immediately {
                    "Simulated crash".to_string()
                } else {
                    "Worker interrupted".to_string()
                }
            }
            GolemError::ParamTypeMismatch(_) => "Parameter type mismatch".to_string(),
            GolemError::NoValueInMessage(_) => "No value in message".to_string(),
            GolemError::ValueMismatch(GolemErrorValueMismatch { details }) => {
                format!("Parameter value mismatch: {}", details.log_color_warn())
            }
            GolemError::UnexpectedOplogEntry(GolemErrorUnexpectedOplogEntry { expected, got }) => {
                format!(
                    "Unexpected oplog entry: expected {}, got {}",
                    expected.log_color_highlight(),
                    got.log_color_warn()
                )
            }
            GolemError::RuntimeError(GolemErrorRuntimeError { details }) => {
                format!("Runtime error: {}", details)
            }
            GolemError::InvalidShardId(GolemErrorInvalidShardId {
                shard_id,
                shard_ids,
            }) => {
                format!(
                    "Invalid shard id: {} not in [{}]",
                    shard_id,
                    shard_ids.iter().join(", ")
                )
            }
            GolemError::PreviousInvocationFailed(_) => {
                "The previously invoked function failed".to_string()
            }
            GolemError::PreviousInvocationExited(_) => {
                "The previously invoked function exited".to_string()
            }
            GolemError::Unknown(GolemErrorUnknown { details }) => {
                format!("Unknown error: {}", details)
            }
            GolemError::InvalidAccount(_) => "Invalid account".to_string(),
            GolemError::ShardingNotReady(_) => "Sharding not ready".to_string(),
            GolemError::InitialComponentFileDownloadFailed(
                GolemErrorInitialComponentFileDownloadFailed { path, reason, .. },
            ) => {
                format!(
                    "Failed to download initial file {}: {}",
                    path.log_color_highlight(),
                    reason.log_color_warn()
                )
            }
            GolemError::FileSystemError(GolemErrorFileSystemError { path, reason, .. }) => {
                format!(
                    "File system error: {}, {}",
                    path.log_color_highlight(),
                    reason.log_color_warn()
                )
            }
        }
    }

    pub fn display_worker_id(worker_id: WorkerId) -> String {
        format!("{}/{}", worker_id.component_id, worker_id.worker_name)
    }

    pub fn display_promise_id(promise_id: PromiseId) -> String {
        format!(
            "{}/{}",
            display_worker_id(promise_id.worker_id),
            promise_id.oplog_idx
        )
    }
}
