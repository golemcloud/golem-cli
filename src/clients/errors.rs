use golem_cloud_client::api::AccountError;
use golem_cloud_client::api::GrantError;
use golem_cloud_client::api::LoginError;
use golem_cloud_client::api::ProjectError;
use golem_cloud_client::api::ProjectGrantError;
use golem_cloud_client::api::ProjectPolicyError;
use golem_cloud_client::api::TemplateError;
use golem_cloud_client::api::TokenError;
use golem_cloud_client::api::WorkerError;

pub trait ResponseContentErrorMapper {
    fn map(self) -> String;
}

impl ResponseContentErrorMapper for AccountError {
    fn map(self) -> String {
        match self {
            AccountError::Error400(errors) => {
                format!("BadRequest: {errors:?}")
            }
            AccountError::Error401(error) => {
                format!("Unauthorized: {error:?}")
            }
            AccountError::Error404(error) => {
                format!("NotFound: {error:?}")
            }
            AccountError::Error500(error) => {
                format!("InternalError: {error:?}")
            }
        }
    }
}

impl ResponseContentErrorMapper for GrantError {
    fn map(self) -> String {
        match self {
            GrantError::Error400(errors) => {
                format!("BadRequest: {errors:?}")
            }
            GrantError::Error401(error) => {
                format!("Unauthorized: {error:?}")
            }
            GrantError::Error404(error) => {
                format!("NotFound: {error:?}")
            }
            GrantError::Error500(error) => {
                format!("InternalError: {error:?}")
            }
        }
    }
}

impl ResponseContentErrorMapper for LoginError {
    fn map(self) -> String {
        match self {
            LoginError::Error400(errors) => {
                format!("BadRequest: {errors:?}")
            }
            LoginError::Error401(error) => {
                format!("Unauthorized: {error:?}")
            }
            LoginError::Error500(error) => {
                format!("InternalError: {error:?}")
            }
        }
    }
}

impl ResponseContentErrorMapper for ProjectError {
    fn map(self) -> String {
        match self {
            ProjectError::Error400(errors) => {
                format!("BadRequest: {errors:?}")
            }
            ProjectError::Error401(error) => {
                format!("Unauthorized: {error:?}")
            }
            ProjectError::Error403(error) => {
                format!("Forbidden: {error:?}")
            }
            ProjectError::Error404(error) => {
                format!("NotFound: {error:?}")
            }
            ProjectError::Error500(error) => {
                format!("InternalError: {error:?}")
            }
        }
    }
}

impl ResponseContentErrorMapper for ProjectGrantError {
    fn map(self) -> String {
        match self {
            ProjectGrantError::Error400(errors) => {
                format!("BadRequest: {errors:?}")
            }
            ProjectGrantError::Error401(error) => {
                format!("Unauthorized: {error:?}")
            }
            ProjectGrantError::Error403(error) => {
                format!("Forbidden: {error:?}")
            }
            ProjectGrantError::Error404(error) => {
                format!("NotFound: {error:?}")
            }
            ProjectGrantError::Error500(error) => {
                format!("InternalError: {error:?}")
            }
        }
    }
}

#[allow(unreachable_patterns)]
impl ResponseContentErrorMapper for ProjectPolicyError {
    fn map(self) -> String {
        match self {
            ProjectPolicyError::Error400(errors) => {
                format!("BadRequest: {errors:?}")
            }
            ProjectPolicyError::Error401(error) => {
                format!("Unauthorized: {error:?}")
            }
            ProjectPolicyError::Error404(error) => {
                format!("NotFound: {error:?}")
            }
            ProjectPolicyError::Error500(error) => {
                format!("InternalError: {error:?}")
            }
            _ => "UnknownError".into(),
        }
    }
}

impl ResponseContentErrorMapper for TemplateError {
    fn map(self) -> String {
        match self {
            TemplateError::Error400(errors) => {
                format!("BadRequest: {errors:?}")
            }
            TemplateError::Error401(error) => {
                format!("Unauthorized: {error:?}")
            }
            TemplateError::Error403(error) => {
                format!("Forbidden: {error:?}")
            }
            TemplateError::Error404(error) => {
                format!("NotFound: {error:?}")
            }
            TemplateError::Error409(error) => {
                format!("Conflict: {error:?}")
            }
            TemplateError::Error500(error) => {
                format!("InternalError: {error:?}")
            }
        }
    }
}

impl ResponseContentErrorMapper for TokenError {
    fn map(self) -> String {
        match self {
            TokenError::Error400(errors) => {
                format!("BadRequest: {errors:?}")
            }
            TokenError::Error401(error) => {
                format!("Unauthorized: {error:?}")
            }
            TokenError::Error404(error) => {
                format!("NotFound: {error:?}")
            }
            TokenError::Error500(error) => {
                format!("InternalError: {error:?}")
            }
        }
    }
}

impl ResponseContentErrorMapper for WorkerError {
    fn map(self) -> String {
        match self {
            WorkerError::Error400(errors) => {
                format!("BadRequest: {errors:?}")
            }
            WorkerError::Error401(error) => {
                format!("Unauthorized: {error:?}")
            }
            WorkerError::Error403(error) => {
                format!("Forbidden: {error:?}")
            }
            WorkerError::Error404(error) => {
                format!("NotFound: {error:?}")
            }
            WorkerError::Error409(error) => {
                format!("Conflict: {error:?}")
            }
            WorkerError::Error500(error) => {
                format!("InternalError: {error:?}")
            }
        }
    }
}
