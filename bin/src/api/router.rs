use axum::Router;
use axum::routing::{delete, get, post, put};
use crate::api::controller;
use crate::api::state::AppState;

pub fn api() -> Router<AppState> {
    Router::new()
        .nest(
            "/hap_service",
            Router::new()
                .route("/", post(controller::hap_service::add_service))
                .route("/list/:id", get(controller::hap_service::list))
                .route("/disable/:id", put(controller::hap_service::disable))
                .route("/:id", delete(controller::hap_service::delete))
            ,
        )
        .nest(
            "/hap_characteristic",
            Router::new()
                .route("/", post(controller::hap_characteristic::add))
                .route("/:id", put(controller::hap_characteristic::update))
                .route("/:id", delete(controller::hap_characteristic::delete))
                .route("/list/:id", get(controller::hap_characteristic::list))
                .route("/disable/:id", put(controller::hap_characteristic::disable))

            ,
        )
        .nest("/iot_device",
              Router::new()
                  .route("/list", get(controller::iot_device::list))
                  .route("/disable/:id", put(controller::iot_device::disable))
              ,
              // .route("/", post(controller::iot_device::add_device)),
        )
        .nest("/hap_accessory",
              Router::new()
                  .route("/list", get(controller::hap_accessory::list))
                  .route("/disable/:id", put(controller::hap_accessory::disable))
              ,
              // .route("/", post(controller::iot_device::add_device)),
        )
        .nest("/hap_metadata",
              Router::new()
                  .route("/characteristic_meta/:hap_type", get(controller::hap_metadata::get_characteristic_meta))
                  .route("/service_meta/:hap_type", get(controller::hap_metadata::get_service_meta))
              ,
        )
}