//! Driver and gateway error handling.

#[cfg(feature = "serenity")]
use futures::channel::mpsc::TrySendError;
#[cfg(feature = "serenity")]
use serenity::gateway::InterMessage;
#[cfg(feature = "gateway-core")]
use std::{error::Error, fmt};
#[cfg(feature = "twilight")]
use twilight_gateway::{cluster::ClusterCommandError, shard::CommandError};

#[cfg(feature = "gateway-core")]
#[derive(Debug)]
#[non_exhaustive]
/// Error returned when a manager or call handler is
/// unable to send messages over Discord's gateway.
pub enum JoinError {
    /// Request to join was dropped, cancelled, or replaced.
    Dropped,
    /// No available gateway connection was provided to send
    /// voice state update messages.
    NoSender,
    /// Tried to leave a [`Call`] which was not found.
    ///
    /// [`Call`]: crate::Call
    NoCall,
    /// Connection details were not received from Discord in the
    /// time given in [the `Call`'s configuration].
    ///
    /// This can occur if a message is lost by the Discord client
    /// between restarts, or if Discord's gateway believes that
    /// this bot is still in the channel it attempts to join.
    ///
    /// *Users should `leave` the server on the gateway before
    /// re-attempting connection.*
    ///
    /// [the `Call`'s configuration]: crate::Config
    TimedOut,
    /// The given guild ID was zero.
    IllegalGuild,
    /// The given channel ID was zero.
    IllegalChannel,
    #[cfg(feature = "driver-core")]
    /// The driver failed to establish a voice connection.
    ///
    /// *Users should `leave` the server on the gateway before
    /// re-attempting connection.*
    Driver(ConnectionError),
    #[cfg(feature = "serenity")]
    /// Serenity-specific WebSocket send error.
    Serenity(TrySendError<InterMessage>),
    #[cfg(feature = "twilight")]
    /// Twilight-specific WebSocket send error returned when using a shard cluster.
    TwilightCluster(ClusterCommandError),
    #[cfg(feature = "twilight")]
    /// Twilight-specific WebSocket send error when explicitly using a single shard.
    TwilightShard(CommandError),
}

#[cfg(feature = "gateway-core")]
impl JoinError {
    /// Indicates whether this failure may have left (or been
    /// caused by) Discord's gateway state being in an
    /// inconsistent state.
    ///
    /// Failure to `leave` before rejoining may cause further
    /// timeouts.
    pub fn should_leave_server(&self) -> bool {
        matches!(self, JoinError::TimedOut)
    }

    #[cfg(feature = "driver-core")]
    /// Indicates whether this failure can be reattempted via
    /// [`Driver::connect`] with retreived connection info.
    ///
    /// Failure to `leave` before rejoining may cause further
    /// timeouts.
    ///
    /// [`Driver::connect`]: crate::driver::Driver
    pub fn should_reconnect_driver(&self) -> bool {
        matches!(self, JoinError::Driver(_))
    }
}

#[cfg(feature = "gateway-core")]
impl fmt::Display for JoinError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "failed to join voice channel: ")?;
        match self {
            JoinError::Dropped => write!(f, "request was cancelled/dropped"),
            JoinError::NoSender => write!(f, "no gateway destination"),
            JoinError::NoCall => write!(f, "tried to leave a non-existent call"),
            JoinError::TimedOut => write!(f, "gateway response from Discord timed out"),
            JoinError::IllegalGuild => write!(f, "target guild ID was zero"),
            JoinError::IllegalChannel => write!(f, "target channel ID was zero"),
            #[cfg(feature = "driver-core")]
            JoinError::Driver(_) => write!(f, "establishing connection failed"),
            #[cfg(feature = "serenity")]
            JoinError::Serenity(e) => e.fmt(f),
            #[cfg(feature = "twilight")]
            JoinError::TwilightCluster(e) => e.fmt(f),
            #[cfg(feature = "twilight")]
            JoinError::TwilightShard(e) => e.fmt(f),
        }
    }
}

#[cfg(feature = "gateway-core")]
impl Error for JoinError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            JoinError::Dropped => None,
            JoinError::NoSender => None,
            JoinError::NoCall => None,
            JoinError::TimedOut => None,
            JoinError::IllegalGuild => None,
            JoinError::IllegalChannel => None,
            #[cfg(feature = "driver-core")]
            JoinError::Driver(e) => Some(e),
            #[cfg(feature = "serenity")]
            JoinError::Serenity(e) => e.source(),
            #[cfg(feature = "twilight")]
            JoinError::TwilightCluster(e) => e.source(),
            #[cfg(feature = "twilight")]
            JoinError::TwilightShard(e) => e.source(),
        }
    }
}

#[cfg(all(feature = "serenity", feature = "gateway-core"))]
impl From<TrySendError<InterMessage>> for JoinError {
    fn from(e: TrySendError<InterMessage>) -> Self {
        JoinError::Serenity(e)
    }
}

#[cfg(all(feature = "twilight", feature = "gateway-core"))]
impl From<CommandError> for JoinError {
    fn from(e: CommandError) -> Self {
        JoinError::TwilightShard(e)
    }
}

#[cfg(all(feature = "twilight", feature = "gateway-core"))]
impl From<ClusterCommandError> for JoinError {
    fn from(e: ClusterCommandError) -> Self {
        JoinError::TwilightCluster(e)
    }
}

#[cfg(all(feature = "driver-core", feature = "gateway-core"))]
impl From<ConnectionError> for JoinError {
    fn from(e: ConnectionError) -> Self {
        JoinError::Driver(e)
    }
}

#[cfg(feature = "gateway-core")]
/// Convenience type for Discord gateway error handling.
pub type JoinResult<T> = Result<T, JoinError>;

#[cfg(feature = "driver-core")]
pub use crate::{
    driver::connection::error::{Error as ConnectionError, Result as ConnectionResult},
    tracks::{TrackError, TrackResult},
};
