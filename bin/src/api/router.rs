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
            "/users",
            Router::new()
                .route("/login", post(controller::sys_users::login))
                .route("/info", get(controller::sys_users::info)),
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
                  .route("/restart/:id", put(controller::iot_device::restart))
                  .route("/set_property/:id", post(controller::iot_device::set_property))
                  .route("/read_property/:id", post(controller::iot_device::read_property))
              // .route("/", post(controller::iot_device::add_device))
              ,
        )
        .nest("/hap_bridge",
              Router::new()
                  .route("/list", get(controller::hap_bridge::list))
                  .route("/disable/:id", put(controller::hap_bridge::disable))
                  .route("/restart/:id", put(controller::hap_bridge::restart))
                  .route("/", post(controller::hap_bridge::add))
              ,
        )
        .nest("/hap_accessory",
              Router::new()
                  .route("/list", get(controller::hap_accessory::list))
                  .route("/", post(controller::hap_accessory::add))
                  .route("/:id", get(controller::hap_accessory::detail))
                  .route("/disable/:id", put(controller::hap_accessory::disable))
                  .route("/:id", put(controller::hap_accessory::update))
              ,
        )
        .nest("/test", Router::new()
            .route("/ping_js", get(controller::test::ping_js),
            ))
        .nest("/miot_device",
              Router::new()

                  .route("/list", get(controller::miot_device::list))
                  .route("/convert", post(controller::miot_device::convert_to_iot_device))
                  .route("/handshake", post(controller::miot_device::handshake))
                  .route("/accounts", get(controller::miot_device::accounts))
                  .route("/account", post(controller::miot_device::add_account))
                  .route("/account/login", post(controller::miot_device::login))
                  .route("/account", put(controller::miot_device::update_account))
                  .route("/account/sync_mi_devices", get(controller::miot_device::sync_mi_devices))
              // .route("/update_iot_device", get(controller::miot_spec::update_iot_device))
              // .route("/login_mi_account", get(controller::miot_spec::login_mi_account))
              // .route("/delete_device", get(controller::miot_spec::delete_device))
              ,
        )
        .nest("/hap_metadata",
              Router::new()
                  .route("/characteristic_meta/:hap_type", get(controller::hap_metadata::get_characteristic_meta))
                  .route("/service_meta/:hap_type", get(controller::hap_metadata::get_service_meta))
              ,
        )
}