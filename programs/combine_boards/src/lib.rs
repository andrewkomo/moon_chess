use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_spl::token::{self, TokenAccount, Transfer};
use switchboard_v2::{VrfAccountData, VrfRequestRandomness};

declare_id!("C7sgpMEh78hmwz7Q5pUiaEQnFSZ1dW2NkYokkkxi7dTJ");

const COMBINE_COST: u64 = 500;
const TREASURY_ACCOUNT: Pubkey = Pubkey::new_from_array([188, 173, 120, 153, 102, 162, 207, 89, 228, 106, 67, 87, 77, 227, 109, 75, 122, 71, 253, 87, 194, 206, 181, 192, 136, 55, 196, 26, 130, 245, 182, 228]);

const SEED: &[u8] = b"COMBINATION";

#[program]
pub mod combine_boards {
    use super::*;
    pub fn combine_init(ctx: Context<CombineInit>, params: CombineInitParams) -> Result<()> {
        let combination = &mut ctx.accounts.combination.load_init()?;
        combination.board_1 = params.board_1_mint;
        combination.board_2 = params.board_2_mint;
        combination.combiner = ctx.accounts.combiner.key();

        token::transfer(ctx.accounts.pay_combine_ctx(), COMBINE_COST)?;
        combination.paid = true;

        let vrf_account_info = &ctx.accounts.vrf;
        let _vrf = VrfAccountData::new(vrf_account_info)?;
        combination.vrf = vrf_account_info.key();

        Ok(())
    }

    pub fn request_random(ctx: Context<RequestRandom>, params: RequestRandomParams) -> Result<()> {
        let combination = &ctx.accounts.combination.load()?;
        require!(combination.result_buffer == [0;32], CombineErrorCode::RandomAlreadyExists);
        require!(combination.paid, CombineErrorCode::PaymentNotComplete);

        RequestRandom::actuate(&ctx, &params)
    }

    pub fn update_random(ctx: Context<UpdateRandom>) -> Result<()> {
        let combination = &mut ctx.accounts.combination.load_mut()?;
        require!(combination.result_buffer == [0;32], CombineErrorCode::RandomAlreadyExists);
        require!(combination.paid, CombineErrorCode::PaymentNotComplete);

        let vrf_account_info = &ctx.accounts.vrf;
        let vrf = VrfAccountData::new(vrf_account_info)?;

        combination.result_buffer = vrf.get_result()?;

        Ok(())
    }
}

#[account(zero_copy)]
#[derive(Default)]
pub struct BoardCombination {
    board_1: Pubkey,
    board_2: Pubkey,
    combiner: Pubkey,
    vrf: Pubkey,
    result_buffer: [u8; 32],
    paid: bool
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct CombineInitParams {
    board_1_mint: Pubkey,
    board_2_mint: Pubkey,
}
#[derive(Accounts)]
#[instruction(params: CombineInitParams)]
pub struct CombineInit<'info> {
    #[account(
        init,
        seeds = [
            SEED, 
            vrf.key().as_ref(),
            combiner.key().as_ref(),
        ],
        payer = combiner,
        bump,
    )]
    pub combination: AccountLoader<'info, BoardCombination>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut, signer)]
    pub combiner: AccountInfo<'info>,
    #[account(mut, constraint = combiner_token_account.amount >= COMBINE_COST)]
    pub combiner_token_account: Account<'info, TokenAccount>,
    #[account(mut, constraint = treasury_account.owner == TREASURY_ACCOUNT)]
    // #[account(mut)]
    pub treasury_account: Account<'info, TokenAccount>,
    #[account(
        constraint = board_1_token.amount >= 1,
        constraint = board_1_token.owner == combiner.key(),
        constraint = board_1_token.mint == params.board_1_mint,
    )]
    pub board_1_token: Account<'info, TokenAccount>,
    #[account(
        constraint = board_2_token.amount >= 1,
        constraint = board_2_token.owner == combiner.key(),
        constraint = board_2_token.mint == params.board_2_mint,
    )]
    pub board_2_token: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(address = token::ID)]
    pub token_program: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub vrf: AccountInfo<'info>,
}
impl<'info> CombineInit<'info> {
    pub fn pay_combine_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.combiner_token_account.to_account_info().clone(),
            to: self.treasury_account.to_account_info().clone(),
            authority: self.combiner.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }
}

#[derive(Accounts)]
#[instruction(params: RequestRandomParams)] // rpc parameters hint
pub struct RequestRandom<'info> {
    #[account(
        mut,
        seeds = [
            SEED, 
            vrf.key().as_ref(),
            authority.key().as_ref(),
        ],
        bump = params.client_state_bump,
        constraint = combination.load()?.vrf ==  vrf.key()
    )]
    pub combination: AccountLoader<'info, BoardCombination>,
    #[account(signer)] // client authority needs to sign
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub authority: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub switchboard_program: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub vrf: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub oracle_queue: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub queue_authority: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub data_buffer: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub permission: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut, constraint = escrow.owner == program_state.key())]
    pub escrow: Account<'info, TokenAccount>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut, constraint = payer_wallet.owner == payer_authority.key())]
    pub payer_wallet: Account<'info, TokenAccount>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(signer)]
    pub payer_authority: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(address = solana_program::sysvar::recent_blockhashes::ID)]
    pub recent_blockhashes: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub program_state: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(address = anchor_spl::token::ID)]
    pub token_program: AccountInfo<'info>,
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct RequestRandomParams {
    pub client_state_bump: u8,
    pub permission_bump: u8,
    pub switchboard_state_bump: u8,
}

impl RequestRandom<'_> {
    pub fn actuate(ctx: &Context<Self>, params: &RequestRandomParams) -> Result<()> {
        let switchboard_program = ctx.accounts.switchboard_program.to_account_info();

        let vrf_request_randomness = VrfRequestRandomness {
            authority: ctx.accounts.combination.to_account_info(),
            vrf: ctx.accounts.vrf.to_account_info(),
            oracle_queue: ctx.accounts.oracle_queue.to_account_info(),
            queue_authority: ctx.accounts.queue_authority.to_account_info(),
            data_buffer: ctx.accounts.data_buffer.to_account_info(),
            permission: ctx.accounts.permission.to_account_info(),
            escrow: ctx.accounts.escrow.clone(),
            payer_wallet: ctx.accounts.payer_wallet.clone(),
            payer_authority: ctx.accounts.payer_authority.to_account_info(),
            recent_blockhashes: ctx.accounts.recent_blockhashes.to_account_info(),
            program_state: ctx.accounts.program_state.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
        };

        let vrf_key = ctx.accounts.vrf.key.clone();
        let authority_key = ctx.accounts.authority.key.clone();
        let state_seeds: &[&[&[u8]]] = &[&[
            &SEED,
            vrf_key.as_ref(),
            authority_key.as_ref(),
            &[params.client_state_bump],
        ]];
        msg!("requesting randomness");
        vrf_request_randomness.invoke_signed(
            switchboard_program,
            params.switchboard_state_bump,
            params.permission_bump,
            state_seeds,
        )?;

        msg!("randomness requested successfully");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct UpdateRandom<'info> {
    #[account(mut)]
    pub combination: AccountLoader<'info, BoardCombination>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub vrf: AccountInfo<'info>,
}
impl UpdateRandom<'_> {
    pub fn actuate(ctx: &Context<Self>) -> Result<()> {
        let vrf_account_info = &ctx.accounts.vrf;
        let vrf = VrfAccountData::new(vrf_account_info)?;

        let combination = &mut ctx.accounts.combination.load_mut()?;
        combination.result_buffer = vrf.get_result()?;

        Ok(())
    }
}

#[error_code]
pub enum CombineErrorCode {
    PaymentNotComplete,
    RandomAlreadyExists
}