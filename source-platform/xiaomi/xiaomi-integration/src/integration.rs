use hl_integration::integration::HlSourceIntegrator;
use target_hap::delegate::database::get_hap_model_ext_database;
use target_hap::delegate::model;
use target_hap::delegate::model::AccessoryModelExtConstructor;
use crate::models;

pub struct XiaomiIntegration {}


impl HlSourceIntegrator for XiaomiIntegration {
    fn name(&self) -> &str {
        "米家"
    }

    fn init(&self) -> anyhow::Result<()> {
        //注册model
        let database = get_hap_model_ext_database();

        database.insert("common.mode_switch".to_string(), models::common::mode_switch::ModelExt::new)?;
        database.insert("common.miot_spec_prop_mapping".to_string(), models::common::miot_spec_prop_mapping::ModelExt::new)?;
        database.insert("common.ble_value_mapping".to_string(), models::common::ble_value_mapping::ModelExt::new)?;
        // model_map.insert("common.native_ble".to_string(), common::native_ble::ModelExt::new);
        database.insert("lumi.acpartner.mcn02".to_string(), models::lumi::lumi_acpartner_mcn02::ModelExt::new)?;
        database.insert("lumi.gateway.mgl03".to_string(), models::lumi::lumi_gateway_mgl03::ModelExt::new)?;
        database.insert("deerma.humidifier.jsq3".to_string(),models::deerma::deerma_humidifier_jsq3::ModelExt::new)?;

        Ok(())
    }
}