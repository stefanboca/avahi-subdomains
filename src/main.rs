use std::{collections::HashSet, net::Ipv4Addr};

use anyhow::Context;
use simple_dns::Packet;
use socket2::{Domain, Protocol, Socket, Type};

mod avahi;

fn main() -> anyhow::Result<()> {
    let connection = zbus::blocking::Connection::system()?;
    let server = avahi::ServerProxyBlocking::new(&connection)?;
    let fqdn = server.get_host_name_fqdn()?;
    let entry_group = server.entry_group_new()?;
    dbg!(&fqdn);
    let rdata = {
        let mut rdata = Vec::new();
        for label in simple_dns::Name::new(&fqdn)?.iter() {
            rdata.push(label.len() as u8);
            rdata.extend(label.as_ref());
        }
        rdata.push(0);
        rdata
    };

    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    socket.set_reuse_address(true)?;
    socket.set_reuse_port(true)?;
    socket.bind(
        &"0.0.0.0:5353"
            .parse::<std::net::SocketAddr>()
            .unwrap()
            .into(),
    )?;
    socket.join_multicast_v4(&Ipv4Addr::new(224, 0, 0, 251), &Ipv4Addr::UNSPECIFIED)?;
    let socket: std::net::UdpSocket = socket.into();

    let mut registered = HashSet::<String>::new();
    let fqdn_with_dot = format!(".{fqdn}");
    let mut buf = [0u8; 9000];
    loop {
        let len = socket.recv(&mut buf)?;
        let Ok(packet) = Packet::parse(&buf[..len]) else {
            continue;
        };

        for question in packet.questions {
            let name = question.qname.to_string();
            if name.ends_with(&fqdn_with_dot) && !registered.contains(&name) {
                println!("register {name}");
                entry_group
                    .add_record(-1, -1, 0, &name, 1, 5, 1, &rdata)
                    .context("error adding record")?;
                entry_group.commit().context("error commiting")?;
                registered.insert(name);
            }
        }
    }
}
