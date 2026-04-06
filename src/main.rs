use anyhow::Context;
use clap::Parser;

mod avahi;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    #[arg(long, default_value_t = 120)]
    ttl: u32,
    #[arg(long)]
    fqdn: Option<String>,
    #[arg(short, long)]
    subdomain: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if args.subdomain.is_empty() {
        print!("no subdomains specified");
        return Ok(());
    }

    let conn = zbus::blocking::Connection::system().context("error connecting to system buss")?;
    let server =
        avahi::ServerProxyBlocking::new(&conn).context("error connecting to avahi-daemon")?;
    let fqdn = if let Some(fqdn) = args.fqdn {
        fqdn
    } else {
        server
            .get_host_name_fqdn()
            .context("error getting hostname fqdn from avahi-daemon")?
    };

    let rdata = {
        let mut rdata = Vec::new();
        for label in fqdn.split(".") {
            rdata.push(label.len() as u8);
            rdata.extend(label.as_bytes());
        }
        rdata.push(0);
        rdata
    };

    let entry_group = server.entry_group_new()?;
    for name in args.subdomain {
        entry_group
            .add_record(-1, -1, 0, &format!("{name}.{fqdn}"), 1, 5, args.ttl, &rdata)
            .context("error adding cname record to entry group")?;
    }
    entry_group
        .commit()
        .context("error commiting entry group")?;

    let dbus_proxy = zbus::blocking::fdo::DBusProxy::new(&conn)?;
    for signal in dbus_proxy.receive_name_owner_changed()? {
        let args = signal.args()?;
        if args.name() == "org.freedesktop.Avahi"
            && args.new_owner().as_deref().unwrap_or("").is_empty()
        {
            println!("avahi-daemon exited; stopping");
            break;
        }
    }
    Ok(())
}
