use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use crate::helpers::Pieces;
use anchor_lang::prelude::*;


#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct GameState {
    pub piece_board: [[Pieces; 8]; 8],
    pub white_board: [[bool; 8]; 8],
    pub en_passant: u8, // Set to 64 for none
    pub white_active: bool,
    pub white_castle_king: bool,
    pub white_castle_queen: bool,
    pub black_castle_king: bool,
    pub black_castle_queen: bool,
    pub half_moves: u8, // Half moves since last pawn move or capture
}
impl Default for GameState {
    fn default() -> Self {
        Self {
            piece_board: [
                [Pieces::R, Pieces::N, Pieces::B, Pieces::Q, Pieces::K, Pieces::B, Pieces::N, Pieces::R],
                [Pieces::P; 8],
                [Pieces::Empty; 8],
                [Pieces::Empty; 8],
                [Pieces::Empty; 8],
                [Pieces::Empty; 8],
                [Pieces::P; 8],
                [Pieces::R, Pieces::N, Pieces::B, Pieces::Q, Pieces::K, Pieces::B, Pieces::N, Pieces::R]
            ],
            white_board: [
                [true; 8],
                [true; 8],
                [false; 8],
                [false; 8],
                [false; 8],
                [false; 8],
                [false; 8],
                [false; 8]
            ],
            en_passant: 64,
            white_active: true,
            white_castle_king: true,
            white_castle_queen: true,
            black_castle_king: true,
            black_castle_queen: true,
            half_moves: 0,
        }
    }
}
impl Hash for GameState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.piece_board.hash(state);
        self.white_board.hash(state);
        self.en_passant.hash(state);
        self.white_active.hash(state);
        self.white_castle_king.hash(state);
        self.white_castle_queen.hash(state);
        self.black_castle_king.hash(state);
        self.black_castle_queen.hash(state);
    }
}
impl PartialEq for GameState {
    fn eq(&self, other: &Self) -> bool {
        self.piece_board == other.piece_board &&
        self.white_board == other.white_board &&
        self.en_passant == other.en_passant &&
        self.white_active == other.white_active &&
        self.white_castle_king == other.white_castle_king &&
        self.white_castle_queen == other.white_castle_queen &&
        self.black_castle_king == other.black_castle_king &&
        self.black_castle_queen == other.black_castle_queen
    }
}
impl Eq for GameState {}

impl GameState {
    pub fn is_check(&self, white: bool) -> bool {
        // Check if white/black is in check
        
        // Find the right color king
        let mut king_rank: usize = 8;
        let mut king_col: usize = 8;
        'outer: for i in 0..8 {
            for j in 0..8 {
                // Check that location is not empty, not a king, and is the right color
                if (self.piece_board[i][j] == Pieces::K) && (self.white_board[i][j] == white) {
                    king_rank = i;
                    king_col = j;
                    break 'outer;
                }
            }
        }

        // Diagonal
        for move_up in [false,true] {
            for move_right in [false,true] {
                for i in 1..8 {
                    if !in_board(king_rank,king_col,i,move_up,i,move_right) {
                        break;
                    }
                    let new_rank = update_loc(king_rank,i,move_up);
                    let new_col = update_loc(king_col,i,move_right);
                    if self.piece_board[new_rank][new_col] != Pieces::Empty {
                        if self.white_board[new_rank][new_col] == white {
                            break;
                        } else {
                            match self.piece_board[new_rank][new_col] {
                                Pieces::B|Pieces::Q|Pieces::K => return true,
                                Pieces::P => {
                                    if (i==1) && (move_up != white) {
                                        return true;
                                    } else {
                                        break;
                                    }
                                }
                                _ => break,
                            }
                        }
                    }
                }
            }
        }
        // Straight-line
        for move_up_down in [false,true] {
            for move_pos in [false,true] {
                for i in 1..8 {
                    if (move_up_down && !in_board(king_rank,king_col,i,move_pos,0,true)) || 
                        (!move_up_down && !in_board(king_rank,king_col,0,true,i,move_pos)) {
                        break;
                    }
                    let mut new_rank = king_rank;
                    let mut new_col = king_col;
                    if move_up_down {
                        new_rank = update_loc(king_rank,i,move_pos);
                    } else {
                        new_col = update_loc(king_col,i,move_pos);
                    }
                    if self.piece_board[new_rank][new_col] != Pieces::Empty {
                        if self.white_board[new_rank][new_col] == white {
                            break;
                        } else {
                            match self.piece_board[new_rank][new_col] {
                                Pieces::R|Pieces::Q|Pieces::K => return true,
                                _ => break,
                            }
                        }
                    }
                }
            }
        }
        // Knight moves
        for rank_change in [1,2] {
            for rank_pos in [false,true] {
                for col_pos in [false,true] {
                    let col_change = rank_change % 2 + 1;
                    if !in_board(king_rank,king_col,rank_change,rank_pos,col_change,col_pos) {
                        break;
                    }
                    let new_rank = update_loc(king_rank,rank_change,rank_pos);
                    let new_col = update_loc(king_col,col_change,col_pos);
                    if (self.piece_board[new_rank][new_col] == Pieces::N) && (self.white_board[new_rank][new_col] != white) {
                        return true;
                    }
                }
            }
        }
        return false;
    }
    
    pub fn has_valid_move(&mut self) -> bool {    
        // For non-active color, see if any piece has a valid move
        for i in 0..8 {
            for j in 0..8 {
                // Check to see if piece can move
                let piece = self.piece_board[i][j];
                if piece != Pieces::Empty && self.white_board[i][j] != self.white_active {
                    let acts = match piece {
                        Pieces::R => vec![(1,0,true,true),(1,0,false,true),(0,1,true,true),(0,1,true,false)],
                        Pieces::N => vec![
                            (2,1,true,true),(2,1,true,false),(2,1,false,true),(2,1,false,false),
                            (1,2,true,true),(1,2,true,false),(1,2,false,true),(1,2,false,false)
                        ],
                        Pieces::B => vec![(1,1,true,true),(1,1,true,false),(1,1,false,true),(1,1,false,false)],
                        Pieces::Q|Pieces::K => vec![
                            (1,0,true,true),(1,0,false,true),(0,1,true,true),(0,1,true,false),
                            (1,1,true,true),(1,1,true,false),(1,1,false,true),(1,1,false,false)
                        ],
                        _ => vec![
                            (1,0,!self.white_active,true),(1,1,!self.white_active,true),
                            (1,1,!self.white_active,false)
                        ],
                    };
                    for act in acts {
                        if in_board(i,j,act.0,act.2,act.1,act.3) {
                            let test_rank = update_loc(i,act.0,act.2);
                            let test_col = update_loc(j,act.1,act.3);
                            if piece == Pieces::P {
                                if act.1 == 0 && self.piece_board[test_rank][test_col] != Pieces::Empty {
                                    continue;
                                } else if (
                                    self.piece_board[test_rank][test_col] == Pieces::Empty || 
                                    self.white_board[test_rank][test_col] == self.white_active
                                ) && test_rank*8+test_col != usize::from(self.en_passant) {
                                    continue;
                                }
                            } else if self.piece_board[test_rank][test_col] != Pieces::Empty &&
                                self.white_board[test_rank][test_col] != self.white_active {
                                continue;
                            }
                            // Move and check
                            let old_piece = self.piece_board[test_rank][test_col];
                            let old_color = self.white_board[test_rank][test_col];
                            self.piece_board[test_rank][test_col] = piece;
                            self.white_board[test_rank][test_col] = !self.white_active;
                            self.piece_board[i][j] = Pieces::Empty;
                            let is_check = self.is_check(!self.white_active);
                            self.piece_board[i][j] = piece;
                            self.white_board[test_rank][test_col] = old_color;
                            self.piece_board[test_rank][test_col] = old_piece;
                            if !is_check {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        return false;
    }

    pub fn is_insufficient_mat(&self) -> bool {
        // K v k
        // KB v k
        // KN v k
        // KB v kb (bishops are the same color)
        let mut white_light_bishop = false;
        let mut white_dark_bishop = false;
        let mut white_knight = false;
        let mut black_light_bishop = false;
        let mut black_dark_bishop = false;
        let mut black_knight = false;
        for i in 0..8 {
            for j in 0..8 {
                if self.piece_board[i][j] == Pieces::N {
                    if self.white_board[i][j] {
                        white_knight = true;
                    } else {
                        black_knight = true;
                    }
                } else if self.piece_board[i][j] == Pieces::B {
                    if (i+j) % 2 == 0 {
                        if self.white_board[i][j] {
                            white_dark_bishop = true;
                        } else {
                            black_dark_bishop = true;
                        }
                    } else {
                        if self.white_board[i][j] {
                            white_light_bishop = true;
                        } else {
                            black_light_bishop = true;
                        }
                    }
                } else if (self.piece_board[i][j] != Pieces::Empty) && 
                    (self.piece_board[i][j] != Pieces::K) { // Not king, bishop, or knight
                    return false;
                }
            }
        }
        // Knight and other piece
        if white_knight && (white_light_bishop || white_dark_bishop || 
            black_knight ||black_light_bishop || black_dark_bishop) {
            return false;
        }
        if black_knight && (white_knight || white_light_bishop || white_dark_bishop || 
            black_light_bishop || black_dark_bishop) {
            return false;
        }
        // Mismatched bishops
        if (white_light_bishop && black_dark_bishop) || (white_dark_bishop && black_light_bishop) {
            return false;
        }

        return true;
    }

    pub fn only_king(&self, is_white: bool) -> bool {
        // If a player flags, their opponent wins unless player only has a king on the board 
        // or above conds are met
        for i in 0..8 {
            for j in 0..8 {
                // Check that location is not empty, not a king, and is the right color
                if (self.piece_board[i][j] != Pieces::Empty) && 
                    (self.piece_board[i][j] != Pieces::K) && 
                    (is_white == self.white_board[i][j]) {
                    return false;
                }
            }
        }
        return true;
    }

    pub fn small_hash(&self) -> u32 {
        let mut s = DefaultHasher::new();
        self.hash(&mut s);
        (s.finish() >> 32).try_into().unwrap()
    }
}

fn in_board(rank:usize,col:usize,rank_change:usize,rank_pos:bool,col_change:usize,col_pos:bool) -> bool {
    if (rank_pos && (rank + rank_change > 7)) || (!rank_pos && (rank_change > rank)) {
        return false;
    }
    if (col_pos && (col + col_change > 7)) || (!col_pos && (col_change > col)) {
        return false;
    }
    return true;
}

pub fn update_loc(loc:usize,change:usize,pos:bool) -> usize {
    if pos {
        loc + change
    } else {
        loc - change
    }
}