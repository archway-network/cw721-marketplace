use cosmwasm_std::{Uint128, Uint256};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

mod swap;
mod query;

pub mod prelude {
    pub use crate::swap::{CW721Swap, SwapType};
    pub use crate::query::{PageResult, ListResponse, DetailsResponse};
    pub use crate::fee_percentage;
}

// Fee split result
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FeeSplit {
    pub marketplace: Uint128,
    pub seller: Uint128,
}

impl FeeSplit {
    pub fn only_seller(amount: Uint128) -> Self {
        Self {
            marketplace: Uint128::zero(),
            seller: amount
        }
    }
}

pub fn fee_percentage(amount: Uint128, share_percent: u64) -> Uint128 {
    // Allocate extra space for the two decimal places
    let amount = Uint256::from_uint128(amount) * Uint256::from_u128(100);

    // Get percentage and divide by 10 ** 4 (both decimal spots added up)
    let fee = (amount * Uint256::from(share_percent))
        .checked_div(Uint256::from(10000u16)).unwrap_or(Uint256::zero());

    // We can safely unwrap since we've tested against u128::MAX
    fee.try_into().unwrap()
}

#[cfg(test)]
mod test {
    use cosmwasm_std::Uint128;
    use crate::fee_percentage;

    #[test]
    fn fee_percentage_overflow() {
        // Test small number
        let amount = Uint128::new(123456);
        let expected = Uint128::new(43209);
        assert_eq!(fee_percentage(amount, 35), expected);

        // Test largest number
        let amount = Uint128::new(100000000000000000000000000000000000000);
        let expected = Uint128::new(50000000000000000000000000000000000000);
        assert_eq!(fee_percentage(amount, 50), expected);

        // Testing for overflow
        fee_percentage(Uint128::MAX, 1);
        fee_percentage(Uint128::MAX, 0);
        fee_percentage(Uint128::MAX, 100);

        // Testing for underflow
        assert_eq!(fee_percentage(Uint128::one(), 0), Uint128::zero());
        assert_eq!(fee_percentage(Uint128::one(), 1), Uint128::zero());
        assert_eq!(fee_percentage(Uint128::one(), 10), Uint128::zero());
        assert_eq!(fee_percentage(Uint128::one(), 100), Uint128::one());

        assert_eq!(fee_percentage(Uint128::zero(), 0), Uint128::zero());
        assert_eq!(fee_percentage(Uint128::zero(), 1), Uint128::zero());
        assert_eq!(fee_percentage(Uint128::zero(), 10), Uint128::zero());
        assert_eq!(fee_percentage(Uint128::zero(), 100), Uint128::zero());
    }
}