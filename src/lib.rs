use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    pubkey::Pubkey,
    program_error::ProgramError,
    sysvar::rent::Rent,
    program_pack::Pack,
};
use drift::controller::position::PositionDirection;
use drift::cpi::accounts::PlaceAndMake;
use drift::error::DriftResult;
use drift::instructions::optional_accounts::{load_maps, AccountMaps};
use drift::math::casting::Cast;
use drift::math::safe_math::SafeMath;
use drift::program::Drift;
use drift::state::order_params::OrderParams;
use drift::state::state::State;
use drift::state::user::{MarketType as DriftMarketType, OrderTriggerCondition, OrderType};
use drift::state::user::{User, UserStats};

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct BulkOrderParams {
    pub market_index: u64,
    pub order_type: u8, // enum representation
    pub direction: u8,  // enum representation
    pub base_asset_amount: u64,
    pub price: u64,
}

entrypoint!(process_instruction);

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
   let accounts_iter = &mut accounts.iter();

    let state_account = next_account_info(accounts_iter)?;
    let user_account = next_account_info(accounts_iter)?;
    let authority_account = next_account_info(accounts_iter)?;

    // Deserialize instruction data (order parameters)
    let bulk_order_params: BulkOrderParams = BulkOrderParams::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    // Log the unpacked order parameters
    msg!("Market Index: {}", bulk_order_params.market_index);
    msg!("Order Type: {}", bulk_order_params.order_type);
    msg!("Direction: {}", bulk_order_params.direction);
    msg!("Base Asset Amount: {}", bulk_order_params.base_asset_amount);
    msg!("Price: {}", bulk_order_params.price);

   let order_params = OrderParams {
            order_type: bulk_order_params.order_type,
            market_type: DriftMarketType::Perp,
            direction: bulk_order_params.direction,
            user_order_id: 0,
            base_asset_amount: bulk_order_params.base_asset_amount,
            price: bulk_order_params.price,
            market_index,
            reduce_only: false,
            post_only: false,
            immediate_or_cancel: true,
            max_ts: None,
            trigger_price: None,
            trigger_condition: None,
            oracle_price_offset: None,
            auction_duration: None,
            auction_start_price: None,
            auction_end_price: None,
        };

        place_order(&ctx, order_params)?;

    Ok(())
}

// // Function to unpack order parameters from instruction data (you will need to customize this)
// fn unpack_order_params(data: &[u8]) -> Result<(OrderParams, OrderParams), ProgramError> {
//     // Unpacking logic for bid_order_params and ask_order_params from `instruction_data`.
//     // Customize this function based on how your instruction data is structured.
//     // Here, it's just an example, so you need to adapt this.

//     // Assuming the instruction data contains two sets of order parameters.
//     // Replace with actual deserialization logic.
//     let bid_order_params: OrderParams = // Deserialize first half of `data`
//     let ask_order_params: OrderParams = // Deserialize second half of `data`
    
//     Ok((bid_order_params, ask_order_params))
// }

fn place_order<'info>(
    ctx: &Context<'_, '_, '_, 'info, Jit<'info>>,
    order_params: OrderParams,
) -> Result<()> {
    let drift_program = ctx.accounts.drift_program.to_account_info().clone();
    let cpi_accounts = PlaceAndMake {
        state: ctx.accounts.state.to_account_info().clone(),
        user: ctx.accounts.user.to_account_info().clone(),
        user_stats: ctx.accounts.user_stats.to_account_info().clone(),
        authority: ctx.accounts.authority.to_account_info().clone(),
        taker: ctx.accounts.taker.to_account_info().clone(),
        taker_stats: ctx.accounts.taker_stats.to_account_info().clone(),
    };

    let cpi_context = CpiContext::new(drift_program, cpi_accounts)
        .with_remaining_accounts(ctx.remaining_accounts.into());

    if order_params.market_type == DriftMarketType::Perp {
        drift::cpi::place_and_make_perp_order(cpi_context, order_params, None)?;
    } else {
        drift::cpi::place_and_make_spot_order(cpi_context, order_params, None)?;
    }

    Ok(())
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
