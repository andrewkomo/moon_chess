use std::cmp;
use crate::game_state::{update_loc,GameState};
use crate::helpers::{Pieces,Turn};
use anchor_lang::prelude::*;

const MAX_HALFMOVES: u8 = 100; // 50 move rule
// pub const MAX_MOVES: usize = 64*2;

fn try_update_board(turn: &Turn, curr_game: &mut GameState) -> bool {
    // Checks if the move is valid and updates the board

    // Recast rank and col as usize (needed for index)
    let piece = turn.piece();
    let from_rank: usize = turn.from_rank();
    let from_col: usize = turn.from_col();
    let to_rank: usize = turn.to_rank();
    let to_col: usize = turn.to_col();

    // Check if piece at original location is right
    if (curr_game.piece_board[from_rank][from_col] != piece) || 
        (curr_game.white_active != curr_game.white_board[from_rank][from_col]) {
        return false;
    }

    // Check no same color piece at final location
    if (curr_game.piece_board[to_rank][to_col] != Pieces::Empty) && 
        (curr_game.white_active == curr_game.white_board[to_rank][to_col]) {
        return false;
    }

    // Check if piece can move to target square
    if !turn.is_valid_dir() {
        return false;
    }
    let rank_diff = cmp::max(from_rank,to_rank) - cmp::min(from_rank,to_rank);
    let col_diff = cmp::max(from_col,to_col) - cmp::min(from_col,to_col);
    if piece.is_pawn()  {
        // Check movement is valid
        if (curr_game.white_active && to_rank < from_rank) || (!curr_game.white_active && to_rank > from_rank) {
            return false;
        }
        if col_diff == 0 { // Going forward
            if rank_diff == 2 && ((curr_game.white_active && to_rank != 3) || (!curr_game.white_active && to_rank != 4)) {
                return false;
            }
            // Check the movement squares
            for i in 1..=rank_diff {
                let new_rank = update_loc(from_rank,i,curr_game.white_active);
                if curr_game.piece_board[new_rank][from_col] != Pieces::Empty {
                    return false;
                }
            }
            // Update board
            default_update(turn, curr_game);
            if rank_diff == 2 {
                curr_game.en_passant = {
                    if curr_game.white_active {
                        ((from_rank+1)*8+from_col).try_into().unwrap()
                    } else {
                        ((from_rank-1)*8+from_col).try_into().unwrap()
                    }
                };
            }
        } else { // Capture/en passant
            if (curr_game.piece_board[to_rank][to_col] != Pieces::Empty) && 
                (to_rank*8+to_col != usize::from(curr_game.en_passant)) {
                return false;
            }
            // Update board
            if to_rank*8+to_col == usize::from(curr_game.en_passant) {
                if curr_game.white_active {
                    curr_game.piece_board[to_rank-1][to_col] = Pieces::Empty;
                } else {
                    curr_game.piece_board[to_rank+1][to_col] = Pieces::Empty;
                    curr_game.white_board[to_rank+1][to_col] = false;
                }
            }
            default_update(turn, curr_game);
        }

        // Handle promotions separately
        if (curr_game.white_active && (to_rank == 7)) || (!curr_game.white_active && (to_rank == 0)) {
            if piece == Pieces::P {
                return false;
            }
            // Update board
            curr_game.piece_board[to_rank][to_col] = {
                match piece {
                    Pieces::PToR => Pieces::R,
                    Pieces::PToN => Pieces::N,
                    Pieces::PToB => Pieces::B,
                    Pieces::PToQ => Pieces::Q,
                    _ => return false,
                }
            };
        }
        else if piece != Pieces::P {
            return false;
        }
    } else if piece == Pieces::K {
        if col_diff == 2 { // Handle castling separately (by moving the king square by square)
            let mut end_rook_loc = from_col+1;
            let mut start_rook_loc = 7;
            if to_col == 6 { // King-side
                if (curr_game.white_active && !curr_game.white_castle_king) || (!curr_game.white_active && !curr_game.black_castle_king) {
                    return false;
                }
                for i in 1..=2 {
                    if curr_game.piece_board[from_rank][from_col+i] != Pieces::Empty {
                        return false;
                    }
                }
            } else { // Queen-side
                if (curr_game.white_active && !curr_game.white_castle_queen) || (!curr_game.white_active && !curr_game.black_castle_queen) {
                    return false;
                }
                for i in 1..=3 {
                    if curr_game.piece_board[from_rank][from_col-i] != Pieces::Empty {
                        return false;
                    }
                }
                end_rook_loc = from_col-1;
                start_rook_loc = 0;
            }
            // Check the king does not move through check
            curr_game.piece_board[from_rank][end_rook_loc] = Pieces::K;
            curr_game.white_board[from_rank][end_rook_loc] = curr_game.white_active;
            if curr_game.is_check(curr_game.white_active) {
                return false;
            }
            default_update(turn,curr_game);
            // Move the rook
            curr_game.piece_board[from_rank][end_rook_loc] = Pieces::R;
            curr_game.piece_board[from_rank][start_rook_loc] = Pieces::Empty;
            curr_game.white_board[from_rank][start_rook_loc] = false;
        } else { // Otherwise move king normally
            default_update(turn,curr_game);
        }
    } else {
        let rank_move: usize = {
            if piece == Pieces::N || rank_diff == 0 {
                rank_diff
            } else {
                1
            }
        };
        let rank_move_pos: bool = from_rank < to_rank;
        let col_move: usize = {
            if piece == Pieces::N || col_diff == 0 {
                col_diff
            } else {
                1
            }
        };
        let col_move_pos: bool = from_col < to_col;
        let dist: usize = {
            if piece == Pieces::N {
                1
            } else {
                cmp::max(rank_diff,col_diff)
            }
        };
        for i in 1..dist {
            let new_rank: usize = {
                if rank_move_pos {
                    from_rank+i*rank_move
                }
                else {
                    from_rank-i*rank_move
                }
            };
            let new_col: usize = {
                if col_move_pos {
                    from_col + i*col_move
                } else {
                    from_col - i*col_move
                }
            };
            if curr_game.piece_board[new_rank][new_col] != Pieces::Empty {
                return false;
            }
        }
        // Update board
        default_update(turn,curr_game);
    }

    // Check if check
    return !curr_game.is_check(curr_game.white_active);
}

fn default_update(turn: &Turn, curr_game: &mut GameState) -> () {
    let from_rank: usize = turn.from_rank();
    let from_col: usize = turn.from_col();
    let to_rank: usize = turn.to_rank();
    let to_col: usize = turn.to_col();

    // Update move counters
    if turn.piece().is_pawn() || (curr_game.piece_board[to_rank][to_col] != Pieces::Empty) {
        curr_game.half_moves = 0;
    } else {
        curr_game.half_moves += 1;
    }

    curr_game.piece_board[from_rank][from_col] = Pieces::Empty;
    curr_game.white_board[from_rank][from_col] = false;
    curr_game.piece_board[to_rank][to_col] = turn.piece();
    curr_game.white_board[to_rank][to_col] = curr_game.white_active;
    curr_game.en_passant = 64;

    // Update castling rights
    if ((from_col == 0 || from_col == 4) && (from_rank == 0)) || ((to_col == 0) && (to_rank == 0)) {
        curr_game.white_castle_queen = false;
    }
    if ((from_col == 7 || from_col == 4) && (from_rank == 0)) || ((to_col == 7) && (to_rank == 0)) {
        curr_game.white_castle_king = false;
    }
    if ((from_col == 0 || from_col == 4) && (from_rank == 7)) || ((to_col == 0) && (to_rank == 7)) {
        curr_game.black_castle_queen = false;
    }
    if ((from_col == 7 || from_col == 4) && (from_rank == 7)) || ((to_col == 7) && (to_rank == 7)) {
        curr_game.black_castle_king = false;
    }
}

#[derive(PartialEq, Eq, AnchorSerialize, AnchorDeserialize, Copy, Clone)]
pub enum GameCodes {
    Active,
    Invalid,
    WhiteWinCheckmate,
    WhiteWinResignation,
    WhiteWinTime,
    BlackWinCheckmate,
    BlackWinResignation,
    BlackWinTime,
    DrawStalemate,
    DrawInsufficientMaterial,
    DrawFiftyMoves,
    DrawAgreement,
    DrawMaxMoves,
    DrawRepetition,
}
impl Default for GameCodes {
    fn default() -> Self { GameCodes::Active }
}
// impl GameCodes {
//     fn is_draw(&self) -> bool {
//         match self {
//             Self::DrawStalemate|Self::DrawInsufficientMaterial|Self::DrawFiftyMoves|Self::DrawAgreement|Self::DrawMaxMoves => true,
//             _ => false
//         }
//     }
//     fn is_white_winner(&self) -> bool {
//         match self {
//             Self::WhiteWinCheckmate|Self::WhiteWinResignation|Self::WhiteWinTime => true,
//             _ => false
//         }
//     }
//     fn is_black_winner(&self) -> bool {
//         match self {
//             Self::BlackWinCheckmate|Self::BlackWinResignation|Self::BlackWinTime => true,
//             _ => false
//         }
//     }
// }

pub fn active_game_code(game_state: &mut GameState, turn: u16, 
    past_states: &mut [u32; 128], num_moves: usize) -> GameCodes {
    if !try_update_board(&Turn {turn: turn},game_state) {
        return GameCodes::Invalid;
    }
    
    if !game_state.has_valid_move() {
        msg!("Here");
        if game_state.is_check(!game_state.white_active) {
            if game_state.white_active {
                return GameCodes::WhiteWinCheckmate;
            } else {
                return GameCodes::BlackWinCheckmate;
            }
        } else {
            return GameCodes::DrawStalemate;
        }
    }
    if game_state.is_insufficient_mat() {
        return GameCodes::DrawInsufficientMaterial;
    }
    if game_state.half_moves >= MAX_HALFMOVES {
        return GameCodes::DrawFiftyMoves;
    }

    game_state.white_active = !game_state.white_active;

    let hash = game_state.small_hash();
    let mut count = 0;
    for i in 0..num_moves {
        if hash == past_states[i] {
            count += 1;
        }
    }
    if count >= 2 {
        return GameCodes::DrawRepetition;
    } else {
        past_states[num_moves] = hash;
    }

    return GameCodes::Active;
}
pub fn timeout_game_code(game_state: &GameState) -> GameCodes {
    if game_state.only_king(!game_state.white_active) {
        return GameCodes::DrawInsufficientMaterial;
    } else {
        if game_state.white_active {
            return GameCodes::BlackWinTime;
        } else {
            return GameCodes::WhiteWinTime;
        }
    }
}

// fn main() {
//     let default_turn = Turn {
//         piece: Pieces::Empty,
//         from_rank: 0,
//         from_col: 0,
//         to_rank: 0,
//         to_col: 0,
//     };
//     let first_turn = Turn {
//         piece: Pieces::P,
//         from_rank: 1,
//         from_col: 2,
//         to_rank: 3,
//         to_col: 2,
//     };
//     let mut turns = [&default_turn; MAX_MOVES];
//     turns[0] = &first_turn;
//     println!("Here");
//     let code = generate_game_code(turns, 1);
//     match code {
//         GameCodes::Valid => println!("Valid"),
//         GameCodes::Invalid => println!("Invalid"),
//         _ => println!("Sad"),
//     };
// }
