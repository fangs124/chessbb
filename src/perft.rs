use crate::{
    ChessBoard, ChessBoardCore,
    bitboard::SQUARE_SYM,
    chessmove::{ChessMove, MoveType},
    zobrist::ZobristTable,
};

//FIXME wtf is this
impl ChessBoardCore {
    pub fn perft_count(&self, zobrist_table: &mut ZobristTable, depth: usize) -> u64 {
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
            let mut new_chessboard: ChessBoardCore = *self;
            new_chessboard.update_state(chessmove);
            let current_hash = new_chessboard.hash();
            zobrist_table.add(current_hash);
            total += new_chessboard.perft_count(zobrist_table, depth - 1);
            zobrist_table.remove_last(current_hash);
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

//FIXME wtf is this
impl ChessBoard {
    pub fn perft_count(&mut self, depth: usize) -> u64 {
        if depth == 0 {
            // this is used when printing the individual moves in a given position
            return 1;
        }

        let moves = self.try_generate_moves().0;
        if depth == 1 {
            return moves.len() as u64;
        }
        let mut total: u64 = 0;
        total += self.core.perft_count(&mut self.zobrist_table, depth);
        return total;
    }
}
