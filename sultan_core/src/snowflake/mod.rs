//! Snowflake ID Generator
//!
//! Generates unique 64-bit IDs with the following structure:
//! - Unused: 1 bit (always 0, ensures positive i64 for database compatibility)
//! - Timestamp: 40 bits (milliseconds since epoch, ~34.8 years)
//! - Node ID: 8 bits (0-255 nodes)
//! - Step: 15 bits (0-32767 IDs per millisecond per node)
//!
//! Total: 1 + 40 + 8 + 15 = 64 bits
//!
//! By keeping the most significant bit as 0, the generated IDs are always
//! positive when stored as i64 in databases like SQLite/PostgreSQL.

use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

const UNUSED_BITS: u8 = 1;
const TIMESTAMP_BITS: u8 = 40;
const NODE_BITS: u8 = 8;
const STEP_BITS: u8 = 15;

// Verify at compile time that bits add up to 64
const _: () = assert!(UNUSED_BITS + TIMESTAMP_BITS + NODE_BITS + STEP_BITS == 64);

const MAX_NODE: u64 = (1 << NODE_BITS) - 1; // 255
const MAX_STEP: u64 = (1 << STEP_BITS) - 1; // 32767

const NODE_SHIFT: u8 = STEP_BITS; // 15
const TIMESTAMP_SHIFT: u8 = NODE_BITS + STEP_BITS; // 23

// Custom epoch: 2025-01-01 00:00:00 UTC (in milliseconds)
const EPOCH: u64 = 1735689600000;

/// Trait for ID generation - allows mocking in tests
pub trait IdGenerator: Send + Sync {
    fn generate(&self) -> Result<i64, SnowflakeError>;
}

#[derive(Debug)]
pub enum SnowflakeError {
    InvalidNode(u64),
}

impl std::fmt::Display for SnowflakeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SnowflakeError::InvalidNode(node) => {
                write!(f, "Invalid node ID: {}. Must be 0-{}", node, MAX_NODE)
            }
        }
    }
}

impl std::error::Error for SnowflakeError {}

pub struct SnowflakeGenerator {
    node: u64,
    state: Mutex<SnowflakeState>,
}

struct SnowflakeState {
    last_timestamp: u64,
    step: u64,
}

impl SnowflakeGenerator {
    /// Creates a new Snowflake generator with the given node ID.
    ///
    /// # Arguments
    /// * `node` - Node ID (0-255)
    ///
    /// # Returns
    /// * `Ok(SnowflakeGenerator)` - A new generator
    /// * `Err(SnowflakeError::InvalidNode)` - If node ID is > 255
    pub fn new(node: u64) -> Result<Self, SnowflakeError> {
        if node > MAX_NODE {
            return Err(SnowflakeError::InvalidNode(node));
        }

        Ok(Self {
            node,
            state: Mutex::new(SnowflakeState {
                last_timestamp: 0,
                step: 0,
            }),
        })
    }

    /// Generates a new unique snowflake ID.
    ///
    /// The most significant bit is always 0, ensuring the value is
    /// always positive as i64 for database storage.
    ///
    /// # Returns
    /// * `Ok(i64)` - A unique positive 64-bit ID
    /// * `Err(SnowflakeError)` - If an error occurs
    pub fn generate(&self) -> Result<i64, SnowflakeError> {
        loop {
            let mut state = self.state.lock().unwrap_or_else(|poisoned| {
                // Recover from poisoned mutex by taking the inner value
                poisoned.into_inner()
            });

            let mut timestamp = Self::current_timestamp();

            if timestamp < state.last_timestamp {
                // Clock moved backwards, use last known timestamp
                timestamp = state.last_timestamp;
            }

            if timestamp == state.last_timestamp {
                state.step = (state.step + 1) & MAX_STEP;
                if state.step == 0 {
                    // Step overflow - release lock before waiting
                    let last_ts = state.last_timestamp;
                    drop(state);
                    Self::wait_next_millis(last_ts);
                    // Retry with fresh lock
                    continue;
                }
            } else {
                state.step = 0;
            }
            state.last_timestamp = timestamp;

            // Build ID: unused bit (0) is implicit since timestamp < 2^40
            let id = (timestamp << TIMESTAMP_SHIFT) | (self.node << NODE_SHIFT) | state.step;

            return Ok(id as i64);
        }
    }

    /// Extracts the timestamp from a snowflake ID.
    /// Returns the original Unix timestamp in milliseconds.
    pub fn extract_timestamp(id: i64) -> u64 {
        ((id as u64) >> TIMESTAMP_SHIFT) + EPOCH
    }

    /// Extracts the node ID from a snowflake ID.
    pub fn extract_node(id: i64) -> u64 {
        ((id as u64) >> NODE_SHIFT) & MAX_NODE
    }

    /// Extracts the step from a snowflake ID.
    pub fn extract_step(id: i64) -> u64 {
        (id as u64) & MAX_STEP
    }

    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as u64
            - EPOCH
    }

    fn wait_next_millis(last_timestamp: u64) -> u64 {
        let mut timestamp = Self::current_timestamp();
        while timestamp <= last_timestamp {
            std::hint::spin_loop();
            timestamp = Self::current_timestamp();
        }
        timestamp
    }
}

impl IdGenerator for SnowflakeGenerator {
    fn generate(&self) -> Result<i64, SnowflakeError> {
        SnowflakeGenerator::generate(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    const MAX_TIMESTAMP: u64 = (1 << TIMESTAMP_BITS) - 1; // ~34.8 years in ms

    #[test]
    fn test_new_valid_node() {
        let generator = SnowflakeGenerator::new(0);
        assert!(generator.is_ok());

        let generator = SnowflakeGenerator::new(255);
        assert!(generator.is_ok());
    }

    #[test]
    fn test_new_invalid_node() {
        let result = SnowflakeGenerator::new(256);
        assert!(matches!(result, Err(SnowflakeError::InvalidNode(256))));
    }

    #[test]
    fn test_generate_unique_ids() {
        let generator = SnowflakeGenerator::new(1).unwrap();
        let mut ids = HashSet::new();

        for _ in 0..10000 {
            let id = generator.generate().unwrap();
            assert!(ids.insert(id), "Duplicate ID generated: {}", id);
        }
    }

    #[test]
    fn test_generate_increasing_ids() {
        let generator = SnowflakeGenerator::new(1).unwrap();
        let mut last_id = 0i64;

        for _ in 0..1000 {
            let id = generator.generate().unwrap();
            assert!(id > last_id, "ID should be increasing");
            last_id = id;
        }
    }

    #[test]
    fn test_extract_node() {
        let generator = SnowflakeGenerator::new(42).unwrap();
        let id = generator.generate().unwrap();
        assert_eq!(SnowflakeGenerator::extract_node(id), 42);
    }

    #[test]
    fn test_extract_components() {
        let generator = SnowflakeGenerator::new(100).unwrap();
        let id = generator.generate().unwrap();

        let node = SnowflakeGenerator::extract_node(id);
        let step = SnowflakeGenerator::extract_step(id);
        let timestamp = SnowflakeGenerator::extract_timestamp(id);

        assert_eq!(node, 100);
        assert_eq!(step, 0); // First ID should have step 0
        assert!(timestamp > EPOCH); // Timestamp should be after our epoch
    }

    #[test]
    fn test_multiple_nodes_unique() {
        let generator1 = SnowflakeGenerator::new(1).unwrap();
        let generator2 = SnowflakeGenerator::new(2).unwrap();

        let id1 = generator1.generate().unwrap();
        let id2 = generator2.generate().unwrap();

        assert_ne!(id1, id2, "IDs from different nodes should be different");
    }

    #[test]
    fn test_bit_structure() {
        // Verify the bit structure is correct: 1 + 40 + 8 + 15 = 64
        assert_eq!(UNUSED_BITS + TIMESTAMP_BITS + NODE_BITS + STEP_BITS, 64);
        assert_eq!(TIMESTAMP_SHIFT, 23); // 8 + 15
        assert_eq!(NODE_SHIFT, 15);
        assert_eq!(MAX_NODE, 255); // 2^8 - 1
        assert_eq!(MAX_STEP, 32767); // 2^15 - 1
        assert_eq!(MAX_TIMESTAMP, (1u64 << 40) - 1); // 2^40 - 1
    }

    #[test]
    fn test_id_is_always_positive_i64() {
        let generator = SnowflakeGenerator::new(255).unwrap();

        // Generate many IDs and verify they're all positive
        for _ in 0..10000 {
            let id = generator.generate().unwrap();
            assert!(id > 0, "ID {} should be positive", id);
        }
    }

    #[test]
    fn test_max_id_is_positive_i64() {
        // Construct the maximum possible ID
        let max_timestamp = MAX_TIMESTAMP;
        let max_node = MAX_NODE;
        let max_step = MAX_STEP;

        let max_id = (max_timestamp << TIMESTAMP_SHIFT) | (max_node << NODE_SHIFT) | max_step;

        // Verify it's still positive as i64
        let max_id_as_i64 = max_id as i64;
        assert!(
            max_id_as_i64 > 0,
            "Max ID {} should be positive as i64, got {}",
            max_id,
            max_id_as_i64
        );

        // Verify it's less than i64::MAX
        assert!(max_id <= i64::MAX as u64, "Max ID should be <= i64::MAX");
    }

    #[test]
    fn test_epoch_is_2025() {
        // 2025-01-01 00:00:00 UTC in milliseconds
        assert_eq!(EPOCH, 1735689600000);
    }
}
