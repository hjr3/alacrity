use std::sync::{Arc, RwLock};

use server::Server;

/// A round-robin pool for servers
///
/// A simple pool that stores socket addresses and, for now, clones them out.
///
/// Inspired by https://github.com/NicolasLM/nucleon/blob/master/src/backend.rs
#[derive(Clone)]
pub struct Pool {
    inner: Arc<RwLock<inner::Pool>>,
}

impl Pool {
    pub fn with_servers(backends: Vec<Server>) -> Pool {
        let inner = inner::Pool::new(backends);

        Pool {
            inner: Arc::new(RwLock::new(inner)),
        }
    }

    pub fn new() -> Pool {
        Pool::with_servers(vec![])
    }

    /// Get a `Server` from the pool
    ///
    /// The pool may be exhausted of eligible addresses to connect to. The client is expected to
    /// handle this scenario.
    pub fn get(&self) -> Option<Server> {
        self.inner.write().expect("Lock is poisoned").get()
    }

    /// Returns all `Server` from the pool
    pub fn all(&self) -> Vec<Server> {
        self.inner.write().expect("Lock is poisoned").all()
    }

    /// Add a new server to the pool
    ///
    /// Currently, it is possible to add the same server more then once
    pub fn add(&self, server: Server) {
        self.inner.write().expect("Lock is poisoned").add(server)
    }

    /// Remove a server from the pool
    ///
    /// This will remove all instance of the given server. See `add` method for details on
    /// duplicate servers.
    pub fn remove(&self, backend: &Server) {
        self.inner.write().expect("Lock is poisoned").remove(backend)
    }
}

pub mod inner {
    use super::Server;

    pub struct Pool {
        backends: Vec<Server>,
        last_used: usize,
    }

    impl Pool {
        pub fn new(backends: Vec<Server>) -> Pool {
            Pool {
                backends: backends,
                last_used: 0,
            }
        }

        pub fn get(&mut self) -> Option<Server> {
            if self.backends.is_empty() {
                warn!("Pool is exhausted of socket addresses");
                return None;
            }
            self.last_used = (self.last_used + 1) % self.backends.len();
            self.backends.get(self.last_used).map(|server| {
                debug!("Pool is cloaning (hehe) out {:?}", server);
                server.clone()
            })
        }

        pub fn all(&mut self) -> Vec<Server> {
            if self.backends.is_empty() {
                warn!("Pool is exhausted of socket addresses");
                return Vec::new();
            }
            self.backends.clone()
        }

        pub fn add(&mut self, server: Server) {
            self.backends.push(server);
        }

        pub fn remove(&mut self, server: &Server) {
            self.backends.retain(|s| s != server);
        }
    }


    #[cfg(test)]
    mod tests {
        use super::Pool;
        use server::Server;
        use std::str::FromStr;
        use hyper::Url;

        #[test]
        fn test_from_str() {
            let backend1: Server = FromStr::from_str("http://127.0.0.1:6000").unwrap();
            assert_eq!(backend1.url(), Url::parse("http://127.0.0.1:6000").unwrap());

            let backend2: Server = FromStr::from_str("https://10.10.10.10:1010").unwrap();
            assert_eq!(backend2.url(), Url::parse("https://10.10.10.10:1010").unwrap());

            let backend3: Server = FromStr::from_str("8.8.8.8:6543").unwrap();
            assert_eq!(backend3.url(), Url::parse("http://8.8.8.8:6543").unwrap());
        }

        #[test]
        fn test_rrb_backend() {
            let backends: Vec<Server> = vec![
                FromStr::from_str("127.0.0.1:6000").unwrap(),
                FromStr::from_str("127.0.0.1:6001").unwrap(),
            ];

            let mut rrb = Pool::new(backends);
            assert_eq!(2, rrb.backends.len());

            let first = rrb.get().unwrap();
            let second = rrb.get().unwrap();
            let third = rrb.get().unwrap();
            let fourth = rrb.get().unwrap();
            assert_eq!(first, third);
            assert_eq!(second, fourth);
            assert!(first != second);
        }

        #[test]
        fn test_empty_rrb_backend() {
            let backends= vec![];
            let mut rrb = Pool::new(backends);
            assert_eq!(0, rrb.backends.len());
            assert!(rrb.get().is_none());
            assert!(rrb.all().is_empty());
        }

        #[test]
        fn test_add_to_rrb_backend() {
            let mut rrb = Pool::new(vec![]);
            assert!(rrb.get().is_none());
            let server: Server = FromStr::from_str("127.0.0.1:6000").unwrap();
            rrb.add(server.clone());
            assert!(rrb.get().is_some());
            assert_eq!(vec![server], rrb.all());
        }

        #[test]
        fn test_remove_from_rrb_backend() {
            let mut rrb = Pool::new(vec![]);
            let server1: Server = FromStr::from_str("127.0.0.1:6000").unwrap();
            let server2: Server = FromStr::from_str("127.0.0.1:6001").unwrap();
            rrb.add(server1.clone());
            rrb.add(server2.clone());
            assert_eq!(2, rrb.backends.len());
            assert_eq!(vec![server1.clone(), server2.clone()], rrb.all());

            let unknown_server: Server = FromStr::from_str("127.0.0.1:1234").unwrap();
            rrb.remove(&unknown_server);
            assert_eq!(2, rrb.backends.len());
            assert_eq!(vec![server1.clone(), server2.clone()], rrb.all());

            rrb.remove(&server1);
            assert_eq!(1, rrb.backends.len());
            assert_eq!(vec![server2.clone()], rrb.all());

            rrb.remove(&server2);
            assert_eq!(0, rrb.backends.len());
            assert!(rrb.all().is_empty());
        }
    }
}
