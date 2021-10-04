use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Transfer};

declare_id!("5CZmHUkQTnp3pX28LD6hy4McMM9gbzn154S9w7cGAsoR");

#[program]
mod ftr_distributor {
    use super::*;

    #[access_control(InitializeDistributor::accounts(&ctx, nonce))]
    pub fn initialize_distributor(
        ctx: Context<InitializeDistributor>,
        price_of_contract: u64,
        ftr_per_contract: u64,
        no_of_contracts: u64,
        nonce: u8,
    ) -> ProgramResult {
        let distributor_account = &mut ctx.accounts.distributor_account;
        distributor_account.price_of_contract = price_of_contract;
        distributor_account.ftr_per_contract = ftr_per_contract;
        distributor_account.nonce = nonce;
        distributor_account.distribution_authority = *ctx.accounts.distribution_authority.key;
        distributor_account.distributor_usdc = *ctx.accounts.distributor_usdc.to_account_info().key;
        distributor_account.distributor_ftr = *ctx.accounts.distributor_ftr.to_account_info().key;
        distributor_account.distributor_contract =
            *ctx.accounts.distributor_contract.to_account_info().key;
        //this address will be used as seed for creating the distiributor Signer account PDA
        distributor_account.ftr_mint = ctx.accounts.distributor_ftr.mint;

        //Transfer Contract Tokens from creator to distributor account
        let cpi_accounts = Transfer {
            from: ctx.accounts.creator_contract.to_account_info(),
            to: ctx.accounts.distributor_contract.to_account_info(),
            authority: ctx.accounts.distribution_authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.clone();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, no_of_contracts)?;

        Ok(())
    }

    pub fn update_distributor(
        ctx: Context<UpdateDistributor>,
        price_of_contract: Option<u64>,
        ftr_per_contract: Option<u64>,
    ) -> ProgramResult {
        let distributor_account = &mut ctx.accounts.distributor_account;
        if let Some(price_of_contract) = price_of_contract {
            distributor_account.price_of_contract = price_of_contract;
        };
        if let Some(ftr_per_contract) = ftr_per_contract {
            distributor_account.ftr_per_contract = ftr_per_contract;
        }

        Ok(())
    }

    pub fn distribute(ctx: Context<Distribute>, no_of_contracts_required: u64) -> ProgramResult {
        // While token::transfer will check this, we prefer a verbose err msg.
        let distributor_account = &ctx.accounts.distributor_account;
        if ctx.accounts.distributor_contract.amount < no_of_contracts_required {
            return Err(ErrorCode::InSufficientNoOfContracts.into());
        }
        let amount_in_usdc = distributor_account.price_of_contract * no_of_contracts_required;
        let amount_in_ftr = distributor_account.ftr_per_contract * no_of_contracts_required;
        // Transfer user's USDC to distributor USDC account.
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_usdc.to_account_info(),
            to: ctx.accounts.distributor_usdc.to_account_info(),
            authority: ctx.accounts.user_authority.clone(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount_in_usdc)?;

        // Transfer user's FTR to disributor FTR account
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_ftr.to_account_info(),
            to: ctx.accounts.distributor_ftr.to_account_info(),
            authority: ctx.accounts.user_authority.clone(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount_in_ftr)?;

        // Transfer contract token from distributor account to user account
        let seeds = &[
            ctx.accounts.distributor_account.ftr_mint.as_ref(),
            &[ctx.accounts.distributor_account.nonce],
        ];
        let signer = &[&seeds[..]];
        let cpi_accounts = Transfer {
            from: ctx.accounts.distributor_contract.to_account_info(),
            to: ctx.accounts.user_contract.to_account_info(),
            authority: ctx.accounts.distributor_signer.clone(),
        };
        let cpi_program = ctx.accounts.token_program.clone();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, no_of_contracts_required)?;

        Ok(())
    }

    pub fn redeem(ctx: Context<Redeem>, no_of_contracts_redeemed: u64) -> ProgramResult {
        let distributor_account = &ctx.accounts.distributor_account;

        let amount_in_usdc = distributor_account.price_of_contract * no_of_contracts_redeemed;
        let amount_in_ftr = distributor_account.ftr_per_contract * no_of_contracts_redeemed;
        // Transfer user's contract token to distributor contract account.
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_contract.to_account_info(),
            to: ctx.accounts.distributor_contract.to_account_info(),
            authority: ctx.accounts.user_authority.clone(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, no_of_contracts_redeemed)?;

        // Transfer distributors USDC to user USDC
        let seeds = &[
            ctx.accounts.distributor_account.ftr_mint.as_ref(),
            &[ctx.accounts.distributor_account.nonce],
        ];
        let signer = &[&seeds[..]];
        let cpi_accounts = Transfer {
            from: ctx.accounts.distributor_usdc.to_account_info(),
            to: ctx.accounts.user_usdc.to_account_info(),
            authority: ctx.accounts.distributor_signer.clone(),
        };
        let cpi_program = ctx.accounts.token_program.clone();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, amount_in_usdc)?;

        // Transfer distributors FTR to user FTR
        let seeds = &[
            ctx.accounts.distributor_account.ftr_mint.as_ref(),
            &[ctx.accounts.distributor_account.nonce],
        ];
        let signer = &[&seeds[..]];
        let cpi_accounts = Transfer {
            from: ctx.accounts.distributor_ftr.to_account_info(),
            to: ctx.accounts.user_ftr.to_account_info(),
            authority: ctx.accounts.distributor_signer.clone(),
        };
        let cpi_program = ctx.accounts.token_program.clone();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, amount_in_ftr)?;
        Ok(())
    }

    pub fn backdoor_withdraw(
        ctx: Context<BackdoorWithdraw>,
        no_of_contracts_to_withdraw: Option<u64>,
        amount_of_usdc_to_withdraw: Option<u64>,
        amount_of_ftr_to_withdraw: Option<u64>,
    ) -> ProgramResult {

        //for withdrawal of contracts
        if let Some(no_of_contracts_to_withdraw) = no_of_contracts_to_withdraw {
            // While token::transfer will check this, we prefer a verbose err msg.
            if no_of_contracts_to_withdraw > ctx.accounts.distributor_contract.amount {
                return Err(ErrorCode::InSufficientNoOfContracts.into()); 
            }
            let seeds = &[
                ctx.accounts.distributor_account.ftr_mint.as_ref(),
                &[ctx.accounts.distributor_account.nonce],
            ];
            let signer = &[&seeds[..]];
            let cpi_accounts = Transfer {
                from: ctx.accounts.distributor_contract.to_account_info(),
                to: ctx.accounts.creator_contract.to_account_info(),
                authority: ctx.accounts.distributor_signer.clone(),
            };
            let cpi_program = ctx.accounts.token_program.clone();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            token::transfer(cpi_ctx, no_of_contracts_to_withdraw)?;
        };

        //for withdrawal of usdc
        if let Some(amount_of_usdc_to_withdraw) = amount_of_usdc_to_withdraw {
            // While token::transfer will check this, we prefer a verbose err msg.
            if amount_of_usdc_to_withdraw > ctx.accounts.distributor_usdc.amount {
                return Err(ErrorCode::InSufficientAmountOfUsdc.into()); 
            };
            let seeds = &[
                ctx.accounts.distributor_account.ftr_mint.as_ref(),
                &[ctx.accounts.distributor_account.nonce],
            ];
            let signer = &[&seeds[..]];
            let cpi_accounts = Transfer {
                from: ctx.accounts.distributor_usdc.to_account_info(),
                to: ctx.accounts.creator_usdc.to_account_info(),
                authority: ctx.accounts.distributor_signer.clone(),
            };
            let cpi_program = ctx.accounts.token_program.clone();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            token::transfer(cpi_ctx, amount_of_usdc_to_withdraw)?;
        };

        //for withdrawal of ftr
        if let Some(amount_of_ftr_to_withdraw) = amount_of_ftr_to_withdraw {
            // While token::transfer will check this, we prefer a verbose err msg.
            if amount_of_ftr_to_withdraw > ctx.accounts.distributor_ftr.amount {
                return Err(ErrorCode::InSufficientAmountOfFtr.into()); 
            }
            let seeds = &[
                ctx.accounts.distributor_account.ftr_mint.as_ref(),
                &[ctx.accounts.distributor_account.nonce],
            ];
            let signer = &[&seeds[..]];
            let cpi_accounts = Transfer {
                from: ctx.accounts.distributor_ftr.to_account_info(),
                to: ctx.accounts.creator_ftr.to_account_info(),
                authority: ctx.accounts.distributor_signer.clone(),
            };
            let cpi_program = ctx.accounts.token_program.clone();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            token::transfer(cpi_ctx, amount_of_ftr_to_withdraw)?;
        };

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeDistributor<'info> {
    #[account(zero)]
    pub distributor_account: Box<Account<'info, DistributorAccount>>,
    pub distributor_signer: AccountInfo<'info>,
    #[account(constraint = distributor_ftr.owner == *distributor_signer.key)]
    pub distributor_ftr: Account<'info, TokenAccount>,
    #[account(constraint = distributor_usdc.owner == *distributor_signer.key)]
    pub distributor_usdc: Account<'info, TokenAccount>,
    #[account(mut, constraint = distributor_contract.owner == *distributor_signer.key)]
    pub distributor_contract: Account<'info, TokenAccount>,
    #[account(signer)]
    pub distribution_authority: AccountInfo<'info>,
    #[account(mut, constraint = creator_contract.owner == *distribution_authority.key)]
    pub creator_contract: Account<'info, TokenAccount>,
    #[account(constraint = token_program.key == &token::ID)]
    pub token_program: AccountInfo<'info>,
}

impl<'info> InitializeDistributor<'info> {
    fn accounts(ctx: &Context<InitializeDistributor<'info>>, nonce: u8) -> Result<()> {
        let expected_signer = Pubkey::create_program_address(
            &[ctx.accounts.distributor_ftr.mint.as_ref(), &[nonce]],
            ctx.program_id,
        )
        .map_err(|_| ErrorCode::InvalidNonce)?;
        if ctx.accounts.distributor_signer.key != &expected_signer {
            return Err(ErrorCode::InvalidNonce.into());
        }
        Ok(())
    }
}

#[derive(Accounts)]
pub struct UpdateDistributor<'info> {
    #[account(mut, has_one = distribution_authority)]
    pub distributor_account: Account<'info, DistributorAccount>,
    #[account(signer)]
    distribution_authority: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct Distribute<'info> {
    #[account(has_one = distributor_ftr,has_one = distributor_contract, has_one = distributor_usdc)]
    pub distributor_account: Account<'info, DistributorAccount>,
    #[account(
        seeds = [distributor_account.ftr_mint.as_ref()],
        bump = distributor_account.nonce,
    )]
    distributor_signer: AccountInfo<'info>,
    #[account(mut, constraint = distributor_usdc.owner == *distributor_signer.key)]
    pub distributor_usdc: Account<'info, TokenAccount>,
    #[account(mut, constraint = distributor_usdc.owner == *distributor_signer.key)]
    pub distributor_ftr: Account<'info, TokenAccount>,
    #[account(mut, constraint = distributor_usdc.owner == *distributor_signer.key)]
    pub distributor_contract: Account<'info, TokenAccount>,
    #[account(signer)]
    pub user_authority: AccountInfo<'info>,
    #[account(mut, constraint = user_usdc.owner == *user_authority.key)]
    pub user_usdc: Account<'info, TokenAccount>,
    #[account(mut, constraint = user_ftr.owner == *user_authority.key)]
    pub user_ftr: Box<Account<'info, TokenAccount>>,
    #[account(mut, constraint = user_contract.owner == *user_authority.key)]
    pub user_contract: Box<Account<'info, TokenAccount>>,
    #[account(constraint = token_program.key == &token::ID)]
    pub token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct Redeem<'info> {
    #[account(has_one = distributor_ftr,has_one = distributor_contract, has_one = distributor_usdc)]
    pub distributor_account: Account<'info, DistributorAccount>,
    #[account(
        seeds = [distributor_account.ftr_mint.as_ref()],
        bump = distributor_account.nonce,
    )]
    distributor_signer: AccountInfo<'info>,
    #[account(mut, constraint = distributor_usdc.owner == *distributor_signer.key)]
    pub distributor_usdc: Account<'info, TokenAccount>,
    #[account(mut, constraint = distributor_usdc.owner == *distributor_signer.key)]
    pub distributor_ftr: Account<'info, TokenAccount>,
    #[account(mut, constraint = distributor_usdc.owner == *distributor_signer.key)]
    pub distributor_contract: Account<'info, TokenAccount>,
    #[account(signer)]
    pub user_authority: AccountInfo<'info>,
    #[account(mut, constraint = user_usdc.owner == *user_authority.key)]
    pub user_usdc: Account<'info, TokenAccount>,
    #[account(mut, constraint = user_ftr.owner == *user_authority.key)]
    pub user_ftr: Box<Account<'info, TokenAccount>>,
    #[account(mut, constraint = user_contract.owner == *user_authority.key)]
    pub user_contract: Box<Account<'info, TokenAccount>>,
    #[account(constraint = token_program.key == &token::ID)]
    pub token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct BackdoorWithdraw<'info> {
    #[account(has_one = distribution_authority)]
    pub distributor_account: Account<'info, DistributorAccount>,
    #[account(signer)]
    distribution_authority: AccountInfo<'info>,
    #[account(
        seeds = [distributor_account.ftr_mint.as_ref()],
        bump = distributor_account.nonce,
    )]
    distributor_signer: AccountInfo<'info>,
    #[account(mut, constraint = distributor_usdc.owner == *distributor_signer.key)]
    pub distributor_usdc: Account<'info, TokenAccount>,
    #[account(mut, constraint = distributor_usdc.owner == *distributor_signer.key)]
    pub distributor_ftr: Account<'info, TokenAccount>,
    #[account(mut, constraint = distributor_usdc.owner == *distributor_signer.key)]
    pub distributor_contract: Account<'info, TokenAccount>,
    #[account(mut, constraint = creator_usdc.owner == *distribution_authority.key)]
    pub creator_usdc: Account<'info, TokenAccount>,
    #[account(mut, constraint = creator_ftr.owner == *distribution_authority.key)]
    pub creator_ftr: Box<Account<'info, TokenAccount>>,
    #[account(mut, constraint = creator_contract.owner == *distribution_authority.key)]
    pub creator_contract: Box<Account<'info, TokenAccount>>,
    #[account(constraint = token_program.key == &token::ID)]
    pub token_program: AccountInfo<'info>,
}

#[account]
pub struct DistributorAccount {
    pub price_of_contract: u64,
    pub ftr_per_contract: u64,
    pub nonce: u8,
    pub distribution_authority: Pubkey,
    pub distributor_usdc: Pubkey,
    pub distributor_ftr: Pubkey,
    pub distributor_contract: Pubkey,
    pub ftr_mint: Pubkey,
}

#[error]
pub enum ErrorCode {
    #[msg("Given nonce is invalid")]
    InvalidNonce,
    #[msg("Insufficient no of contracts in the distributor")]
    InSufficientNoOfContracts,
    #[msg("Insufficient amount of usdc in distributor")]
    InSufficientAmountOfUsdc,
    #[msg("Insufficient amount of usdc in distributor")]
    InSufficientAmountOfFtr,
}
