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
    fn is_check(&self, white: bool) -> bool {
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
                    if (move_up && (king_rank + i > 7)) || (!move_up && (king_rank < i)) {
                        break;
                    }
                    let new_rank = {
                        if move_up {
                            king_rank + i
                        } else {
                            king_rank - i
                        }
                    };
                    if (move_right && (king_col + i > 7)) || (!move_right && (king_col < i)) {
                        break;
                    }
                    let new_col = {
                        if move_right {
                            king_col + i
                        } else {
                            king_col - i
                        }
                    };
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
                    let mut new_rank = king_rank;
                    let mut new_col = king_col;
                    if move_up_down {
                        if (move_pos && (king_rank + i > 7)) || (!move_pos && (king_rank < i)) {
                            break;
                        }
                        new_rank = {
                            if move_pos {
                                king_rank + i
                            } else {
                                king_rank - i
                            }
                        }; 
                    } else {
                        if (move_pos && (king_col + i > 7)) || (!move_pos && (king_col < i)) {
                            break;
                        }
                        new_col = {
                            if move_pos {
                                king_col + i
                            } else {
                                king_col - i
                            }
                        };
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
                if (rank_pos && (king_rank + rank_change > 7)) || (!rank_pos && (king_rank < rank_change)) {
                    break;
                }
                let new_rank = {
                    if rank_pos {
                        king_rank + rank_change
                    } else {
                        king_rank - rank_change
                    }
                };
                for col_pos in [false,true] {
                    let col_change = rank_change % 2 + 1;
                    if (col_pos && (king_col + col_change > 7)) || (!col_pos && (king_col < col_change)) {
                        break;
                    }
                    let new_col = {
                        if col_pos {
                            king_col + col_change
                        } else {
                            king_col - col_change
                        }
                    };
                    if (self.piece_board[new_rank][new_col] == Pieces::N) && (self.white_board[new_rank][new_col] != white) {
                        return true;
                    }
                }
            }
        }
        return false;
    }
    
    fn has_valid_move(&self) -> bool {    
        // For non-active color, see if any piece has a valid move
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

fn is_valid(turn: &Turn, curr_game: &mut GameState) -> bool { // I think I'm doing the mut wrong
    // Checks if the move is valid and updates the board

    // Check if both coordinates are valid
    if (turn.from_rank >= 8) || (turn.from_col >= 8) || (turn.to_rank >= 8) || (turn.to_col >= 8) {
        return false;
    }

    // Check the piece moves
    if (turn.from_rank == turn.to_rank) && (turn.from_col == turn.to_col) {
        return false;
    }

    // Recast rank and col as usize (needed for index)
    let from_rank: usize = turn.from_rank.into();
    let from_col: usize = turn.from_col.into();
    let to_rank: usize = turn.to_rank.into();
    let to_col: usize = turn.to_col.into();

    // Check if piece at original location is right
    if (curr_game.piece_board[from_rank][from_col] != turn.piece) || 
        (curr_game.white_active != curr_game.white_board[from_rank][from_col]) {
        return false;
    }

    // Check no same color piece at final location
    if (curr_game.piece_board[to_rank][to_col] != Pieces::Empty) && 
        (curr_game.white_active == curr_game.white_board[to_rank][to_col]) {
        return false;
    }

    // Check if piece can move to target square
    let rank_diff = cmp::max(from_rank,to_rank) - cmp::min(from_rank,to_rank);
    let col_diff = cmp::max(from_col,to_col) - cmp::min(from_col,to_col);
    if turn.piece.is_pawn()  {
        // Check movement is valid
        if (curr_game.white_active && to_rank < from_rank) || (!curr_game.white_active && to_rank > from_rank) {
            return false;
        }
        if rank_diff != 1 && rank_diff != 2 {
            return false;
        }
        if col_diff == 0 { // Going forward
            if rank_diff == 2 && ((curr_game.white_active && to_rank != 3) || (!curr_game.white_active && to_rank != 4)) {
                return false;
            }
            // Check the movement squares
            for i in 1..=rank_diff {
                let new_rank = {
                    if curr_game.white_active {
                        from_rank + i
                    } else {
                        from_rank - i
                    }
                };
                if curr_game.piece_board[new_rank][from_col] != Pieces::Empty {
                    return false;
                }
            }
            // Update board
            default_update(turn, curr_game);
            if rank_diff == 2 {
                curr_game.en_passant = {
                    if curr_game.white_active {
                        (turn.from_rank+1,turn.from_col)
                    } else {
                        (turn.from_rank-1,turn.from_col)
                    }
                };
            }
        } else if col_diff == 1 { // Capture/en passant
            if rank_diff != 1 {
                return false;
            }
            if curr_game.piece_board[to_rank][to_col] != Pieces::Empty && (turn.to_rank,turn.to_col) != curr_game.en_passant {
                return false;
            }
            // Update board
            default_update(turn, curr_game);
        } else { // Invalid
            return false;
        }

        // Handle promotions separately
        if (curr_game.white_active && (to_rank == 7)) || (!curr_game.white_active && (to_rank == 0)) {
            if turn.piece == Pieces::P {
                return false;
            }
            // Update board
            curr_game.piece_board[to_rank][to_col] = {
                match turn.piece {
                    Pieces::PToR => Pieces::R,
                    Pieces::PToN => Pieces::N,
                    Pieces::PToB => Pieces::B,
                    Pieces::PToQ => Pieces::Q,
                    _ => return false,
                }
            };
        }
        else if turn.piece != Pieces::P {
            return false;
        }
    } else if turn.piece == Pieces::K {
        if rank_diff > 1 || col_diff > 2 {
            return false;
        }
        if col_diff == 2 { // Handle castling separately (by moving the king square by square)
            if rank_diff != 0 {
                return false; 
            }
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
            } else if to_col == 2 { // Queen-side
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
            } else {
                return false;
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
        // Remove castling
        if curr_game.white_active {
            curr_game.white_castle_king = false;
            curr_game.white_castle_queen = false;
        } else {
            curr_game.black_castle_king = false;
            curr_game.black_castle_queen = false;
        }
    } else {
        let rank_move: usize = {
            if turn.piece == Pieces::N || rank_diff == 0 {
                rank_diff
            } else {
                1
            }
        };
        let rank_move_pos: bool = turn.from_rank < turn.to_rank;
        let col_move: usize = {
            if turn.piece == Pieces::N || col_diff == 0 {
                col_diff
            } else {
                1
            }
        };
        let col_move_pos: bool = turn.from_col < turn.to_col;
        let dist: usize = {
            if turn.piece == Pieces::N {
                1
            } else {
                cmp::max(rank_diff,col_diff)
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
        default_update(turn,curr_game);
    }

    // Check if check
    return !curr_game.is_check(curr_game.white_active);
}

fn default_update(turn:&Turn, curr_game: &mut GameState) -> () {
    let from_rank: usize = turn.from_rank.into();
    let from_col: usize = turn.from_col.into();
    let to_rank: usize = turn.to_rank.into();
    let to_col: usize = turn.to_col.into();
    curr_game.piece_board[from_rank][from_col] = Pieces::Empty;
    curr_game.white_board[from_rank][from_col] = false;
    curr_game.piece_board[to_rank][to_col] = turn.piece;
    curr_game.white_board[to_rank][to_col] = curr_game.white_active;
    curr_game.en_passant = (8,8);

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
    if ((from_col == 0) && (from_rank == 0)) || ((to_col == 0) && (to_rank == 0)) {
        curr_game.white_castle_queen = false;
    }
    if ((from_col == 7) && (from_rank == 0)) || ((to_col == 7) && (to_rank == 0)) {
        curr_game.white_castle_king = false;
    }
    if ((from_col == 0) && (from_rank == 7)) || ((to_col == 0) && (to_rank == 7)) {
        curr_game.black_castle_queen = false;
    }
    if ((from_col == 7) && (from_rank == 7)) || ((to_col == 7) && (to_rank == 7)) {
        curr_game.black_castle_king = false;
    }
}

#[derive(PartialEq)]
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

fn generate_game_code(turns: [&Turn; MAX_MOVES*2], half_moves: usize) -> GameCodes {
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
        
        if !game_state.has_valid_move() {
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
        if usize::from(game_state.half_moves) >= MAX_HALFMOVES {
            return GameCodes::DrawFiftyMoves;
        }
        if usize::from(game_state.full_moves) >= MAX_MOVES {
            return GameCodes::DrawMaxMoves;
        }

        // Check not threefold rep
        // TODO

        game_state.white_active = !game_state.white_active;
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
        from_rank: 1,
        from_col: 2,
        to_rank: 3,
        to_col: 2,
    };
    let mut turns = [&default_turn; MAX_MOVES*2];
    turns[0] = &first_turn;
    println!("Here");
    let code = generate_game_code(turns, 1);
    match code {
        GameCodes::Valid => println!("Valid"),
        GameCodes::Invalid => println!("Invalid"),
        _ => println!("Sad"),
    };
}
