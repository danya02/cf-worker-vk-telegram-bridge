use axum::response::IntoResponse;

pub struct AppError(pub anyhow::Error);

impl<T> From<T> for AppError
where
    T: Into<anyhow::Error>,
{
    fn from(error: T) -> Self {
        Self(error.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            self.0.to_string(),
        )
            .into_response()
    }
}

pub type AppResult<T> = std::result::Result<T, AppError>;
