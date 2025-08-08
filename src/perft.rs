use crate::{
    ChessBoard,
    bitboard::SQUARE_SYM,
    chessmove::{ChessMove, MoveType},
};

//FIXME wtf is this
impl ChessBoard {
    pub fn perft_count(&self, depth: usize) -> u64 {
        if depth == 0 {
            // this is used when printing the individual moves in a given position
            return 1;
        }

        let moves = self.generate_moves();
        if depth == 1 {
            return moves.len() as u64;
        }
        let mut total: u64 = 0;
        for chessmove in moves {
            let mut new_chessboard: ChessBoard = *self;
            new_chessboard.update_state(chessmove);
            total += new_chessboard.perft_count(depth - 1);
        }
        return total;
    }
}

//FIXME wtf is this
impl ChessMove {
    pub fn print_move(&self) -> String {
        if let MoveType::Promotion(piece) = self.move_type() {
            return format!(
                "{}{}{}",
                SQUARE_SYM[self.source().to_usize()],
                SQUARE_SYM[self.target().to_usize()],
                piece.to_uci_char()
            );
        } else {
            return format!("{}{}", SQUARE_SYM[self.source().to_usize()], SQUARE_SYM[self.target().to_usize()]);
        }
    }
}
