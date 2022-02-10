use std::hash::{Hash, Hasher};

struct GameState {
    piece_board: [[u8; 8]; 8],
    color_board: [[u8; 8]; 8],
    en_passant: u8,
    white_active: bool,
    white_castle_king: bool,
    white_caste_queen: bool,
    black_castle_king: bool
    black_castle_queen: bool,
    half_moves: u16, // Half moves since last pawn move or capture
    full_moves: u16,
}

impl Hash for GameState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.piece_board.hash(state);
        self.color_board.hash(state);
        self.en_passant.hash(state);
    }
}

struct Move {
    piece: u8, 
    from_rank: u8,
    from_col: u8,
    to_rank: u8,
    to_col: u8,
}

const MAX_MOVES: u16 = 500;
const MAX_HALFMOVES: u16 = 100; // 50 move rule

fn is_valid(move: &Move, curr_game: &GameState) -> bool {
    true
}

fn is_check(move: &Move, curr_game: &GameState) -> bool {
    true
}

fn is_checkmate(move: &Move, curr_game: &GameState) -> bool {
    // Assume check
    true
}

fn is_stalemate(move: &Move, curr_game: &GameState) -> bool {
    // Assume not check
    true
}

fn is_insufficient_mat(move: &Move, curr_game: &GameState) -> bool {
    // K v k
    // KB v k
    // KN v k
    // KB v kb (bishops are the same color)
    true
}

fn only_king(is_white: bool, curr_game: &GameState) -> bool {
    // If a player flags, their opponent wins unless player only has a king on the board 
    // or above conds are met
    true
}

fn is_capture(move: &Move, curr_game: &GameState) -> bool {
    // Check if the move is a capture for 50 move rule
    true
}

fn generate_game_code(moves: [&Move; MAX_MOVES], full_moves: u16) -> u8 {
    // Game codes:
    // 0 - Invalid
    // 1 - Valid
    // 2 - Checkmate
    // 3 - Draw (stalemate)
    // 4 - Draw (insufficient material)
    // 5 - Draw (by threefold repitition)
    // 6 - Draw (by 50 move rule), no choice to take
    // 7 - Draw (by agreement), not checked here

    // Initiate Game
    // Run through each move, checking game code and updating the board
    // Return last game code
}