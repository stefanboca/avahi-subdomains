use zbus::fdo;

#[zbus::proxy(
    interface = "org.freedesktop.Avahi.Server",
    default_service = "org.freedesktop.Avahi",
    default_path = "/"
)]
pub trait Server {
    async fn get_host_name_fqdn(&self) -> fdo::Result<String>;

    #[zbus(object = "EntryGroup")]
    async fn entry_group_new(&self);
}

#[zbus::proxy(
    interface = "org.freedesktop.Avahi.EntryGroup",
    default_service = "org.freedesktop.Avahi"
)]
pub trait EntryGroup {
    async fn commit(&self) -> fdo::Result<()>;

    async fn add_record(
        &self,
        interface: i32,
        protocol: i32,
        flags: u32,
        name: &str,
        clazz: u16,
        type_: u16,
        ttl: u32,
        rdata: &[u8],
    ) -> fdo::Result<()>;
}
