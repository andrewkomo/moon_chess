use std::hash::{Hash, Hasher};
use std::cmp;

#[derive(PartialEq, Hash, Clone, Copy)]
enum Pieces {
    Empty,
    R,
    N,
    B,
    Q,
    K,
    P,
    PToR,
    PToN,
    PToB,
    PToQ,
}
impl Pieces {
    fn is_pawn(&self) -> bool {
        match self {
            Self::P | Self::PToR | Self::PToN | Self::PToB | Self::PToQ => true,
            _ => false,
        }
    }
}

struct GameState {
    piece_board: [[Pieces; 8]; 8], //(R,N,B,Q,K,P) = (1,2,3,4,5,10)
    white_board: [[bool; 8]; 8], // Which pieces are occupied by white pieces
    en_passant: (u8, u8), // Set to (8,8) for none
    white_active: bool,
    white_castle_king: bool,
    white_castle_queen: bool,
    black_castle_king: bool,
    black_castle_queen: bool,
    half_moves: u16, // Half moves since last pawn move or capture
    full_moves: u16,
}

impl Hash for GameState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.piece_board.hash(state);
        self.white_board.hash(state);
        self.en_passant.hash(state);
    }
}

impl GameState {
    fn is_checkmate(&self) -> bool {
        // Check if the non-active color is in checkmate
        // Assume check
        true
    }

    fn is_check(&self, white: bool) -> bool {
        // Check if white/black is in check
        true
    }
    
    fn is_stalemate(&self) -> bool {
        // Assume not check
    
        // For each piece see if there is a valid move
        true
    }

    fn is_insufficient_mat(&self) -> bool {
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
        let mut other_piece = false; // Not king, bishop, or knight
        for i in 0..8 {
            for j in 0..8 {
                if self.piece_board[i][j] == Pieces::N { // knights
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
                    (self.piece_board[i][j] != Pieces::K) {
                    other_piece = true;
                }
            }
        }
    
        if other_piece {
            return false
        }
        if white_knight {
            if white_light_bishop || white_dark_bishop || 
                black_knight ||black_light_bishop || black_dark_bishop {
                return false;
            }
        }
        if black_knight {
            if white_knight || white_light_bishop || white_dark_bishop || 
                black_light_bishop || black_dark_bishop {
                return false;
            }
        }
        if white_light_bishop && black_dark_bishop {
            return false;
        }
        if white_dark_bishop && black_light_bishop {
            return false;
        }
        return true;
    }

    fn only_king(&self, is_white: bool) -> bool {
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

}

struct Turn {
    piece: Pieces, 
    from_rank: u8,
    from_col: u8,
    to_rank: u8,
    to_col: u8,
}

const MAX_MOVES: usize = 500;
const MAX_HALFMOVES: usize = 100; // 50 move rule

fn in_board(rank: u8, col: u8) -> bool {
    (rank < 8) && (col < 8)
}

fn is_valid(turn: &Turn, curr_game: &mut GameState) -> bool { // I think I'm doing the mut wrong
    // Checks if the move is valid and updates the board
    // Check if both coordinates are valid
    if !(in_board(turn.from_rank,turn.from_col) && in_board(turn.to_rank,turn.to_col)) {
        return false;
    }

    // Check the piece moves
    if (turn.from_rank == turn.to_rank) && (turn.from_col == turn.to_col) {
        return false;
    }

    // Check if piece at original location is right
    let from_rank: usize = turn.from_rank.into();
    let from_col: usize = turn.from_col.into();
    if (curr_game.piece_board[from_rank][from_col] != turn.piece) || 
        (curr_game.white_active != curr_game.white_board[from_rank][from_col]) {
        return false;
    }

    // Check no same color piece at final location
    let to_rank: usize = turn.to_rank.into();
    let to_col: usize = turn.to_col.into();
    if (curr_game.piece_board[to_rank][to_col] != Pieces::Empty) && 
        (curr_game.white_active == curr_game.white_board[to_rank][to_col]) {
        return false;
    }

    // Update move counters
    if turn.piece.is_pawn() || (curr_game.piece_board[to_rank][to_col] != Pieces::Empty) {
        curr_game.half_moves = 0;
    } else {
        curr_game.half_moves += 1;
    }
    if !curr_game.white_active {
        curr_game.full_moves += 1;
    }

    // Update castling rights
    if ((turn.from_col == 0) && (turn.from_rank == 0)) ||
        ((turn.to_col == 0) && (turn.to_rank == 0)) {
        curr_game.white_castle_queen = false;
    }
    if ((turn.from_col == 7) && (turn.from_rank == 0)) ||
        ((turn.to_col == 7) && (turn.to_rank == 0)) {
        curr_game.white_castle_king = false;
    }
    if ((turn.from_col == 0) && (turn.from_rank == 7)) ||
        ((turn.to_col == 0) && (turn.to_rank == 7)) {
        curr_game.black_castle_queen = false;
    }
    if ((turn.from_col == 7) && (turn.from_rank == 7)) ||
        ((turn.to_col == 7) && (turn.to_rank == 7)) {
        curr_game.black_castle_king = false;
    }

    // Check if piece can move to target square
    if turn.piece.is_pawn()  { // Pawns first
        // First check if valid promotion
        // Then check forward movement
        // Then captures/en passant
    } else if turn.piece == Pieces::K { // King
        // Handle castling separately (by moving the king square by square)

        // Update castling rights
        if curr_game.white_active {
            curr_game.white_castle_king = false;
            curr_game.white_castle_queen = false;
        } else {
            curr_game.black_castle_king = false;
            curr_game.black_castle_queen = false;
        }
    } else {
        let rank_diff = cmp::max(turn.from_rank,turn.to_rank) - cmp::min(turn.from_rank,turn.to_rank);
        let col_diff = cmp::max(turn.from_col,turn.to_col) - cmp::min(turn.from_col,turn.to_col);
        let rank_move: usize = {
            if turn.piece == Pieces::N || rank_diff == 0 {
                rank_diff.into()
            } else {
                1
            }
        };
        let rank_move_pos: bool = turn.from_rank < turn.to_rank;
        let col_move: usize = {
            if turn.piece == Pieces::N || col_diff == 0 {
                col_diff.into()
            } else {
                1
            }
        };
        let col_move_pos: bool = turn.from_col < turn.to_col;
        let dist: usize = {
            if turn.piece == Pieces::N {
                1
            } else {
                cmp::max(rank_diff,col_diff).into()
            }
        };
        if turn.piece == Pieces::R && (rank_diff != 0) && (col_diff != 0) {
            return false;
        }
        if turn.piece == Pieces::N && 
            (rank_diff != 2 || col_diff != 1) && (rank_diff != 1 || col_diff != 2) {
            return false;
        }
        if turn.piece == Pieces::B && rank_diff != col_diff {
            return false;
        }
        if turn.piece == Pieces::Q && 
            (rank_diff != col_diff) && (rank_diff != 0) && (col_diff != 0) {
            return false;
        }
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
        curr_game.piece_board[from_rank][from_col] = Pieces::Empty;
        curr_game.white_board[from_rank][from_col] = false;
        curr_game.piece_board[to_rank][to_col] = turn.piece;
        curr_game.white_board[to_rank][to_col] = curr_game.white_active;
    }

    // Check if check
    return !curr_game.is_check(curr_game.white_active);
}

fn is_capture(turn: Turn, curr_game: GameState) -> bool {
    // Check if the move is a capture for 50 move rule
    // Assume that the move is valid
    let to_rank: usize = turn.to_rank.into();
    let to_col: usize = turn.to_col.into();
    curr_game.piece_board[to_rank][to_col] != Pieces::Empty
}

enum GameCodes {
    Invalid,
    Valid,
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
}

// const INVALID = 0;
// const VALID = 1;
// const WHITE_WIN_CHECKMATE = 10;
// const WHITE_WIN_RESIGNATION = 11;
// const WHITE_WIN_TIME = 12;
// const BLACK_WIN_CHECKMATE = 20;
// const BLACK_WIN_RESIGNATION = 21;
// const BLACK_WIN_TIME = 22;
// const DRAW_STALEMATE = 30;
// const DRAW_INSUFFICIENT_MATERIAL = 31;
// const DRAW_THREEFOLD_REPITITION = 32;
// const DRAW_FIFTY_MOVES = 33;
// const DRAW_AGREEMENT = 34;
// const DRAW_MAX_MOVES = 35;

fn generate_game_code(turns: [&Turn; MAX_MOVES*2], half_moves: usize) -> GameCodes {
    // Game codes:
    // 0 - Invalid
    // 1 - Valid
    // 10 - White wins (by checkmate)
    // 11 - White wins (by resignation) -- not checked here
    // 12 - White wins (by time) -- not checked here
    // 20 - Black wins (by checkmate)
    // 21 - Black wins (by resignation) -- not checked here
    // 22 - Black wins (by time) -- not checked here
    // 30 - Draw (stalemate)
    // 31 - Draw (insufficient material)
    // 32 - Draw (by threefold repitition)
    // 33 - Draw (by 50 move rule) -- no choice to take
    // 34 - Draw (by agreement) -- not checked here
    // 35 - Draw (maximum number of moves)

    // Initiate Game
    let mut game_state = GameState {
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
        en_passant: (8,8),
        white_active: true,
        white_castle_king: true,
        white_castle_queen: true,
        black_castle_king: true,
        black_castle_queen: true,
        half_moves: 0,
        full_moves: 1,
    };

    // Run through each move, checking game code and updating the board
    for i in 0..half_moves {
        if !is_valid(turns[i],&mut game_state) {
            return GameCodes::Invalid;
        }

        if game_state.is_checkmate() {
            if game_state.white_active {
                return GameCodes::WhiteWinCheckmate;
            } else {
                return GameCodes::BlackWinCheckmate;
            }
        }
        if game_state.is_stalemate() {
            return GameCodes::DrawStalemate;
        }
        if game_state.is_insufficient_mat() {
            return GameCodes::DrawInsufficientMaterial;
        }
        if usize::from(game_state.half_moves) >= MAX_HALFMOVES {
            return GameCodes::DrawFiftyMoves;
        }
        if usize::from(game_state.full_moves) >= MAX_MOVES {
            return GameCodes::DrawMaxMoves;
        }

        // Check not threefold rep
        // TODO
    }
    return GameCodes::Valid;
}

fn main() {
    let default_turn = Turn {
        piece: Pieces::Empty,
        from_rank: 0,
        from_col: 0,
        to_rank: 0,
        to_col: 0,
    };
    let first_turn = Turn {
        piece: Pieces::P,
        from_rank: 3,
        from_col: 2,
        to_rank: 3,
        to_col: 4,
    };
    let mut turns = [&default_turn; MAX_MOVES*2];
    turns[0] = &first_turn;
    println!("Here");
    generate_game_code(turns, 1);
}
