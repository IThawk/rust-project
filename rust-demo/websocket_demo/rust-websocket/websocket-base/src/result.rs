//! The result type used within Rust-WebSocket

use std;
use std::convert::From;
use std::error::Error;
use std::fmt;
use std::io;
use std::str::Utf8Error;

/// The type used for WebSocket results
pub type WebSocketResult<T> = Result<T, WebSocketError>;

/// This module contains convenience types to make working with Futures and
/// websocket results easier.
#[cfg(feature = "async")]
pub mod r#async {
	use super::WebSocketError;
	use futures::Future;

	/// The most common Future in this library, it is simply some result `I` or
	/// a `WebSocketError`. This is analogous to the `WebSocketResult` type.
	pub type WebSocketFuture<I> = Box<dyn Future<Item = I, Error = WebSocketError> + Send>;
}

/// Represents a WebSocket error
#[derive(Debug)]
pub enum WebSocketError {
	/// A WebSocket protocol error
	ProtocolError(&'static str),
	/// Invalid WebSocket data frame error
	DataFrameError(&'static str),
	/// No data available
	NoDataAvailable,
	/// An input/output error
	IoError(io::Error),
	/// A UTF-8 error
	Utf8Error(Utf8Error),
	/// Other error from higher-level crate, for downcasting
	Other(Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl fmt::Display for WebSocketError {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.write_str("WebSocketError: ")?;
		fmt.write_str(self.description())?;
		Ok(())
	}
}

impl Error for WebSocketError {
	fn description(&self) -> &str {
		match *self {
			WebSocketError::ProtocolError(_) => "WebSocket protocol error",
			WebSocketError::DataFrameError(_) => "WebSocket data frame error",
			WebSocketError::NoDataAvailable => "No data available",
			WebSocketError::IoError(_) => "I/O failure",
			WebSocketError::Utf8Error(_) => "UTF-8 failure",
			WebSocketError::Other(ref e) => e.description(),
		}
	}

	#[allow(deprecated)]
	fn cause(&self) -> Option<&dyn Error> {
		match *self {
			WebSocketError::IoError(ref error) => Some(error),
			WebSocketError::Utf8Error(ref error) => Some(error),
			WebSocketError::Other(ref error) => error.cause(),
			_ => None,
		}
	}
}

impl From<io::Error> for WebSocketError {
	fn from(err: io::Error) -> WebSocketError {
		if err.kind() == io::ErrorKind::UnexpectedEof {
			return WebSocketError::NoDataAvailable;
		}
		WebSocketError::IoError(err)
	}
}

impl From<Utf8Error> for WebSocketError {
	fn from(err: Utf8Error) -> WebSocketError {
		WebSocketError::Utf8Error(err)
	}
}
