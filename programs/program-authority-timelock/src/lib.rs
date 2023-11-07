#![deny(warnings)]
#![allow(clippy::result_large_err)]

use anchor_lang::{
    prelude::*,
    solana_program::{
        bpf_loader_upgradeable,
        program::{
            invoke,
            invoke_signed,
        },
    },
};

#[cfg(test)]
mod tests;

declare_id!("escMHe7kSqPcDHx4HU44rAHhgdTLBZkUrU39aN8kMcL");
const ONE_YEAR: i64 = 365 * 24 * 60 * 60;

#[program]
pub mod program_authority_escrow {
    use super::*;

    pub fn commit(ctx: Context<Commit>, timestamp: i64) -> Result<()> {
        let current_authority = &ctx.accounts.current_authority;
        let escrow_authority = &ctx.accounts.escrow_authority;
        let program_account = &ctx.accounts.program_account;

        invoke(
            &bpf_loader_upgradeable::set_upgrade_authority(
                &program_account.key(),
                &current_authority.key(),
                Some(&escrow_authority.key()),
            ),
            &ctx.accounts.to_account_infos(),
        )?;

        // Check that the timelock is no longer than 1 year
        if Clock::get()?.unix_timestamp.saturating_add(ONE_YEAR) < timestamp {
            return Err(ErrorCode::TimestampTooLate.into());
        }

        Ok(())
    }

    pub fn transfer(ctx: Context<Transfer>, timestamp: i64) -> Result<()> {
        let new_authority = &ctx.accounts.new_authority;
        let escrow_authority = &ctx.accounts.escrow_authority;
        let program_account = &ctx.accounts.program_account;

        invoke_signed(
            &bpf_loader_upgradeable::set_upgrade_authority(
                &program_account.key(),
                &escrow_authority.key(),
                Some(&new_authority.key()),
            ),
            &ctx.accounts.to_account_infos(),
            &[&[
                new_authority.key().as_ref(),
                timestamp.to_be_bytes().as_ref(),
                &[*ctx.bumps.get("escrow_authority").unwrap()],
            ]],
        )?;

        if Clock::get()?.unix_timestamp < timestamp {
            return Err(ErrorCode::TimestampTooEarly.into());
        }

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(timestamp : i64)]
pub struct Commit<'info> {
    pub current_authority:     Signer<'info>,
    /// CHECK: Unchecked new authority, can be a native wallet or a PDA of another program
    pub new_authority:         AccountInfo<'info>,
    #[account(seeds = [new_authority.key().as_ref(), timestamp.to_be_bytes().as_ref()], bump)]
    pub escrow_authority:      SystemAccount<'info>,
    #[account(executable, constraint = matches!(program_account.as_ref(), UpgradeableLoaderState::Program{..}))]
    pub program_account:       Account<'info, UpgradeableLoaderState>,
    #[account(mut, seeds = [program_account.key().as_ref()], bump, seeds::program = bpf_upgradable_loader.key())]
    pub program_data:          Account<'info, ProgramData>,
    pub bpf_upgradable_loader: Program<'info, BpfUpgradableLoader>,
}

#[derive(Accounts)]
#[instruction(timestamp : i64)]
pub struct Transfer<'info> {
    /// CHECK: Unchecked new authority, can be a native wallet or a PDA of another program
    pub new_authority:         AccountInfo<'info>,
    #[account(seeds = [new_authority.key().as_ref(), timestamp.to_be_bytes().as_ref()], bump)]
    pub escrow_authority:      SystemAccount<'info>,
    #[account(executable, constraint = matches!(program_account.as_ref(), UpgradeableLoaderState::Program{..}))]
    pub program_account:       Account<'info, UpgradeableLoaderState>,
    #[account(mut, seeds = [program_account.key().as_ref()], bump, seeds::program = bpf_upgradable_loader.key())]
    pub program_data:          Account<'info, ProgramData>,
    pub bpf_upgradable_loader: Program<'info, BpfUpgradableLoader>,
}

#[derive(Clone)]
pub struct BpfUpgradableLoader {}

impl Id for BpfUpgradableLoader {
    fn id() -> Pubkey {
        bpf_loader_upgradeable::id()
    }
}

#[error_code]
#[derive(PartialEq, Eq)]
pub enum ErrorCode {
    #[msg("Timestamp too early")]
    TimestampTooEarly,
    #[msg("Timestamp too late")]
    TimestampTooLate,
}
