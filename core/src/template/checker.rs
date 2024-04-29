use crate::template::hl_template::HlDeviceTemplate;

pub fn check_template(temp: &HlDeviceTemplate) -> anyhow::Result<()> {
    //检测tag 是否重复
    for x in &temp.devices {

    }
    Ok(())
}