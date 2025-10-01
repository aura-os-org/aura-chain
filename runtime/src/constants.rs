pub mod currency {
    pub type Balance = u128;

    pub const UNIT: Balance = 1_000_000_000_000;
    pub const MILLIUNIT: Balance = 1_000_000_000;
    pub const MICROUNIT: Balance = 1_000_000;

    pub const fn deposit(items: u32, bytes: u32) -> Balance {
        items as Balance * 20 * UNIT + (bytes as Balance) * 100 * MILLIUNIT
    }
}

/// Time and blocks.
pub mod time {
    use super::currency::MILLIUNIT;

    pub const MILLISECS_PER_BLOCK: u64 = 6000;
    pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

    // These time units are defined in number of blocks.
    pub const MINUTES: u32 = 60_000 / (MILLISECS_PER_BLOCK as u32);
    pub const HOURS: u32 = MINUTES * 60;
    pub const DAYS: u32 = HOURS * 24;

    pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);
}

// Weight units
pub const WEIGHT_REF_TIME_PER_SECOND: u64 = 1_000_000_000_000;
pub const WEIGHT_PROOF_SIZE_PER_MB: u64 = 1024 * 1024;

pub const NORMAL_DISPATCH_RATIO: sp_runtime::Perbill = sp_runtime::Perbill::from_percent(75);
pub const AVERAGE_ON_INITIALIZE_RATIO: sp_runtime::Perbill = sp_runtime::Perbill::from_percent(10);

pub const MAXIMUM_BLOCK_WEIGHT: sp_weights::Weight = sp_weights::Weight::from_parts(
    WEIGHT_REF_TIME_PER_SECOND / 2,
    WEIGHT_PROOF_SIZE_PER_MB * 64,
);

pub const EXTRINSIC_BASE_WEIGHT: sp_weights::Weight = sp_weights::Weight::from_parts(125_000_000, 0);
