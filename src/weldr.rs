extern crate env_logger;
extern crate weldr;

use std::env;
use std::net::SocketAddr;
use std::thread;
use std::time::Duration;

use weldr::server::Server;
use weldr::pool::Pool;
use weldr::mgmt;
use weldr::health;

fn main() {
    env_logger::init().expect("Failed to start logger");

    let addr = env::args().nth(1).unwrap_or("127.0.0.1:8080".to_string());
    let addr = addr.parse::<SocketAddr>().unwrap();

    let backend = env::args().nth(2).unwrap_or("127.0.0.1:12345".to_string());
    let backend = backend.parse::<Server>().unwrap();
    let map_host = env::args().nth(4).unwrap_or("false".to_string());
    let map_host: bool = map_host.parse().unwrap();
    let backend = backend.with_map_host(map_host);
    let pool = Pool::with_servers(vec![backend]);

    let admin_ip = env::args().nth(3).unwrap_or("127.0.0.1:8687".to_string());
    let admin_addr = admin_ip.parse::<SocketAddr>().unwrap();
    let p = pool.clone();
    let _ = thread::Builder::new().name("management".to_string()).spawn(move || {
        mgmt::listen(admin_addr, p);
    }).expect("Failed to create proxy thread");

    let p = pool.clone();
    let _ = thread::Builder::new().name("health-check".to_string()).spawn(move || {
        let checker = health::HealthCheck::new(Duration::from_millis(1000), p, "/".to_owned());
        checker.run();
    }).expect("Failed to create proxy thread");

    let (handle, _) = weldr::proxy::listen(addr, pool.clone()).expect("Failed to start server");
    handle.join().unwrap();
}
