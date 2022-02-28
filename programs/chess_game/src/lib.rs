use anchor_lang::prelude::*;
declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

mod code_generator;
mod game_state;
use code_generator::{MAX_MOVES,GameCodes};
use code_generator::generate_game_code;

mod helpers;


#[program]
pub mod chess_game {
    use super::*;
    pub fn setup_game(ctx: Context<SetupGame>, black_player: Pubkey, white_time: i64, black_time: i64, white_bonus: u32, black_bonus: u32) -> Result<()> {
        let game = &mut ctx.accounts.game;
        game.white_player = ctx.accounts.white_player.key();
        game.black_player = black_player;
        game.white_time_left = white_time;
        game.black_time_left = black_time;
        game.white_bonus_time = white_bonus;
        game.black_bonus_time = black_bonus;
        game.last_move  = match Clock::get() {
            Ok(clock) => clock.unix_timestamp,
            Err(_e) => return Err(error!(ChessError::ClockError))
        };
        Ok(())
    }
    pub fn play(ctx: Context<Play>, turn: u16) -> Result<()> {
        let game = &mut ctx.accounts.game;
    
        require!(
            game.current_player() == ctx.accounts.player.key(),
            ChessError::NotPlayersTurn
        );
    
        game.play(turn)
    }
    pub fn update_draw(ctx: Context<UpdateDraw>, is_draw: bool) -> Result<()> {
        let game = &mut ctx.accounts.game;
    
        require!(
            game.is_player(ctx.accounts.player.key()),
            ChessError::NotValidPlayer
        );
    
        game.update_draw(ctx.accounts.player.key(), is_draw)
    }
    pub fn resign(ctx: Context<Resign>) -> Result<()> {
        let game = &mut ctx.accounts.game;
    
        require!(
            game.is_player(ctx.accounts.player.key()),
            ChessError::NotValidPlayer
        );
    
        game.resign(ctx.accounts.player.key())
    }
    pub fn claim_timeout(ctx: Context<Timeout>) -> Result<()> {
        ctx.accounts.game.claim_timeout()
    }
}


#[account]
pub struct Game {
    white_player: Pubkey,          // 32
    black_player: Pubkey,          // 32
    turns: [u16; 256],             // 16*512 = 8192
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
    fn default() -> Game {
        Game {
            white_player: Default::default(),
            black_player: Default::default(),
            turns: [Default::default(); MAX_MOVES],
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
    fn current_player(&self) -> Pubkey {
        if self.num_moves % 2 == 0 {
            self.white_player
        } else {
            self.black_player
        }
    }
    fn is_player(&self, player: Pubkey) -> bool {
        (player == self.white_player) || (player == self.black_player)
    }
    fn resign(&mut self, player: Pubkey) -> Result<()> {
        if !self.is_active() {
            return err!(ChessError::GameAlreadyOver);
        }
        if player == self.white_player {
            self.status = GameCodes::BlackWinResignation;
        } else {
            self.status = GameCodes::WhiteWinResignation;
        }
        Ok(())
    }
    fn update_draw(&mut self, player: Pubkey, is_draw: bool) -> Result<()> {
        if !self.is_active() {
            return err!(ChessError::GameAlreadyOver);
        }
        if player == self.white_player {
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
        let curr_time = match Clock::get() {
            Ok(clock) => clock.unix_timestamp,
            Err(_e) => return Err(error!(ChessError::ClockError))
        };
        let time_diff = curr_time - self.last_move;
        let game_code: GameCodes;
        if self.is_timeout(curr_time) {
            game_code = generate_game_code(&self.turns,self.num_moves.into(),true);
        } else {
            self.num_moves += 1;
            let num_moves: usize = self.num_moves.into();
            if num_moves >= MAX_MOVES {
                self.status = GameCodes::DrawMaxMoves;
                return Ok(());
            }
            self.turns[num_moves] = turn;
            game_code = generate_game_code(&self.turns,num_moves,false);
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
        let curr_time = match Clock::get() {
            Ok(clock) => clock.unix_timestamp,
            Err(_e) => return Err(error!(ChessError::ClockError))
        };
        if self.is_timeout(curr_time) {
            let game_code = generate_game_code(&self.turns,self.num_moves.into(),true);
            self.status = game_code;
        }
        Ok(())
    }
}

#[derive(Accounts)]
pub struct SetupGame<'info> {
    #[account(init, payer = white_player)]
    pub game: Account<'info, Game>,
    #[account(mut)]
    pub white_player: Signer<'info>,
    pub system_program: Program<'info, System>
}


#[derive(Accounts)]
pub struct Play<'info> {
    #[account(mut)]
    pub game: Account<'info, Game>,
    pub player: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateDraw<'info> {
    #[account(mut)]
    pub game: Account<'info, Game>,
    pub player: Signer<'info>,
}

#[derive(Accounts)]
pub struct Resign<'info> {
    #[account(mut)]
    pub game: Account<'info, Game>,
    pub player: Signer<'info>,
}

#[derive(Accounts)]
pub struct Timeout<'info> {
    #[account(mut)]
    pub game: Account<'info, Game>,
    pub reporter: Signer<'info>,
}

#[error_code]
pub enum ChessError {
    InvalidMove,
    GameAlreadyOver,
    NotPlayersTurn,
    NotValidPlayer,
    ClockError,
}