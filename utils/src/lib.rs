use cosmwasm_std::{Decimal, Uint128, Uint256};

mod swap;
mod query;

pub mod prelude {
    pub use crate::swap::{CW721Swap, SwapType};
    pub use crate::query::{PageResult, ListResponse, DetailsResponse};
    pub use crate::fee_percentage;
}

pub fn fee_percentage(amount: Uint128, share_percent: u64) -> Uint128 {
    // Allocate extra space for the two decimal places
    let amount = Uint256::from_uint128(amount) * Uint256::from_u128(100);

    // Get percentage and divide by 10 ** 4 (both decimal spots added up)
    let fee = (amount * Uint256::from(share_percent)) / Uint256::from(10000u16);

    // We can safely unwrap since weve tested against u128::MAX
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

        fee_percentage(Uint128::MAX, 1);
        fee_percentage(Uint128::MAX, 0);
        fee_percentage(Uint128::MAX, 100);
    }
}