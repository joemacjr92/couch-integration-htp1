//! CouchOS integration for the Monoprice HTP-1.
 //!
//! The initial project skeleton separates the HTP-1 wire protocol from the
 //! Couch SDK adapter. The protocol implementation is intentionally staged:
 //! do not advertise controls until their command/confirmation behavior has
 //! been implemented and tested against an HTP-1.

pub mod client;
pub mod error;
pub mod protocol;
pub mod settings;
pub mod state;
pub mod transport;

pub use error::{Error, Result};
pub use settings::Settings;
pub use state::Htp1State;
