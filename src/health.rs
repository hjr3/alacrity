use std::time::Duration;

use futures::*;
use tokio_timer::*;
use tokio_core::reactor::Core;
use hyper::Client;
use hyper_tls::HttpsConnector;
use std::io;

use pool::Pool;

pub struct HealthCheck {
    interval: Duration,
    pool: Pool,
    uri_path: String,
}

impl HealthCheck {
    pub fn new(interval: Duration, pool: Pool, uri_path: String) -> HealthCheck {

        HealthCheck {
            interval: interval,
            pool: pool,
            uri_path: uri_path,
        }
    }

    pub fn run(&self) -> io::Result<()> {
        let mut core = try!(Core::new());
        let timer = Timer::default();

        let handle = core.handle();
        let client = Client::configure()
            .connector(HttpsConnector::new(4, &handle))
            .build(&handle);

        let pool = self.pool.clone();
        let work = timer.interval(self.interval).for_each(move |()| {
            let servers = pool.all();
            for server in servers {
                let url = match server.url().join(&self.uri_path) {
                Ok(url) => url,
                Err(e) => {
                    error!("Invalid health check url: {:?}",e);
                    return Ok(());
                }
            };
                debug!("Health check {:?}", url);

                let pool1 = pool.clone();
                let pool2 = pool.clone();
                let server1 = server.clone();
                let server2 = server.clone();
                let req = client.get(url).and_then(move |res| {
                    debug!("Response: {}", res.status());
                    debug!("Headers: \n{}", res.headers());

                    if ! res.status().is_success() {
                        info!("Removing {:?} from pool", server1);
                        pool1.remove(&server1);
                    }

                    ::futures::finished(())
                }).map_err(move |e| {
                    error!("Error connecting to backend: {:?}", e);
                    info!("Removing {:?} from pool", server2);
                    pool2.remove(&server2);
                });

                handle.spawn(req);
            }

            Ok(())
        }).map_err(From::from);

        core.run(work)
    }
}
