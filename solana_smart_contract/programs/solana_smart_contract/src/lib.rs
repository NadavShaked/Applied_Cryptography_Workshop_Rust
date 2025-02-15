use anchor_lang::prelude::*;
use anchor_lang::system_program;

declare_id!("AS3mJ2X2HY6jrGD25QFHpFZWA5u3uFRBYEJgKDJWkmaZ");

#[program]
pub mod solana_smart_contract {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let mediator = &ctx.accounts.mediator;
        let program_context = &ctx.accounts.program_context;

        msg!("Mediator account created! Current mediator balance: {}", mediator.balance);
        msg!("Program Context account created! Current subscription duration: {}", program_context.subscription_duration);
        Ok(())
    }

    pub fn set_client_curve_points(
        ctx: Context<SetClientCurvePoints>,
        g_norm: [u8; 96],
        v_norm: [u8; 96]
    ) -> Result<()> {
        let payer = &ctx.accounts.payer;
        let mediator = &ctx.accounts.mediator;
        let program_context = &mut ctx.accounts.program_context;

        //todo: validate g_norm is on curve instead
        // Validate that g_norm is not the default array of zeros
        if g_norm.iter().all(|&x| x == 0) {
            return Err(ErrorCode::InvalidCurvePoints.into()); // Return error if g_norm is all zeros
        }

        //todo: validate v_norm is on curve instead
        // Validate that v_norm is not the default array of zeros
        if v_norm.iter().all(|&x| x == 0) {
            return Err(ErrorCode::InvalidCurvePoints.into()); // Return error if v_norm is all zeros
        }

        // Validate that program_context.g_norm is not set
        if !program_context.g_norm.iter().all(|&x| x == 0) {
            return Err(ErrorCode::CurvePointsAlreadySet.into()); // Return error if g_norm is all zeros
        }

        // Validate that program_context.v_norm is not set
        if !program_context.v_norm.iter().all(|&x| x == 0) {
            return Err(ErrorCode::CurvePointsAlreadySet.into()); // Return error if v_norm is all zeros
        }

        let transfer_amount = 1_000_000_000; // 1 SOL in lamports

        // Ensure the client has at least 1 SOL (1_000_000_000 lamports)
        if payer.lamports() < transfer_amount {
            return Err(ErrorCode::InsufficientFunds.into());
        }

        // Perform the transfer of 1 SOL (1_000_000_000 lamports) from the client to the mediator
        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.payer.key(),
            &ctx.accounts.mediator.key(),
            transfer_amount,
        );

        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.payer.to_account_info(),
                ctx.accounts.mediator.to_account_info(),
            ],
        ).expect("Transaction failed");

        // Initialize shared context values with subscription_duration = 0
        program_context.is_subscription_ended = false;  // Subscription is active at the start
        program_context.is_server_turn = true;      // turn the turn to the server
        program_context.subscription_duration = 1;      // Set default value to 0
        program_context.mediator_balance = transfer_amount;  // Set mediator balance to 1 SOL

        program_context.g_norm = g_norm;
        program_context.v_norm = v_norm;

        msg!("Transferred 1 SOL from client {:?} to mediator {:?}", payer.key(), mediator.key());
        msg!("Initialized shared context for subscription: Duration: {}, Mediator Balance: {}",
             program_context.subscription_duration,
             program_context.mediator_balance);
        msg!("Stored g_norm and v_norm in shared context");

        Ok(())
    }

    pub fn extend_subscription(
        ctx: Context<ExtendSubscription>
    ) -> Result<()> {
        let payer = &ctx.accounts.payer;
        let mediator = &ctx.accounts.mediator;
        let program_context = &mut ctx.accounts.program_context;

        if program_context.subscription_duration < 1 {
            return Err(ErrorCode::SubscriptionDoesntStart.into());
        }

        if program_context.is_server_turn {
            return Err(ErrorCode::NotClientTurn.into());
        }

        let transfer_amount = 1_000_000_000; // 1 SOL in lamports

        // Ensure the client has at least 1 SOL (1_000_000_000 lamports)
        if payer.lamports() < transfer_amount {
            return Err(ErrorCode::InsufficientFunds.into());
        }

        // Perform the transfer of 1 SOL (1_000_000_000 lamports) from the client to the mediator
        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.payer.key(),
            &ctx.accounts.mediator.key(),
            transfer_amount,
        );

        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.payer.to_account_info(),
                ctx.accounts.mediator.to_account_info(),
            ],
        ).expect("Transaction failed");

        // Initialize shared context values with subscription_duration = 0
        program_context.is_server_turn = true;      // turn the turn to the server
        program_context.subscription_duration += 1;      // increment the subscription duration
        program_context.mediator_balance += transfer_amount;  // increment mediator balance by 1

        msg!("Transferred 1 SOL from client {:?} to mediator {:?}", payer.key(), mediator.key());
        msg!("Initialized shared context for subscription: Duration: {}, Mediator Balance: {}",
             program_context.subscription_duration,
             program_context.mediator_balance);
        msg!("Stored g_norm and v_norm in shared context");

        Ok(())
    }

    pub fn end_subscription(
        ctx: Context<EndSubscription>
    ) -> Result<()> {
        let program_context = &mut ctx.accounts.program_context;

        // Initialize shared context values with subscription_duration = 0
        program_context.is_server_turn = true;      // turn the turn to the server
        program_context.is_subscription_ended = true;   // end the client subscription

        msg!("Subscription ended.");

        Ok(())
    }

    pub fn retrieve(
        ctx: Context<Retrieve>
    ) -> Result<()> {
        let server = &ctx.accounts.server;
        let mediator = &ctx.accounts.mediator;
        let program_context = &mut ctx.accounts.program_context;

        let mediator_lamports = mediator.to_account_info().lamports();  // Dereference to get lamports value

        if !program_context.is_server_turn {
            return Err(ErrorCode::NotServerTurn.into());
        }

        if program_context.is_subscription_ended {
            // Perform the transfer from the mediator to the server
            let ix = anchor_lang::solana_program::system_instruction::transfer(
                &ctx.accounts.mediator.key(),
                &ctx.accounts.server.key(),
                mediator_lamports,
            );

            anchor_lang::solana_program::program::invoke(
                &ix,
                &[
                    ctx.accounts.mediator.to_account_info(),
                    ctx.accounts.server.to_account_info(),
                ],
            ).expect("Transaction failed");

            msg!("Transferred {:?} SOL from mediator {:?} to server {:?}", mediator_lamports, mediator.key(), server.key());
        }
        else {
            let transfer_amount = 1_000_000_000; // 1 SOL in lamports

            // Ensure the mediator hold more than 5 SOL so will transfer SOL to server
            if mediator_lamports > 5 * transfer_amount {
                // Perform the transfer from the mediator to the server
                let ix = anchor_lang::solana_program::system_instruction::transfer(
                    &ctx.accounts.mediator.key(),
                    &ctx.accounts.server.key(),
                    transfer_amount,
                );

                anchor_lang::solana_program::program::invoke(
                    &ix,
                    &[
                        ctx.accounts.mediator.to_account_info(),
                        ctx.accounts.server.to_account_info(),
                    ],
                ).expect("Transaction failed");
            }

            msg!("Transferred 1 SOL from mediator {:?} to server {:?}", mediator.key(), server.key());
        }

        // Initialize shared context values with subscription_duration = 0
        program_context.is_server_turn = false;      // turn the false to the server

        Ok(())
    }
}

#[error_code]
pub enum ErrorCode {
    #[msg("The curve points cannot be the default array of zeros.")]
    InvalidCurvePoints,

    #[msg("The curve points already set.")]
    CurvePointsAlreadySet,

    #[msg("Client has insufficient funds to initialize the subscription.")]
    InsufficientFunds,

    #[msg("The subscription doesnt started.")]
    SubscriptionDoesntStart,

    #[msg("This is not client turn.")]
    NotClientTurn,

    #[msg("This is not server turn.")]
    NotServerTurn,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = 8 + 8
    )]
    pub mediator: Account<'info, Mediator>,

    #[account(
        init,
        payer = payer,
        space = 8 + 1 + 1 + 8 + 8 + 96 + 96
    )]
    pub program_context: Account<'info, ProgramContext>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetClientCurvePoints<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(mut)]
    pub mediator: Account<'info, Mediator>,

    #[account(mut)]
    pub program_context: Account<'info, ProgramContext>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ExtendSubscription<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(mut)]
    pub mediator: Account<'info, Mediator>,

    #[account(mut)]
    pub program_context: Account<'info, ProgramContext>,

    pub system_program: Program<'info, System>, //todo: maybe not needed
}

#[derive(Accounts)]
pub struct EndSubscription<'info> {
    #[account(mut)]
    pub program_context: Account<'info, ProgramContext>,    //todo: maybe not needed
}

#[derive(Accounts)]
pub struct Retrieve<'info> {
    #[account(mut)]
    pub server: Signer<'info>,

    #[account(mut)]
    pub mediator: Account<'info, Mediator>,

    #[account(mut)]
    pub program_context: Account<'info, ProgramContext>,

    pub system_program: Program<'info, System>, //todo: maybe not needed
}

#[account]
pub struct Mediator {
    pub balance: u64,          // Mediator's balance in lamports    //todo: not needed
}

#[account]
pub struct ProgramContext {
    pub is_subscription_ended: bool,    // Indicates if the subscription has ended
    pub is_server_turn: bool,           // Indicates if the server turn
    pub subscription_duration: u64,     // Subscription duration in seconds
    pub mediator_balance: u64,          // Mediator's balance in lamports
    pub g_norm: [u8; 96],               // Array to store g_norm (96 bytes)
    pub v_norm: [u8; 96],               // Array to store v_norm (96 bytes)
}
