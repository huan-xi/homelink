use axum::Router;
use axum::routing::{get, post, put};
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
            ,
        )
        .nest(
            "/hap_characteristic",
            Router::new()
                .route("/", post(controller::hap_characteristic::add))
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
        ).nest("/hap_accessory",
               Router::new()
                   .route("/list", get(controller::hap_accessory::list))
                   .route("/disable/:id", put(controller::hap_accessory::disable))
               ,
               // .route("/", post(controller::iot_device::add_device)),
    )
}