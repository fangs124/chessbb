use chessbb::chessmove::*;
use chessbb::*;
//use chessbb::chessmove::ChessMove;
use std::{env, time::Instant};

extern crate chessbb;
fn main() {
    unsafe { env::set_var("RUST_BACKTRACE", "full") };
    /* from starting pos */
    /*
    let mut chessboard = ChessBoard::start_pos();
    let move1 = ChessMove { data: 1487 }; //a2a3
    let move2 = ChessMove { data: 2551 }; //a7a5
    let move3 = ChessMove { data: 1934 }; //b2b4
    let move4 = ChessMove { data: 1959 }; //a5b4
    let move5 = ChessMove { data: 1869 }; //c2c4
    let move6 = ChessMove { data: 1535 }; //a8a3
    let move7 = ChessMove { data: 1988 }; //d1a4
    //three errors: b7b5, f7f5, f7f6
    let move8a = ChessMove { data: 2486 }; //b7b5
    let move9a = ChessMove { data: 2461 }; //c4b5
    //two errors: a3a1, c7c5
    let move10aa = ChessMove { data: 471 }; //a3a1
    let move11aa = ChessMove { data: 479 }; //a4a1
    let move12aa = ChessMove { data: 2421 }; //c7c5
    let move13aa = ChessMove { data: 259 }; //e1d1
    let move14aa = ChessMove { data: 1438 }; //b4b3
    let move15aa = ChessMove { data: 2503 }; //a1a5
    let move16aa = ChessMove { data: 2226 }; //f7f5
    let move17aa = ChessMove { data: 3879 }; //a5d8
    chessboard.update_state(move1);
    chessboard.update_state(move2);
    chessboard.update_state(move3);
    chessboard.update_state(move4);
    chessboard.update_state(move5);
    chessboard.update_state(move6);
    chessboard.update_state(move7);
    chessboard.update_state(move8a);
    chessboard.update_state(move9a);
    chessboard.update_state(move10aa);
    chessboard.update_state(move11aa);
    chessboard.update_state(move12aa);
    chessboard.update_state(move13aa);
    chessboard.update_state(move14aa);
    chessboard.update_state(move15aa);
    chessboard.update_state(move16aa);
    chessboard.update_state(move17aa);
    */
    let start_fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"; //kiwipete
    let mut chessboard = ChessBoard::from_fen(start_fen);

    let move1 = ChessMove { data: 3363 }; //e5d7
    let move2 = ChessMove { data: 2479 }; //a6b5
    let move3 = ChessMove { data: 4020 }; //d7b8
    let move4 = ChessMove { data: 3374 }; //b6d7
    let move5 = ChessMove { data: 2443 }; //e2b5
    let move6 = ChessMove { data: 2933 }; //c7c6
    let move7 = ChessMove { data: 3390 }; //b8d7
    chessboard.update_state(move1);
    chessboard.update_state(move2);
    chessboard.update_state(move3);
    chessboard.update_state(move4);
    chessboard.update_state(move5);
    chessboard.update_state(move6);
    chessboard.update_state(move7);

    let mut moves = chessboard.generate_moves();
    moves.sort();
    println!("chessmove: {:#?}", moves[15]);
    //println!("castle_bools: {:#?}", chessboard.castle_bools);
    //println!("chessboard.enpassant_bb:\n{}\n", chessboard.enpassant_bb);
    println!("chessboard.pinned_bb:\n{}\n", chessboard.pinned_bb);
    println!("chessboard.pinner_bb:\n{}\n", chessboard.pinner_bb);
    println!("==== start position ====\n");
    println!("{}", chessboard);
    println!("========================");
    panic!();
    let mut depth: usize = 1;
    let max_depth: usize = 7;
    while depth <= max_depth {
        let now = Instant::now();
        let total = chessboard.perft_count(depth);
        let elapsed = now.elapsed();
        let mut moves = chessboard.generate_moves();
        moves.sort();
        let mut result_str_vec = Vec::<String>::new();

        for chessmove in moves {
            let mut s = chessmove.print_move();
            let mut state = chessboard.duplicate();
            state.update_state(chessmove);
            let branch_total = state.perft_count(depth - 1);
            s.push_str(format!(" - {}", branch_total).as_str());
            result_str_vec.push(s);
        }
        println!("depth: {}, time: {}, total: {}", depth, elapsed.as_secs(), total);

        for result_str in result_str_vec {
            println!("{}", result_str);
        }
        println!("\n");
        depth += 1;
    }
}
