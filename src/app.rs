use std::sync::Arc;
use crate::application::auth_usecase::AuthUseCase;
use crate::config::settings::Settings;

#[derive(Clone)]
pub struct AppState {
    pub auth_usecase: Arc<AuthUseCase>,
    pub settings: Settings,
}
