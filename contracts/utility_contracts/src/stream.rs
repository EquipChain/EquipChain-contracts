use std::time::{SystemTime, UNIX_EPOCH};

pub type Address = String;

#[derive(Clone, Debug)]
pub struct Stream {
    pub id: u64,
    pub provider: Address,
    pub payer: Address,

    pub tokens_per_second: u128,
    pub balance: u128,

    pub start_time: u64,
    pub end_time: u64,

    pub last_withdrawal_time: u64,

    pub pending_rate_change: Option<RateChangeProposal>,
}

#[derive(Clone, Debug)]
pub struct RateChangeProposal {
    pub new_rate: u128,
    pub proposed_at: u64,
    pub expires_at: u64,
}    /// Withdraw accumulated balance from a continuous stream
    pub fn withdraw_continuous(&self, stream_id: u64, amount: i128) -> Result<(), StreamError> {
        let flow = self.get_flow(stream_id)?;
        flow.provider.require_auth();
        self.withdraw_from_flow(flow, amount)
    }
