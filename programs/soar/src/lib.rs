#![allow(clippy::result_large_err)]
#![allow(dead_code)]

use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

mod error;
mod instructions;
mod seeds;
mod state;
mod utils;

use error::CrateError;
use instructions::*;
use state::*;

#[program]
pub mod soar {
    use super::*;

    /// Initialize a new [Game] and register its [LeaderBoard].
    pub fn initialize_game(
        ctx: Context<InitializeGame>,
        game_meta: GameMeta,
        game_auth: Vec<Pubkey>,
    ) -> Result<()> {
        create_game::handler(ctx, game_meta, game_auth)
    }

    /// Update a [Game]'s meta-information or authority list.
    pub fn update_game(
        ctx: Context<UpdateGame>,
        new_meta: Option<GameMeta>,
        new_auth: Option<Vec<Pubkey>>,
    ) -> Result<()> {
        update_game::handler(ctx, new_meta, new_auth)
    }

    /// Add a new [Achievement] that can be attained for a particular [Game].
    pub fn add_achievement(
        ctx: Context<AddAchievement>,
        title: String,
        description: String,
        nft_meta: Pubkey,
    ) -> Result<()> {
        add_achievement::handler(ctx, title, description, nft_meta)
    }

    /// Update an [Achievement]'s meta information.
    pub fn update_achievement(
        ctx: Context<UpdateAchievement>,
        new_title: Option<String>,
        new_description: Option<String>,
        nft_meta: Option<Pubkey>,
    ) -> Result<()> {
        update_achievement::handler(ctx, new_title, new_description, nft_meta)
    }

    /// Overwrite the active [LeaderBoard] and set a newly created one.
    pub fn add_leaderboard(
        ctx: Context<AddLeaderBoard>,
        input: RegisterLeaderBoardInput,
    ) -> Result<()> {
        add_leaderboard::handler(ctx, input)
    }

    /// Create a [Player] account for a particular user.
    pub fn create_player(
        ctx: Context<NewPlayer>,
        username: String,
        nft_meta: Pubkey,
    ) -> Result<()> {
        create_player::handler(ctx, username, nft_meta)
    }

    /// Update the username or nft_meta for a [Player] account.
    pub fn update_player(
        ctx: Context<UpdatePlayer>,
        username: Option<String>,
        nft_meta: Option<Pubkey>,
    ) -> Result<()> {
        update_player::handler(ctx, username, nft_meta)
    }

    /// Register a [Player] for a particular [Leaderboard], resulting in a newly-
    /// created [PlayerEntryList] account.
    pub fn register_player(ctx: Context<RegisterPlayer>) -> Result<()> {
        register_player::handler(ctx)
    }

    /// Submit a score for a player and have it timestamped and added to the [PlayerEntryList].
    /// Optionally increase the player's rank if needed.
    pub fn submit_score(ctx: Context<SubmitScore>, score: u64, _rank: Option<u64>) -> Result<()> {
        submit_score::handler(ctx, score)
    }

    /// Merge multiple accounts as belonging to the same user. The `hint` argument
    /// specifies the number of additional accounts to be merged.
    pub fn merge_player_accounts<'info>(
        ctx: Context<'_, '_, '_, 'info, MergePlayerAccounts<'info>>,
        hint: u64,
    ) -> Result<()> {
        merge_players::handler(ctx, hint)
    }

    pub fn unlock_player_achievement(ctx: Context<UnlockPlayerAchievement>) -> Result<()> {
        unlock_player_achievement::handler(ctx)
    }

    pub fn add_reward(ctx: Context<AddReward>, input: RegisterNewRewardInput) -> Result<()> {
        add_reward::handler(ctx, input)
    }

    pub fn mint_reward(ctx: Context<MintReward>) -> Result<()> {
        mint_reward::handler(ctx)
    }

    pub fn verify_reward(ctx: Context<VerifyReward>) -> Result<()> {
        verify_reward::handler(ctx)
    }
}

#[derive(Accounts)]
#[instruction(meta: GameMeta, auth: Vec<Pubkey>)]
pub struct InitializeGame<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,
    #[account(
        init,
        payer = creator,
        space = Game::size_with_auths(auth.len()),
        seeds = [seeds::GAME, meta.title.as_bytes()],
        bump
    )]
    pub game: Account<'info, Game>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateGame<'info> {
    #[account(
        constraint = game.check_authority_is_signer(authority.key)
        @ CrateError::InvalidAuthority
    )]
    pub authority: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub game: Account<'info, Game>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(title: String)]
pub struct AddAchievement<'info> {
    #[account(
        mut,
        constraint = game.check_authority_is_signer(authority.key)
        @ CrateError::InvalidAuthority
    )]
    pub authority: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub game: Account<'info, Game>,
    #[account(
        init,
        payer = payer,
        space = Achievement::SIZE,
        seeds = [seeds::ACHIEVEMENT, game.key().as_ref(), title.as_bytes()],
        bump,
    )]
    pub new_achievement: Account<'info, Achievement>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateAchievement<'info> {
    #[account(
        mut,
        constraint = game.check_authority_is_signer(authority.key)
        @ CrateError::InvalidAuthority
    )]
    pub authority: Signer<'info>,
    pub game: Account<'info, Game>,
    #[account(mut, has_one = game)]
    pub achievement: Account<'info, Achievement>,
}

#[derive(Accounts)]
pub struct AddLeaderBoard<'info> {
    #[account(
        mut,
        constraint = game.check_authority_is_signer(authority.key)
        @ CrateError::InvalidAuthority
    )]
    pub authority: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub game: Account<'info, Game>,
    /// TODO: Close previous leaderboard account?
    #[account(
        init,
        payer = payer,
        space = LeaderBoard::SIZE,
        seeds = [seeds::LEADER, game.key().as_ref(), &next_leaderboard_id(&game).to_le_bytes()],
        bump,
    )]
    pub leaderboard: Account<'info, LeaderBoard>,
    pub system_program: Program<'info, System>,
}

fn next_leaderboard_id(game: &Account<'_, Game>) -> u64 {
    game.leaderboard.checked_add(1).unwrap()
}

#[derive(Accounts)]
pub struct NewPlayer<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init,
        payer = user,
        space = Player::SIZE,
        seeds = [seeds::PLAYER, user.key().as_ref()],
        bump,
    )]
    pub player_info: Account<'info, Player>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RegisterPlayer<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(has_one = user)]
    pub player_info: Account<'info, Player>,
    #[account(constraint = game.leaderboard == leaderboard.id)]
    pub game: Account<'info, Game>,
    #[account(has_one = game)]
    pub leaderboard: Account<'info, LeaderBoard>,
    #[account(
        init,
        payer = user,
        space = PlayerEntryList::initial_size(),
        seeds = [seeds::ENTRY, player_info.key().as_ref(), leaderboard.key().as_ref()],
        bump
    )]
    pub new_list: Account<'info, PlayerEntryList>,
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdatePlayer<'info> {
    pub user: Signer<'info>,
    #[account(mut)]
    pub player_info: Account<'info, Player>,
}

#[derive(Accounts)]
// TODO: Optionally update rank here or use a separate ix for that.
pub struct SubmitScore<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(constraint = game.check_authority_is_signer(&authority.key()))]
    pub authority: Signer<'info>,
    #[account(has_one = user)]
    pub player_info: Account<'info, Player>,
    pub game: Account<'info, Game>,
    #[account(has_one = game)]
    pub leaderboard: Account<'info, LeaderBoard>,
    #[account(has_one = player_info, has_one = leaderboard)]
    pub player_entries: Account<'info, PlayerEntryList>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MergePlayerAccounts<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(has_one = user)]
    pub player_info: Account<'info, Player>,
    /// CHECK: The [Merge] account to be initialized in handler.
    pub merge_account: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UnlockPlayerAchievement<'info> {
    pub authority: Signer<'info>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(has_one = user)]
    pub player_info: Account<'info, Player>,
    #[account(has_one = player_info, has_one = leaderboard)]
    pub player_entry: Account<'info, PlayerEntryList>,
    #[account(has_one = game)]
    pub leaderboard: Account<'info, LeaderBoard>,
    #[account(constraint = game.check_authority_is_signer(&authority.key()))]
    pub game: Account<'info, Game>,
    #[account(has_one = game)]
    pub achievement: Account<'info, Achievement>,
    #[account(
        init,
        payer = user,
        space = PlayerAchievement::SIZE,
        seeds = [seeds::PLAYER_ACHIEVEMENT, player_info.key().as_ref(), achievement.key().as_ref()],
        bump
    )]
    pub player_achievement: Account<'info, PlayerAchievement>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AddReward<'info> {
    pub authority: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(constraint = game.check_authority_is_signer(&authority.key()))]
    pub game: Account<'info, Game>,
    #[account(has_one = game)]
    pub achievement: Account<'info, Achievement>,
    #[account(
        init,
        payer = payer,
        space = Reward::SIZE,
        seeds = [seeds::REWARD, achievement.key().as_ref()],
        bump,
    )]
    pub new_reward: Account<'info, Reward>,
    pub collection_update_auth: Option<Signer<'info>>,
    pub collection_mint: Option<Account<'info, Mint>>,
    #[account(mut)]
    /// CHECK: Checked in instruction handler.
    pub collection_metadata: Option<UncheckedAccount<'info>>,
    pub system_program: Program<'info, System>,
    #[account(address = mpl_token_metadata::ID)]
    /// CHECK: We check that the ID is the correct one.
    pub token_metadata_program: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct MintReward<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub authority: Signer<'info>,
    /// CHECK: Checked in has_one relationship with `player`.
    pub user: UncheckedAccount<'info>,
    #[account(constraint = game.check_authority_is_signer(&authority.key()))]
    pub game: Box<Account<'info, Game>>,
    #[account(
        has_one = game,
        constraint = achievement.reward.unwrap() == reward.key()
    )]
    pub achievement: Box<Account<'info, Achievement>>,
    #[account(has_one = achievement)]
    pub reward: Box<Account<'info, Reward>>,
    #[account(has_one = user)]
    pub player: Box<Account<'info, Player>>,
    #[account(
        has_one = player,
        has_one = achievement,
    )]
    pub player_achievement: Box<Account<'info, PlayerAchievement>>,
    #[account(mut)]
    /// CHECK: Initialized as mint in instruction.
    pub mint: Signer<'info>,
    #[account(mut)]
    /// CHECK: Checked in metaplex program.
    pub metadata: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: Checked in metaplex program.
    pub master_edition: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: Initialized in handler as token account owned by `user`.
    pub mint_nft_to: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    #[account(address = mpl_token_metadata::ID)]
    /// CHECK: Verified program address.
    pub token_metadata_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct VerifyReward<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub authority: Signer<'info>,
    #[account(constraint = game.check_authority_is_signer(&authority.key()))]
    pub game: Box<Account<'info, Game>>,
    #[account(
        has_one = game,
        constraint = achievement.reward.unwrap() == reward.key()
    )]
    pub achievement: Box<Account<'info, Achievement>>,
    #[account(
        has_one = achievement,
        seeds = [seeds::REWARD, achievement.key().as_ref()], bump,
        constraint = reward.collection_mint == Some(collection_mint.key())
    )]
    pub reward: Box<Account<'info, Reward>>,
    /// CHECK: Checked in has_one relationship with `player`.
    pub user: UncheckedAccount<'info>,
    #[account(has_one = user)]
    pub player: Box<Account<'info, Player>>,
    #[account(
        has_one = player, has_one = achievement,
        constraint = player_achievement.metadata.unwrap() == metadata_to_verify.key()
    )]
    pub player_achievement: Box<Account<'info, PlayerAchievement>>,
    /// CHECK: We check that it's the same metadata in `player_achievement`.
    pub metadata_to_verify: UncheckedAccount<'info>,
    /// CHECK: We check that it's the reward's collection mint.
    pub collection_mint: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: Checked in CPI to Metaplex.
    pub collection_metadata: UncheckedAccount<'info>,
    /// CHECK: Checked in CPI to Metaplex.
    pub collection_master_edition: UncheckedAccount<'info>,
}
