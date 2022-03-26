use cosmwasm_std::StdError;
use std::convert::TryFrom;

mod commands;
pub mod contract;
pub mod msg;
mod reply_response;
pub mod state;

// #[cfg(test)]
// mod tests;

pub const MIN_PSI_REWARDS_TO_CLAIM: u64 = 1_000_000_000u64;

pub enum SubmsgIds {
    InitANAsset,
    PsiClaimed,
    PsiSold,
}

impl TryFrom<u64> for SubmsgIds {
    type Error = StdError;

    fn try_from(v: u64) -> Result<Self, Self::Error> {
        match v {
            x if x == SubmsgIds::InitANAsset.id() => Ok(SubmsgIds::InitANAsset),
            x if x == SubmsgIds::PsiClaimed.id() => Ok(SubmsgIds::PsiClaimed),
            x if x == SubmsgIds::PsiSold.id() => Ok(SubmsgIds::PsiSold),
            unknown => Err(StdError::generic_err(format!(
                "unknown reply message id: {}",
                unknown
            ))),
        }
    }
}

impl SubmsgIds {
    pub const fn id(&self) -> u64 {
        match self {
            SubmsgIds::InitANAsset => 0,
            SubmsgIds::PsiClaimed => 1,
            SubmsgIds::PsiSold => 2,
        }
    }
}

#[inline]
fn concat(namespace: &[u8], key: &[u8]) -> Vec<u8> {
    let mut k = namespace.to_vec();
    k.extend_from_slice(key);
    k
}
