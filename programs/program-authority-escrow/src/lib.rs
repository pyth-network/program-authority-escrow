use anchor_lang::prelude::*;
use anchor_lang::solana_program::bpf_loader_upgradeable;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_lang::solana_program::program::invoke;

#[cfg(test)]
mod tests;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod program_authority_escrow {
    use super::*;

    pub fn propose(ctx: Context<Propose>) -> Result<()> {
        let current_authority = &ctx.accounts.current_authority;
        let escrow_authority = &ctx.accounts.escrow_authority;
        let program = &ctx.accounts.program_account;

        invoke(
            &bpf_loader_upgradeable::set_upgrade_authority(&program.key(), &current_authority.key(), Some(&escrow_authority.key())),
            &ctx.accounts.to_account_infos()
        )?;
        Ok(())
    }

    pub fn revert(ctx: Context<Propose>) -> Result<()> {
        let current_authority = &ctx.accounts.current_authority;
        let new_authority = &ctx.accounts.new_authority;
        let escrow_authority = &ctx.accounts.escrow_authority;

        let program = &ctx.accounts.program_account;

        invoke_signed(
            &bpf_loader_upgradeable::set_upgrade_authority(&program.key(), &escrow_authority.key(), Some(&current_authority.key())),
            &ctx.accounts.to_account_infos(),
            &[&[current_authority.key().as_ref(),new_authority.key().as_ref()]]
        )?;
        Ok(())
    }

    pub fn accept(ctx: Context<Accept>) -> Result<()> {
        let current_authority = &ctx.accounts.current_authority;
        let new_authority = &ctx.accounts.new_authority;
        let escrow_authority = &ctx.accounts.escrow_authority;
        let program = &ctx.accounts.program_account;

        invoke_signed(
            &bpf_loader_upgradeable::set_upgrade_authority(&program.key(), &escrow_authority.key(), Some(&new_authority.key())),
            &ctx.accounts.to_account_infos(),
            &[&[current_authority.key().as_ref(),new_authority.key().as_ref()]]
        )?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Propose<'info> {
    pub current_authority : Signer<'info>,
    /// CHECK: Unchecked new authority, can be a native wallet or a PDA of another program
    pub new_authority : AccountInfo<'info>,
    #[account(seeds = [current_authority.key().as_ref(),new_authority.key().as_ref()], bump)]
    pub escrow_authority : SystemAccount<'info>,
    #[account(executable, constraint = matches!(program_account.as_ref(), UpgradeableLoaderState::Program{..}))]
    pub program_account : Account<'info, UpgradeableLoaderState>,
    #[account(seeds = [program_account.key().as_ref()], bump, owner = bpf_upgradable_loader.key())]
    pub program_data: Account<'info, ProgramData>,
    pub bpf_upgradable_loader : Program<'info, BpfUpgradableLoader>
}

#[derive(Accounts)]
pub struct Accept<'info> {
    /// CHECK: CPI will have the wrong seeds and fail if this is the wrong current authority
    pub current_authority : AccountInfo<'info>,
    pub new_authority : Signer<'info>,
    #[account(seeds = [current_authority.key().as_ref(),new_authority.key().as_ref()], bump)]
    pub escrow_authority : SystemAccount<'info>,
    #[account(executable, constraint = matches!(program_account.as_ref(), UpgradeableLoaderState::Program{..}))]
    pub program_account : Account<'info, UpgradeableLoaderState>,
    #[account(seeds = [program_account.key().as_ref()], bump, owner = bpf_upgradable_loader.key())]
    pub program_data: Account<'info, ProgramData>,
    pub bpf_upgradable_loader : Program<'info, BpfUpgradableLoader>
}

#[derive(Clone)]
pub struct BpfUpgradableLoader {

}

impl Id for BpfUpgradableLoader {
    fn id() -> Pubkey {
        bpf_loader_upgradeable::id()
    }
}