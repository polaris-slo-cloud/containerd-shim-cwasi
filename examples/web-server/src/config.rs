use actix_web::web::{Data, ServiceConfig};
use log::info;

/// Run custom configuration as part of the application building
/// process.
///
/// This function should contain all custom configuration for your function application.
///
/// ```rust
/// fn configure(cfg: &mut ServiceConfig) {
///     let db_driver = my_db();
///     cfg.data(db_driver.clone());
/// }
/// ```
///
/// Then you can use configured resources in your function.
///
/// ```rust
/// pub async fn index(
///     req: HttpRequest,
///     driver: Data<DbDriver>,
/// ) -> HttpResponse {
///     HttpResponse::NoContent()
/// }
pub fn configure(cfg: &mut ServiceConfig) {
    info!("Configuring service");
    cfg.app_data(Data::new(HandlerConfig::default()));
}

/// An example of the function configuration structure.
#[derive(Clone)]
pub struct HandlerConfig {
    pub name: String,
}

impl Default for HandlerConfig {
    fn default() -> HandlerConfig {
        HandlerConfig {
            name: "world".into(),
        }
    }
}
