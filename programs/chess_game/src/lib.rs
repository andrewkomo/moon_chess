use anchor_lang::prelude::*;
declare_id!("2G1pVxGS4p9jFFTVBVirZ2vdQMw22nKfbf6CupVEVZSg");

mod code_generator;
mod game_state;
use game_state::{GameState};
use code_generator::{GameCodes};
use code_generator::{active_game_code,timeout_game_code};

mod helpers;


#[program]
pub mod chess_game {
    use super::*;
    pub fn setup_game(ctx: Context<SetupGame>, white_player: Pubkey, black_player: Pubkey, white_time: i64, black_time: i64, white_bonus: u32, black_bonus: u32) -> Result<()> {
        let game = &mut *ctx.accounts.game;
        game.authority = ctx.accounts.authority.key();
        game.white_player = white_player;
        game.black_player = black_player;
        game.white_time_left = white_time;
        game.black_time_left = black_time;
        game.white_bonus_time = white_bonus;
        game.black_bonus_time = black_bonus;
        game.curr_board = GameState::default();
        game.past_states[0] = game.curr_board.small_hash();
        game.last_move = Clock::get().unwrap().unix_timestamp;
        msg!("{}",game.curr_board.white_active);
        Ok(())
    }
    pub fn play(ctx: Context<Play>, turn: u16) -> Result<()> {
        let game = &mut *ctx.accounts.game;    
        game.play(turn)
    }
    pub fn update_draw(ctx: Context<UpdateDraw>, is_white: bool, is_draw: bool) -> Result<()> {
        let game = &mut *ctx.accounts.game;
        game.update_draw(is_white, is_draw)
    }
    pub fn resign(ctx: Context<Resign>, is_white: bool) -> Result<()> {
        let game = &mut *ctx.accounts.game;
        game.resign(is_white)
    }
    pub fn claim_timeout(ctx: Context<Timeout>) -> Result<()> {
        let game = &mut *ctx.accounts.game;
        game.claim_timeout()
    }
}


#[account]
pub struct Game {
    authority: Pubkey,             // 32
    white_player: Pubkey,          // 32
    black_player: Pubkey,          // 32
    past_states: [u32; 256],       // 32*256 = 8192
    curr_board: GameState,         // ~560
    num_moves: u16, // half-moves  // 16
    status: GameCodes,             // 4
    white_draw_open: bool,         // 1
    black_draw_open: bool,         // 1
    white_time_left: i64, // sec   // 64
    black_time_left: i64, // sec   // 64
    white_bonus_time: u32, // sec  // 32
    black_bonus_time: u32, // sec  // 32
    last_move: i64, // sec         // 64
}
impl Default for Game {
    fn default() -> Self {
        Self {
            authority: Default::default(),
            white_player: Default::default(),
            black_player: Default::default(),
            past_states: [0; 256],
            curr_board: Default::default(),
            num_moves: 0,
            status: GameCodes::Active,
            white_draw_open: false,
            black_draw_open: false,
            white_time_left: 0,
            black_time_left: 0,
            white_bonus_time: 0,
            black_bonus_time: 0,
            last_move: 0,
        }
    }
}
impl Game {
    fn is_active(&self) -> bool {
        self.status == GameCodes::Active
    }
    fn is_timeout(&self, curr_time: i64) -> bool {
        let is_white: bool = self.num_moves % 2 == 0;
        let time_diff = curr_time - self.last_move;
        (is_white && time_diff > self.white_time_left.into()) || 
        (!is_white && time_diff > self.black_time_left.into())
    }
    fn resign(&mut self, is_white: bool) -> Result<()> {
        if !self.is_active() {
            return err!(ChessError::GameAlreadyOver);
        }
        if is_white {
            self.status = GameCodes::BlackWinResignation;
        } else {
            self.status = GameCodes::WhiteWinResignation;
        }
        Ok(())
    }
    fn update_draw(&mut self, is_white: bool, is_draw: bool) -> Result<()> {
        if !self.is_active() {
            return err!(ChessError::GameAlreadyOver);
        }
        if is_white {
            self.white_draw_open = is_draw;
        } else {
            self.black_draw_open = is_draw;
        }
        if self.white_draw_open && self.black_draw_open {
            self.status = GameCodes::DrawAgreement;
        }
        Ok(())
    }
    fn play(&mut self, turn: u16) -> Result<()> {
        if !self.is_active() {
            return err!(ChessError::GameAlreadyOver);
        }
        let is_white: bool = self.num_moves % 2 == 0;
        let curr_time = Clock::get().unwrap().unix_timestamp;
        let time_diff = curr_time - self.last_move;
        let game_code: GameCodes;
        if self.is_timeout(curr_time) {
            game_code = timeout_game_code(&self.curr_board);
        } else {
            self.num_moves += 1;
            let num_moves: usize = self.num_moves.into();
            if num_moves >= self.past_states.len()-1 {
                self.status = GameCodes::DrawMaxMoves;
                return Ok(());
            }
            msg!("Turn #{}: {}",num_moves,turn);
            game_code = active_game_code(&mut self.curr_board, turn, &mut self.past_states, num_moves);
        }
        if game_code == GameCodes::Invalid {
            return err!(ChessError::InvalidMove);
        } else {
            self.status = game_code;
        }
        if is_white {
            self.white_time_left = self.white_time_left - time_diff + i64::from(self.white_bonus_time);
        } else {
            self.black_time_left = self.black_time_left - time_diff + i64::from(self.black_bonus_time);
        }
        Ok(())
    }
    fn claim_timeout(&mut self) -> Result<()> {
        let curr_time = Clock::get().unwrap().unix_timestamp;
        if self.is_timeout(curr_time) {
            let game_code = timeout_game_code(&self.curr_board);
            self.status = game_code;
        }
        Ok(())
    }
}

#[derive(Accounts)]
pub struct SetupGame<'info> {
    #[account(init, payer = authority)]
    pub game: Box<Account<'info, Game>>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>
}


#[derive(Accounts)]
pub struct Play<'info> {
    #[account(mut, has_one = authority)]
    pub game: Box<Account<'info, Game>>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateDraw<'info> {
    #[account(mut, has_one = authority)]
    pub game: Box<Account<'info, Game>>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct Resign<'info> {
    #[account(mut, has_one = authority)]
    pub game: Box<Account<'info, Game>>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct Timeout<'info> {
    #[account(mut)]
    pub game: Box<Account<'info, Game>>,
    pub reporter: Signer<'info>,
}

#[error_code]
pub enum ChessError {
    InvalidMove,
    GameAlreadyOver,
}