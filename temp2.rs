use anchor_lang::prelude::*;

declare_id!("AS3mJ2X2HY6jrGD25QFHpFZWA5u3uFRBYEJgKDJWkmaZ");

#[program]
pub mod solana_smart_contract {
    use super::*;

    pub fn initialize_subscription(
        ctx: Context<InitializeSubscription>
    ) -> Result<()> {
        let client = &ctx.accounts.client;
        let mediator = &ctx.accounts.mediator;
        let shared_context = &ctx.accounts.shared_context;

        msg!("Mediator account created! Current mediator balance: {}", mediator.balance);
        msg!("Shared Context account created! Current duration: {}", shared_context.subscription_duration);

        Ok(())
    }

    // pub fn initialize_subscription(
    //     ctx: Context<InitializeSubscription>,
    //     g_norm: [u8; 96],
    //     v_norm: [u8; 96]
    // ) -> Result<()> {
    //     let client = &ctx.accounts.client;
    //     let mediator = &ctx.accounts.mediator;
    //     let shared_context = &ctx.accounts.shared_context;
    //
    //     // Ensure the client has at least 1 SOL (1_000_000_000 lamports)
    //     if client.lamports() < 1_000_000_000 {
    //         return Err(ErrorCode::InsufficientFunds.into());
    //     }
    //
    //     // Perform the transfer of 1 SOL (1_000_000_000 lamports) from the client to the mediator
    //     let transfer_amount = 1_000_000_000; // 1 SOL in lamports
    //     **client.to_account_info().lamports.borrow_mut() -= transfer_amount;
    //     **mediator.to_account_info().lamports.borrow_mut() += transfer_amount;
    //
    //     // Initialize shared context values with subscription_duration = 0
    //     shared_context.is_subscription_ended = false;  // Subscription is active at the start
    //     shared_context.subscription_duration = 0;      // Set default value to 0
    //     shared_context.mediator_balance = transfer_amount;  // Set mediator balance to 1 SOL
    //
    //     // Store the g_norm and v_norm in the shared context
    //     shared_context.g_norm = g_norm;
    //     shared_context.v_norm = v_norm;
    //
    //     msg!("Transferred 1 SOL from client {:?} to mediator {:?}", client.key(), mediator.key());
    //     msg!("Initialized shared context for subscription: Duration: {}, Mediator Balance: {}",
    //          shared_context.subscription_duration,
    //          shared_context.mediator_balance);
    //     msg!("Stored g_norm and v_norm in shared context");
    //
    //     Ok(())
    // }

    // pub fn extend_subscription(ctx: Context<ExtendSubscription>) -> Result<()> {
    //     let client = &ctx.accounts.client;
    //     let mediator = &mut ctx.accounts.mediator;
    //     let shared_context = &mut ctx.accounts.shared_context;
    //
    //     // Ensure the client has at least 1 SOL (1_000_000_000 lamports) to transfer
    //     if client.lamports() < 1_000_000_000 {
    //         return Err(ErrorCode::InsufficientFunds.into());
    //     }
    //
    //     // Perform the transfer of 1 SOL (1_000_000_000 lamports) from the client to the mediator
    //     let transfer_amount = 1_000_000_000; // 1 SOL in lamports
    //     **client.to_account_info().lamports.borrow_mut() -= transfer_amount;
    //     **mediator.to_account_info().lamports.borrow_mut() += transfer_amount;
    //
    //     // Optionally, you can extend the subscription duration if needed
    //     // Here we just log the extension, but if you'd like, you can increment the duration, etc.
    //     shared_context.mediator_balance += transfer_amount; // Add the new balance to the mediator balance
    //     shared_context.subscription_duration += 1;      // Add one to the subscription duration
    //
    //     msg!("Transferred 1 SOL from client {:?} to mediator {:?}", client.key(), mediator.key());
    //     msg!("Subscription extended. New mediator balance: {}", shared_context.mediator_balance);
    //
    //     Ok(())
    // }
    //
    // pub fn end_subscription(ctx: Context<EndSubscription>) -> Result<()> {
    //     let client = &ctx.accounts.client;
    //     let storage_service = &ctx.accounts.storage_service;
    //     let mediator = &mut ctx.accounts.mediator;
    //     let shared_context = &mut ctx.accounts.shared_context;
    //
    //     // Check that the client or storage service is calling this function (or whoever is allowed)
    //     if client.key() != storage_service.key() {  // You can adjust the validation as needed
    //         return Err(ErrorCode::UnauthorizedCaller.into());
    //     }
    //
    //     // Ensure that the mediator has balance to transfer
    //     let mediator_balance = **mediator.to_account_info().lamports.borrow();
    //     if mediator_balance == 0 {
    //         return Err(ErrorCode::NoBalanceToTransfer.into());
    //     }
    //
    //     // Transfer all lamports (SOL) from the mediator to the storage service
    //     **mediator.to_account_info().lamports.borrow_mut() -= mediator_balance;
    //     **storage_service.to_account_info().lamports.borrow_mut() += mediator_balance;
    //
    //     // Set the shared context's subscription to ended and reset mediator balance
    //     shared_context.is_subscription_ended = true;
    //     shared_context.mediator_balance = 0;
    //
    //     // Log the transfer and state update
    //     msg!("Transferred all SOL from mediator {:?} to storage service {:?}.", mediator.key(), storage_service.key());
    //     msg!("Subscription ended. Shared context updated: is_subscription_ended = {}, mediator_balance = {}",
    //          shared_context.is_subscription_ended, shared_context.mediator_balance);
    //
    //     Ok(())
    // }
    //
    // pub fn conditional_transfer(ctx: Context<ConditionalTransfer>, sigma_norm: [u8; 48], multiplication_sum_norm: [u8; 48]) -> Result<()> {
    //     let client = &ctx.accounts.client;
    //     let storage_service = &ctx.accounts.storage_service;
    //     let mediator = &mut ctx.accounts.mediator;
    //
    //     // Check if the two byte arrays are equal
    //     if sigma_norm == multiplication_sum_norm {
    //         // If they are equal, transfer 1 SOL (1_000_000_000 lamports) from the mediator to the storage_service
    //         let transfer_amount = 1_000_000_000; // 1 SOL in lamports
    //         let mediator_balance = **mediator.to_account_info().lamports.borrow();
    //
    //         if mediator_balance < transfer_amount {
    //             return Err(ErrorCode::InsufficientBalanceInMediator.into());
    //         }
    //
    //         **mediator.to_account_info().lamports.borrow_mut() -= transfer_amount;
    //         **storage_service.to_account_info().lamports.borrow_mut() += transfer_amount;
    //
    //         msg!("Transferred 1 SOL from mediator {:?} to storage service {:?}", mediator.key(), storage_service.key());
    //     } else {
    //         // If they are not equal, transfer all the mediator's balance to the client
    //         let mediator_balance = **mediator.to_account_info().lamports.borrow();
    //
    //         if mediator_balance == 0 {
    //             return Err(ErrorCode::NoBalanceInMediator.into());
    //         }
    //
    //         **mediator.to_account_info().lamports.borrow_mut() -= mediator_balance;
    //         **client.to_account_info().lamports.borrow_mut() += mediator_balance;
    //
    //         msg!("Transferred all SOL from mediator {:?} to client {:?}", mediator.key(), client.key());
    //     }
    //
    //     Ok(())
    // }
}

#[error_code]
pub enum ErrorCode {
    #[msg("Client has insufficient funds to initialize the subscription.")]
    InsufficientFunds,

    #[msg("Unauthorized caller. Only the client or storage service can call this function.")]
    UnauthorizedCaller,

    #[msg("No balance available in the mediator to transfer.")]
    NoBalanceToTransfer,

    #[msg("Insufficient balance in the mediator to transfer 1 SOL.")]
    InsufficientBalanceInMediator,

    #[msg("No balance available in the mediator to transfer.")]
    NoBalanceInMediator,
}

#[derive(Accounts)]
pub struct InitializeSubscription<'info> {
    #[account(mut)]
    pub client: Signer<'info>,  // The client initiating the transfer

    #[account(
        init,
        payer = client,
        space = 8 + 8,  // Adjust space for mediator account (2 * u64 for balance storage)
    )]
    pub mediator: Account<'info, Mediator>,  // Mediator account receiving 1 SOL

    #[account(
        init,
        payer = client,
        space = 8 + 8 + 8 + 8,  // Space for isSubscriptionEnded, subscriptionDuration, mediatorBalance (4 * u64)
    )]
    pub shared_context: Account<'info, SharedContext>,  // Shared context storing subscription state

    pub system_program: Program<'info, System>, // System program required for account creation
}

#[derive(Accounts)]
pub struct ExtendSubscription<'info> {
    #[account(mut)]
    pub client: Signer<'info>,  // The client initiating the transfer

    #[account(
        init,
        payer = client,
        space = 8 + 8
    )]
    pub mediator: Account<'info, Mediator>,  // The mediator account receiving 1 SOL

    #[account(
        init,
        payer = client,
        space = 8 + 1 + 8 + 8 + 96 + 96 + 7
    )]
    pub shared_context: Account<'info, SharedContext>,  // Shared context to track subscription state

    pub system_program: Program<'info, System>, // System program required for account creation
}

#[derive(Accounts)]
pub struct EndSubscription<'info> {
    #[account(mut)]
    pub client: Signer<'info>,  // The client initiating the end of the subscription

    #[account(mut)]
    pub storage_service: Signer<'info>,  // The storage service receiving the mediator's balance

    #[account(mut)]
    pub mediator: Account<'info, Mediator>,  // The mediator account holding the balance to be transferred

    #[account(mut)]
    pub shared_context: Account<'info, SharedContext>,  // Shared context to update the subscription state

    pub system_program: Program<'info, System>, // System program for basic account operations
}

#[derive(Accounts)]
pub struct ConditionalTransfer<'info> {
    #[account(mut)]
    pub client: Signer<'info>,  // The client initiating the transfer

    #[account(mut)]
    pub storage_service: Signer<'info>,  // The storage service receiving the balance if arrays are equal

    #[account(mut)]
    pub mediator: Account<'info, Mediator>,  // The mediator account holding the balance to be transferred

    pub system_program: Program<'info, System>, // System program required for basic account operations
}


#[account]
pub struct Mediator {
    pub balance: u64,  // Tracks the balance of the mediator account
}

#[account]
pub struct SharedContext {
    pub is_subscription_ended: bool,  // Indicates if the subscription has ended
    pub subscription_duration: u64,   // Subscription duration in seconds
    pub mediator_balance: u64,        // Mediator's balance in lamports
    pub g_norm: [u8; 96],  // Array to store g_norm (96 bytes)
    pub v_norm: [u8; 96],  // Array to store v_norm (96 bytes)
}
