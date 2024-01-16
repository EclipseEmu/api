//! DNS resolution via the [trust_dns_resolver](https://github.com/bluejekyll/trust-dns) crate

// NOTE: This file is pretty much entirely taken from reqwest, just modified to deny non-global IPs.

use {
    hyper::client::connect::dns::Name,
    once_cell::sync::OnceCell,
    reqwest::dns::{Addrs, Resolve, Resolving},
    std::{
        io,
        net::{IpAddr, SocketAddr},
        sync::Arc,
    },
    trust_dns_resolver::{lookup_ip::LookupIpIntoIter, system_conf, TokioAsyncResolver},
};

/// Wrapper around an `AsyncResolver`, which implements the `Resolve` trait.
#[derive(Debug, Default, Clone)]
pub(crate) struct TrustDnsResolver {
    /// Since we might not have been called in the context of a
    /// Tokio Runtime in initialization, so we must delay the actual
    /// construction of the resolver.
    state: Arc<OnceCell<TokioAsyncResolver>>,
}

struct SocketAddrs {
    iter: LookupIpIntoIter,
}

impl Resolve for TrustDnsResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let resolver = self.clone();
        Box::pin(async move {
            let resolver = resolver.state.get_or_try_init(new_resolver)?;

            let lookup = resolver.lookup_ip(name.as_str()).await?;
            let addrs: Addrs = Box::new(SocketAddrs {
                iter: lookup.into_iter(),
            });
            Ok(addrs)
        })
    }
}

impl Iterator for SocketAddrs {
    type Item = SocketAddr;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(ip_addr) => match self.iter.next() {
                Some(IpAddr::V4(ip)) if ip.is_global() => Some(SocketAddr::new(ip_addr, 0)),
                Some(IpAddr::V6(ip)) if ip.is_global() => Some(SocketAddr::new(ip_addr, 0)),
                _ => None,
            },
            None => None,
        }
    }
}

/// Create a new resolver with the default configuration,
/// which reads from `/etc/resolve.conf`.
fn new_resolver() -> io::Result<TokioAsyncResolver> {
    let (config, opts) = system_conf::read_system_conf().map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("error reading DNS system conf: {}", e),
        )
    })?;
    Ok(TokioAsyncResolver::tokio(config, opts))
}
