pub struct IntegratorContext {}


/// 来源平台集成
pub trait HlSourceIntegrator {
    fn name(&self) -> &str;

    fn init(&self) -> anyhow::Result<()>;
}