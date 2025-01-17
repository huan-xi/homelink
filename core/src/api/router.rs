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
            "/system",
            Router::new()
                .route("/restart", post(controller::system::restart)),
        )
        .nest(
            "/hap_characteristic",
            Router::new()
                .route("/", post(controller::hap_characteristic::add))
                .route("/", put(controller::hap_characteristic::update))
                .route("/:id", delete(controller::hap_characteristic::delete))
                .route("/list/:id", get(controller::hap_characteristic::list))
                .route("/disable/:id", put(controller::hap_characteristic::disable))

            ,
        )
        .nest("/source_device", Router::new()
            .route("/list", get(controller::source_device::list)))
        .nest("/iot_device",
              Router::new()
                  .route("/list", get(controller::iot_device::list))
                  .route("/disable/:id", put(controller::iot_device::disable))
                  .route("/restart/:id", put(controller::iot_device::restart))
                  .route("/:id", delete(controller::iot_device::delete))
                  .route("/set_property/:id", post(controller::iot_device::set_property))
                  .route("/read_property/:id", post(controller::iot_device::read_property))
                  .route("/", put(controller::iot_device::edit_device))
              ,
        )
        .nest("/hap_bridge",
              Router::new()
                  .route("/:id", get(controller::hap_bridge::get_detail))
                  .route("/template/:id", get(controller::hap_bridge::get_template))
                  .route("/template", post(controller::hap_bridge::add_by_template))
                  .route("/template", put(controller::hap_bridge::update_by_template))
                  .route("/list", get(controller::hap_bridge::list))
                  .route("/disable/:id", put(controller::hap_bridge::disable))
                  .route("/restart/:id", put(controller::hap_bridge::restart))
                  .route("/reset/:id", put(controller::hap_bridge::reset))
                  .route("/:id", delete(controller::hap_bridge::delete))
                  .route("/accessories_json/:id", get(controller::hap_bridge::accessories_json))
                  .route("/", post(controller::hap_bridge::add))
                  .route("/", put(controller::hap_bridge::update))
              ,
        )
        .nest("/hap_accessory",
              Router::new()
                  .route("/list", get(controller::hap_accessory::list))
                  .route("/", post(controller::hap_accessory::add))
                  .route("/", put(controller::hap_accessory::update))
                  .route("/template/:id", get(controller::hap_accessory::get_template))
                  .route("/template", put(controller::hap_accessory::update_by_template))
                  .route("/:id", get(controller::hap_accessory::detail))
                  .route("/disable/:id", put(controller::hap_accessory::disable))
                  .route("/:id", delete(controller::hap_accessory::delete))
                  .route("/:id", put(controller::hap_accessory::update))
              ,
        )
        .nest("/test", Router::new()
            .route("/ping_js", get(controller::test::ping_js),
            ))
        .nest("/miot_device",
              Router::new()
                  .route("/list", get(controller::miot_device::list))
                  .route("/templates/:model", get(controller::miot_device::templates))
                  .route("/access", post(controller::miot_device::access))
                  .route("/convert_by_template", post(controller::miot_device::convert_by_template))
                  .route("/handshake", post(controller::miot_device::handshake))
                  .route("/accounts", get(controller::miot_device::accounts))
                  .route("/account", post(controller::miot_device::add_account))
                  .route("/account", delete(controller::miot_device::delete_account))
                  //修改密码
                  .route("/account/password", put(controller::miot_device::change_password))
                  .route("/account/login", post(controller::miot_device::login))
                  .route("/account", put(controller::miot_device::update_account))
                  .route("/account/sync_mi_devices", get(controller::miot_device::sync_mi_devices))
              // .route("/update_iot_device", get(controller::miot-proto::update_iot_device))
              // .route("/login_mi_account", get(controller::miot-proto::login_mi_account))
              // .route("/delete_device", get(controller::miot-proto::delete_device))
              ,
        )
        .nest("/hap_metadata",
              Router::new()
                  .route("/characteristic_meta/:hap_type", get(controller::hap_metadata::get_characteristic_meta))
                  .route("/service_meta/:hap_type", get(controller::hap_metadata::get_service_meta))
              ,
        )
        .nest("/template",
              Router::new()
                  .route("/:id", get(controller::template::get))
                  .route("/check_update", post(controller::template::check_template_update))
                  .route("/check_add", post(controller::template::check_template_add))
                  .route("/text/:id", get(controller::template::get_text))
                  .route("/apply_mijia", post(controller::template::apply_mijia))
              ,
        )
        .nest("/native_ble",
              Router::new()
                  .route("/list", get(controller::native_ble_device::list))
                  .route("/status", get(controller::native_ble_device::status))
              ,
        )
}