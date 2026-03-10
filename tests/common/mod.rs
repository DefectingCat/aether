pub mod mock_client;
pub mod mock_room;
pub mod test_helpers;
pub mod test_utils;

// Re-export mock types and traits for easy access in tests
pub use mock_client::{MatrixClient, MockClient};
pub use mock_room::{MessageSender, MockRoom};
pub use test_helpers::*;
pub use test_utils::*;
