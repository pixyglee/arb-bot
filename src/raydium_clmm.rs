use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

#[derive(Debug, Serialize, Deserialize)]
pub struct PoolState {
    /// Bump to identify PDA
    pub bump: [u8; 1],
    // Which config the pool belongs
    pub amm_config: Pubkey,
    // Pool creator
    pub owner: Pubkey,
    /// Token pair of the pool, where token_mint_0 address < token_mint_1 address
    pub token_mint0: Pubkey,
    pub token_mint1: Pubkey,

    /// Token pair vault
    pub token_vault0: Pubkey,
    pub token_vault1: Pubkey,
    /// observation account key
    pub observation_key: Pubkey,
    /// mint0 and mint1 decimals
    pub mint_decimals0: u8,
    pub mint_decimals1: u8,
    /// The minimum number of ticks between initialized ticks
    pub tick_spacing: u16,
    /// The currently in range liquidity available to the pool.
    pub liquidity: u128,
    /// The current price of the pool as a sqrt(token_1/token_0) Q64.64 value
    pub sqrt_price_x64: u128,
    /// The current tick of the pool, i.e. according to the last tick transition that was run.
    pub tick_current: i32,
    // not sure
    pub observation_index: u16,
    pub observation_update_duration: u16,
    /// The fee growth as a Q64.64 number, i.e. fees of token_0 and token_1 collected per
    /// unit of liquidity for the entire life of the pool.
    pub fee_growth_global0_x64: u128,
    pub fee_growth_global1_x64: u128,
    /// The amounts of token_0 and token_1 that are owed to the protocol.
    pub protocol_fees_token0: u64,
    pub protocol_fees_token1: u64,
    /// The amounts in and out of swap token_0 and token_1
    pub swap_in_amount_token0: u128,
    pub swap_out_amount_token1: u128,
    pub swap_in_amount_token1: u128,
    pub swap_out_amount_token0: u128,

    /// Bitwise representation of the state of the pool
    /// bit0, 1: disable open position and increase liquidity, 0: normal
    /// bit1, 1: disable decrease liquidity, 0: normal
    /// bit2, 1: disable collect fee, 0: normal
    /// bit3, 1: disable collect reward, 0: normal
    /// bit4, 1: disable swap, 0: normal
    pub status: u8,
    /// Leave blank for future use
    pub padding: [u8; 7],
    pub reward_infos: [RewardInfo; 3],
    /// Packed initialized tick array state
    pub tick_array_bitmap: [u64; 16],
    /// except protocol_fee and fund_fee
    pub total_fees_token0: u64,
    pub total_fees_claimed_token0: u64,
    /// except protocol_fee and fund_fee
    pub total_fees_token1: u64,
    pub total_fees_claimed_token1: u64,
    pub fund_fees_token0: u64,
    pub fund_fees_token1: u64,
    // The timestamp allowed for swap in the pool.
    // Note: The open_time is disabled for now.
    pub open_time: u64,
    // padding for later
    pub padding1: [u64; 25],
    pub padding2: [u64; 32],
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct RewardInfo {
    pub reward_state: u8,
    pub open_time: u64,
    pub end_time: u64,
    pub last_update_time: u64,
    pub emissions_per_second_x64: u128,
    pub reward_total_emissioned: u64,
    pub reward_claimed: u64,
    pub token_mint: Pubkey,
    pub token_vault: Pubkey,
    pub authority: Pubkey,
    pub reward_growth_global_x64: u128,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct TickState {
    pub tick: i32,
    pub liquidity_net: i128,
    pub liquidity_gross: u128,
    pub fee_growth_outside0_x64: u128,
    pub fee_growth_outside1_x64: u128,
    pub reward_growths_outside_x64: [u128; 3],
    pub padding: [u32; 13],
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PoolStatusBitIndex {
    OpenPositionOrIncreaseLiquidity,
    DecreaseLiquidity,
    CollectFee,
    CollectReward,
    Swap,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PoolStatusBitFlag {
    Enable,
    Disable,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RewardState {
    Uninitialized,
    Initialized,
    Opening,
    Ended,
}
