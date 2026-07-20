pub mod error;
pub mod registry;
pub mod responder;

pub use error::{DnsError, Result};
pub use registry::{DnsEntry, DnsRegistry};
pub use responder::{MdnsResponder, ResponderConfig, ResponderStatus};
