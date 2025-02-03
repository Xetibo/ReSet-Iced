use zbus::proxy;

#[proxy(
    default_service = "org.Xetibo.ReSet.Daemon",
    default_path = "/org/Xetibo/ReSet/Daemon",
    interface = "org.Xetibo.ReSet.Daemon"
)]
pub trait ReSetDbus {
    fn register_client(&self, name: &str) -> zbus::Result<bool>;
}
