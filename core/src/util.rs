use rand::prelude::*;

/// Generates a random port within the private range untouched by IANA.
pub fn random_port() -> u16 {
    thread_rng().gen_range(49152..u16::MAX)
}
