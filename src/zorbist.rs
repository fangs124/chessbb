use crate::chessmove::Castling;
use crate::{ChessBoard, square::Square};
use crate::{bitboard::*, square, zorbist};

use std::hint::assert_unchecked;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(super) struct ZorbistHash {
    value: u64,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(super) struct ZorbistTable {
    data: [ZorbistHash; 1 << 14],
    index: usize,
}

impl ZorbistTable {
    pub(super) const fn new(hash: ZorbistHash) -> ZorbistTable {
        let mut data: [ZorbistHash; 1 << 14] = [ZorbistHash { value: 0 }; 1 << 14];
        data[0] = hash;
        ZorbistTable { data, index: 0 }
    }

    pub(super) const fn initial_table() -> ZorbistTable {
        ZorbistTable::new(ZorbistHash::initial_hash())
    }

    pub(super) const fn add(&mut self, hash: ZorbistHash) {
        assert!(self.index < (1 << 14));
        self.index += 1;
        self.data[self.index] = hash;
    }

    pub(super) const fn last_hash(&self) -> ZorbistHash {
        self.data[self.index]
    }

    pub(super) const fn count_hash(&self, hash: ZorbistHash) -> usize {
        let mut i: usize = 0;
        let mut count: usize = 0;
        while i <= self.index {
            if self.data[i].value == hash.value {
                count += 1;
            }
            i += 1
        }
        return count;
    }
}

impl ZorbistHash {
    pub(super) const fn new(value: u64) -> ZorbistHash {
        ZorbistHash { value }
    }

    pub(super) const fn initial_hash() -> ZorbistHash {
        let mut value: u64 = 0;

        //starting side is white, no hash
        //no en-passant in starting position either

        //piece hash
        let mut i: usize = 0;
        while i < 64 {
            if let Some(piece_data) = ChessBoard::INITIAL_MAILBOX[i] {
                value ^= PIECE_HASH[i][cp_index(piece_data)];
            }
            i += 1;
        }

        //castle hash
        i = 0;
        while i < 4 {
            value ^= CASTLE_HASH[i];
            i += 1;
        }

        return ZorbistHash { value };
    }

    pub(super) const fn compute_hash(chessboard: &ChessBoard) -> ZorbistHash {
        //side hash
        let mut value = match chessboard.side_to_move {
            crate::bitboard::Side::White => 0u64,
            crate::bitboard::Side::Black => SIDE_HASH[0],
        };

        //piece hash
        let mut i: usize = 0;
        while i < 64 {
            if let Some(piece_data) = chessboard.mailbox[i] {
                value ^= PIECE_HASH[i][cp_index(piece_data)];
            }
            i += 1;
        }

        //castle hash
        i = 0;
        while i < 4 {
            if chessboard.castle_bools[i] {
                value ^= CASTLE_HASH[i]
            }
            i += 1;
        }

        //en-passant hash
        let mut enpassant_bb = chessboard.enpassant_bb;
        while enpassant_bb.is_not_zero() {
            let square = enpassant_bb.lsb_square().unwrap();
            value ^= ENPASSANT_FILE_HASH[COLS[square.to_index()]];
            enpassant_bb = enpassant_bb.pop_bit(square);
        }

        return ZorbistHash { value };
    }

    pub(super) const fn compute_castle_hash(chessboard: &ChessBoard) -> ZorbistHash {
        let mut value = 0u64;

        let mut i: usize = 0;
        while i < 4 {
            if chessboard.castle_bools[i] {
                value ^= CASTLE_HASH[i]
            }
            i += 1;
        }
        return ZorbistHash { value };
    }

    pub(super) const fn compute_enpassant_hash(enpassant_bb: BitBoard) -> ZorbistHash {
        let mut value: u64 = 0;
        let mut enpassant_bb = enpassant_bb;
        while enpassant_bb.is_not_zero() {
            let square = enpassant_bb.lsb_square().unwrap();
            value ^= ENPASSANT_FILE_HASH[COLS[square.to_index()]];
            enpassant_bb = enpassant_bb.pop_bit(square);
        }
        return zorbist::ZorbistHash { value };
    }

    const fn update_hash(&self, s: Square, t: Square, s_p: ChessPiece, t_p: Option<ChessPiece>) -> ZorbistHash {
        let mut value = self.value;
        value ^= PIECE_HASH[s.to_index()][cp_index(s_p)];
        value ^= PIECE_HASH[t.to_index()][cp_index(s_p)];
        value ^= match t_p {
            Some(t_p) => PIECE_HASH[t.to_index()][cp_index(t_p)],
            None => 0u64,
        };
        todo!()
    }

    pub(crate) const fn piece_hash(square: Square, chesspiece: ChessPiece) -> ZorbistHash {
        ZorbistHash { value: PIECE_HASH[square.to_index()][cp_index(chesspiece)] }
    }

    pub(crate) const fn castle_hash(castling: Castling) -> ZorbistHash {
        match castling {
            Castling::KINGSIDE(Side::White) => ZorbistHash { value: CASTLE_HASH[0] },
            Castling::QUEENSIDE(Side::White) => ZorbistHash { value: CASTLE_HASH[1] },
            Castling::KINGSIDE(Side::Black) => ZorbistHash { value: CASTLE_HASH[2] },
            Castling::QUEENSIDE(Side::Black) => ZorbistHash { value: CASTLE_HASH[3] },
        }
    }

    pub(crate) const fn side_hash() -> ZorbistHash {
        ZorbistHash { value: SIDE_HASH[0] }
    }
}

impl BitAnd for ZorbistHash {
    type Output = ZorbistHash;
    fn bitand(self, rhs: ZorbistHash) -> Self::Output {
        ZorbistHash { value: self.value & rhs.value }
    }
}

impl BitAndAssign for ZorbistHash {
    fn bitand_assign(&mut self, rhs: Self) {
        self.value &= rhs.value;
    }
}

impl BitOr for ZorbistHash {
    type Output = ZorbistHash;
    fn bitor(self, rhs: ZorbistHash) -> Self::Output {
        ZorbistHash { value: self.value | rhs.value }
    }
}

impl BitOrAssign for ZorbistHash {
    fn bitor_assign(&mut self, rhs: Self) {
        self.value |= rhs.value;
    }
}

impl BitXor for ZorbistHash {
    type Output = ZorbistHash;
    fn bitxor(self, rhs: ZorbistHash) -> Self::Output {
        ZorbistHash { value: self.value ^ rhs.value }
    }
}

impl BitXorAssign for ZorbistHash {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.value ^= rhs.value;
    }
}

impl Not for ZorbistHash {
    type Output = ZorbistHash;
    fn not(self) -> Self::Output {
        ZorbistHash { value: !self.value }
    }
}

/* consts */

#[rustfmt::skip]
const PIECE_HASH: [[u64; 12]; 64] = [
    [
        0xa1aad78b0d1c4872, 0xdc4cb8dca1e1c262, 0xb962357fd09d116f, 0x9fd568e5146ba58d, 0x6c0b797247e279dd, 0xa65d04347c029d24, 
        0x49307a9f06340884, 0x5903702bf6a51be8, 0x544acc826be1d489, 0xaa3ecba3fe449cb1, 0xc09cf396fe4d72ff, 0x408b4e9e61f443bc,
    ],
    [
        0x8dbbb1b49126ad2b, 0xce3771a1384b5a02, 0xd2019d6c48af5ed8, 0xfa18d2c8376fb760, 0x7b8e21329b922f22, 0xa5ea2baf63621e43,
        0x586455cba0759610, 0x2c9fc56f58a18f00, 0x5598a2795b4e9e56, 0x6db8e5b6dff7e0f5, 0x3b1ed55d300e8c38, 0x782a2a5f41d164dc,
    ],
    [
        0x87e2f5d5cbf6cd7f, 0xeab107784ca7efba, 0xcc73d6adbcab3bec, 0xe7e9e3df49c91fde, 0x451d42229cabcfbc, 0xdd1563628f7a7521,
        0x8d85d3b82eb06e08, 0x0734683d1a049dcd, 0x88f7a92b861a4dc2, 0xe238c523fed7a6ca, 0x0a0e6ffd6568fe5c, 0xdc125900f785565a,
    ],
    [
        0x66f6619dc5e0afe9, 0x6a3cfebf771807e5, 0x63cacc39b88f8f3c, 0x24414a42fb6a20c8, 0xee63881c942c9a79, 0xc257f282f2bc2419,
        0xf3ee45db04f6c6f5, 0x384e6571cc974b40, 0x0c695ece9b56c29c, 0xffea5d2ed349bb04, 0xfeef8fccc6d0fa00, 0xfb31d9bde1b1a3c8,
    ],
    [
        0x3f03cc1ead700a47, 0x98174fb2a49103bc, 0xabbcd2c49895b455, 0xdf221cddfcf51284, 0x4c6bee93457757e7, 0xa27c1dbc632d3001,
        0x69f2c3e8f4e55362, 0x18cc5587e97fd9f5, 0x75a8769e4b609cf1, 0xb6e6b5beadebb5d3, 0xd62ab563ba6351be, 0xa9ce42d839a0abfd,
    ],
    [
        0xd1955949c48067d7, 0x5161732e5d5997a4, 0x78a69cfb7c048ddc, 0x42413ff8737e6f57, 0x828f51969a0ad21e, 0x45e6f8f6fac73d42,
        0xe46d5cf5d1c24939, 0x8539bc207ae0ae18, 0x7876c827d5650c17, 0x007c29b0c2ce9acb, 0x9a9c20cb36965022, 0x56506bdc79baccf3,
    ],
    [
        0xb00d6812fc7dc0f6, 0x2a491ad1a0bfdc85, 0x038e27c433f4fcec, 0x58639afaaf876e71, 0xdb94554bdab6f884, 0x83b3245ca487cf92,
        0x055f2d63ae262a96, 0x0bbaf1e405fde7b4, 0x508ecdcef8c4ec57, 0x522838825bee4244, 0xa00df3ff6c6f4d13, 0xc0af25f155ff92f1,
    ],
    [
        0xd9472dfd2cb024a2, 0x20c57c969ae334ce, 0xb68c6cd0c10eba4b, 0xe1557c6d495810e0, 0xe4d980e2778464d5, 0x9eccacbee1553052,
        0x7e6d3c659524aa37, 0xe1237c7efb21d384, 0x65197fe302804176, 0xff38236875ecc3e1, 0x766c368ec2997e63, 0x7e6e619ab11db354,
    ],
    [
        0x4f7ee98e776ce906, 0x761b5f50bb963288, 0x5ca76693eadd92f7, 0x858824e409cb46bd, 0x3f1ea15881d5108d, 0x4612145b1c582fd1,
        0xa8922f1ada3cdbc2, 0x9198329526afb614, 0x6c8a93c3e2dacf49, 0xfd56c179ace1da1d, 0x2102342099ff5fa1, 0x01e14ffdbf74bd18,
    ],
    [
        0x27379e20f85265d6, 0x1a697a53cc7144c0, 0xfb36d6732de2ee0d, 0xbf46febbdaed76d6, 0xcf2fd024cf1d04c9, 0x0a2e93f556d66c48,
        0xc96f3cddd54e9bec, 0xc411796cc51d0650, 0x845a981f72e50819, 0xc211629b816a4df1, 0xe20595b56e2772ea, 0xa91dac4eaf82f943,
    ],
    [
        0xed6c28addc849eb7, 0x6be2de312e3896e0, 0x4db25d5f97a5f192, 0x756bbc864c252c47, 0x9f2926204653b23d, 0x56c736fe74cb9e37,
        0xb45db823f3147433, 0x4d8f8e651026ff4c, 0x07fc0ae1a1214ee8, 0x47afcd725c807226, 0x7c99cd4b2259e786, 0x0f6660db925f5cc6,
    ],
    [
        0x38e4652c34634466, 0x08a6daf56764ed7c, 0x29fc667a7aa5dca5, 0x390e7d93b9a92877,0x62144a1f23397eac, 0xf0b84d2f1f1875eb,
        0x95fc7d52f6943a05, 0x7200fd4f5bc40087, 0xf6a8d183b153cb72, 0x3ac0f5f8c05fba5b, 0xc931cdaab339a84b, 0xa3ff5bfe3c6671e4,
    ],
    [
        0x2302f9d671894cff, 0x057861bf77ffbdff, 0x385e0370cfbb2f4b, 0xadbf84ef0571c63a, 0x458513a03d35d384, 0x15225ffea2e64da9,
        0x7f0ed3a931195816, 0x900cc912de77a358, 0x23adab8f407519c1, 0xf76f37d68bb02e3d, 0xe855fbe13c4878b0, 0xdf149745e426adfe,
    ],
    [
        0xec5ac3402e9c16ba, 0xc87d32c0158007be, 0x68a7262d48364c72, 0xdf47864f7d875ac2, 0x076a220c4f5caa36, 0x95e4ed12e24fbe61,
        0x11ca45a279b7f8eb, 0x42674b90501b9a45, 0x55d976633a51477e, 0xb69d48b860af265f, 0xa285ac67e182f242, 0x7c658b91a3bcca33,
    ],
    [
        0x90a695db3bcab026, 0xc8e49834fe533639, 0xe40e6eddffa98f11, 0x077dacccd8583e4b, 0x2a0056fc218d76e6, 0x7f9107d606ed5b60,
        0xab89df1df07cb2a2, 0xf6dc8f90f9ea0b65, 0xc3528f8c3b7f169e, 0x36fac69b79890ed4, 0x03ff233d00e94290, 0x9c8c65cd7fddd0e3,
    ],
    [
        0x1a8ce26a78fdbf43, 0x2d5f566b7e866aca, 0x82d7d0bc79eaf1c9, 0xe4948b2266d1fe35, 0xbab1a9e840a4222a, 0xbb978b8e5ea06f6b,
        0x729450f7ec034625, 0x86196904293757f9, 0xcdf149385c2d8306, 0x08633261821500fa, 0xbfb2ee5317de27b6, 0x756ccdae0767a548,
    ],
    [
        0x61dc2c91a92403a6, 0xf99908d01ad16029, 0x84472132cca63a98, 0x77352c18601410e1, 0x243c7231eb326f9b, 0xe1ca14f8c44f4150,
        0x072892e5559092d0, 0x3156296408d4b84e, 0x254ec453e3502755, 0x013b9e5949413405, 0x6b911fa36bb3102d, 0xf8b8607260669c87,
    ],
    [
        0x1a599af2e3d69e5b, 0xe3f72ad8a1677ef5, 0x5490bb4f8b6b82e9, 0xf709f2eca6d2c230, 0xb0167b69970c01a7, 0x76c2fbc09e95573f,
        0x7bccafbc4e5dee33, 0x5ae6c414f3595528, 0x9181cc023a6f9836, 0x4a5ec8482656e69f, 0x6925533f6c5508df, 0xae30662196965ce5,
    ],
    [
        0x405b66ecc7e6b5e6, 0x381c2cd0b7b42f00, 0xc937551c494c32dc, 0x58950c99120b1602, 0xa40d86db03b1031f, 0x5fcbb8829fb6b447,
        0xca6d35f23bc0461c, 0xb9ee004bc1665e2f, 0x5b135f79a0b482af, 0x2f242f7ae0699a89, 0x88ef5da4902528be, 0xd6132dd721ad7906,
    ],
    [
        0xc95b3a7841de8d1b, 0x1bb672eef8210617, 0xbc90f4bcf5031399, 0x3ce720dd27932c97, 0x24ec243dae09d823, 0xc5738093c9fee10f,
        0xcdab3010068c2f8d, 0x56991e810ec1a431, 0xe1d62b1eb3b8e90b, 0x50402f36436cef7c, 0xbb689c177117bb8c, 0x624099df9c477d20,
    ],
    [
        0x1bb459efdce7f622, 0xa475d08d724cfa70, 0xde5caf5d370009de, 0xfcffc38735340efc, 0x4f1cb639f4827f6c, 0xb402ca80a6cdc850,
        0x759f5dfb01e417e4, 0x0e6f1f3dc2d1bed4, 0x811b6ef4595f4bed, 0x0fe98ffcf40db2f9, 0xb53b992b15e47316, 0x6f366d6190720380,
    ],
    [
        0xa1e197de6d4fbc62, 0xcfbbbda3ca076543, 0x89663db3ac91f63b, 0xd133cf5c3faff5c7, 0x78a16cf779bf1fc3, 0x6528b5f9d3c68eff,
        0x14e8056472a3a6c8, 0x3821559ce7310d3f, 0x37d8ddd8c18d57df, 0x82229c6bf1a8267b, 0xf20fa18365230a34, 0x6a691faf56d72c49,
    ],
    [
        0x1a6441c75118156f, 0x14ab1e53eae62323, 0x9745635c7cabcca6, 0x478a6878411a0e47, 0xd509f83198a89f79, 0x7dd3b90414b0687f,
        0xe8d1c7ce646d32bc, 0xd4fe3f7f0c966925, 0xbad0fe138ffddd34, 0x9a53613f8e7e8b4b, 0xd91abb901f2461de, 0xc1465acf8777ed8a,
    ],
    [
        0xe819270b9aa3cca8, 0xced9ca618328212a, 0x461134f8aec8fe8f, 0xa1508a476cf0f272, 0x5f7e012a52d31a3d, 0x29118d3fcbf5dce9,
        0x3b552678cbbd074b, 0x82c68b4df9aed21a, 0x0dd5bcf33ad3cdcf, 0xa7d5b468ecc65840, 0x6d8ebfef0d3eae6f, 0x77cec04966f83f04,
    ],
    [
        0xfa0fdb82cb37b2e1, 0x1e6e8e78525e87ef, 0xae7b3bf3392459b6, 0xe9de5b195fee1809, 0x7942b312a5d786f9, 0xc7283a11e8110b81,
        0xdaa5c5ddddee21ab, 0xd720528260d60a19, 0x5d58eede55e91219, 0x6dc18437e93c91b6, 0x0ee8ca722e1bb0df, 0x69d389c3d0447a50,
    ],
    [
        0x6b3a65ba36bdfc13, 0x3d93c1e0d84d1325, 0x90f17382c7183ba8, 0x3567aeb6b8f0444f, 0xa20f08181720912c, 0xaef3440b26ccdb48,
        0x53885bc6e43ecc89, 0xcfd2f4fb2db98917, 0x266346640cc45ef6, 0x2ec9171d1d4d9fd6, 0x863a972be232ee44, 0xb294a0a496d83bcb,
    ],
    [
        0xb697d125c45d9086, 0xe4202485e119e287, 0xec1f09861ae4e064, 0x0a96d69664dd76cf, 0x50c59a9a80657a2e, 0x86c369e4577b639f,
        0x3d7444e81477cc5a, 0x41b98e4eeefa1707, 0xd93659688ea9948c, 0xaa6138572789d197, 0x896f05784c9a9b86, 0x831e8b29d59ff789,
    ],
    [
        0xdfd04661e7cec80b, 0x0e482ca46285c11d, 0x2d19778219088eb3, 0x2f842306c0a3a7fd, 0x4fac71dfa1b4c0ab, 0x72c48687520aafaa,
        0x04ccf4fe8f861fa6, 0xf268661dd7ddb58b, 0x65cf50d30e5fc6ab, 0xe3921255a583641e, 0x5e2a7e7687d1904a, 0xd7aeba0df3486da3,
    ],
    [
        0xe2283968b6fdb89e, 0x214df850e2506141, 0x599ca5aea9cc90b2, 0x894798f739d68dc9, 0xf1bafd8612b4d785, 0x2c5294d6f3bb5ce5,
        0xf648d36000bcadb7, 0xc1f1771c781eb9af, 0x5859b1a7ac7b754f, 0x98d8ee60c5ced540, 0x6a9e3e1fce41c925, 0xdbb4cdc44b0da8cf,
    ],
    [
        0x358da24639a4597f, 0x865e95be5c2697b4, 0x0abcc9d92ffd74de, 0xed63846c9907bc99, 0x2ed42d0cab638ca8, 0x0bbf9fd7f0d177ad,
        0x96d980a3da6f7b74, 0x0d15e54fea850f1c, 0x85a0f116f73e1c91, 0x7eae6e8a29af0fa1, 0x6466e444b7a02c29, 0xaa5f6bf63c3beb4e,
    ],
    [
        0x17b2a298b6860bec, 0x88c93825845ea9d6, 0x3172f5e1f8461636, 0x7fb5b8f24020e65a, 0xb4d5945bbbb93a72, 0xb7ca4d3ef7c56570,
        0x46b76a29ab224f7a, 0x31096525d7fc124a, 0xd60b0ea26bd4b53d, 0x2a543787d4e65c12, 0x0016cf17cdbe4337, 0xcca7bb07ff65da0e,
    ],
    [
        0x6e45adde20783360, 0xdcee09f065ae65b7, 0xdd7bb0041da5627d, 0x3ae31003facce4bb, 0x719764bb01a01e13, 0x77f9c265d317ab42,
        0xf725180dd03f9d78, 0xe765915d7bd8ae4b, 0x26af0506c4913308, 0x4a204f2acd30c9e4, 0x988f8867439369f1, 0x9429dec3d6251337,
    ],
    [
        0x8280625b73731e0a, 0x16a4cb6443910392, 0x1e0d94b4f2d18717, 0xc4a4cfc9f9f841e2, 0xa30a27b7de6be6fa, 0xc1f2a6b8360ee57b,
        0xf88484f143345ee2, 0x8e1788ea5af80650, 0x8b124b0b6be0be9f, 0x32417b483722e3f2, 0x3e4f2a2be8bffd55, 0x5ad71f16e25ec7a2,
    ],
    [
        0xa044a0a7d36331b6, 0x00c7998414cbf185, 0x7f472cab79069ddc, 0x129a4ce477f27744, 0x875a8fc416e84807, 0xab2eb2cf9d4eba4b,
        0xfe897261b97d3105, 0x501b89e091c8fb10, 0x58601400483c6696, 0x13dcdc368e4c00ee, 0x2a7b5dc4e4dfd1fb, 0x190bbbd7eca45329,
    ],
    [
        0xf9dc3709def9caab, 0x31a87874a92de7c6, 0xc0721cf347ef962d, 0x061e9a993a6e2f2c, 0x599e9025a1588fe9, 0xb7668d700276f601,
        0x282734d429dfab53, 0xe91341e50b42a04f, 0x219d4f48c74cf69b, 0x90c879fdb294af7b, 0x1ffa3d0f51d6c549, 0x7cf6231fdfcd3c3a,
    ],
    [
        0x42583edf115a66dc, 0xad4a26cf89171e78, 0x4d9e78ef7d236c9b, 0x4343df5b539fcc2d, 0xc67b812cd3420812, 0x0d73fb0af59aae09,
        0xe6ff1356b251d72d, 0x4505ea85f72bdd50, 0x594b27268d1c3bbb, 0x29383f819f2cc9c4, 0x6bf7f06c05106179, 0xb58f8ffc81726480,
    ],
    [
        0x7f6bef5b19c235bd, 0xbb16e68cafe77ad0, 0x2b191bef61527d7d, 0x15ec733725fcda53, 0xba4db6236b10b191, 0xbedb971fbc6e36ec,
        0x8e8b4e2ffda14830, 0x0c9c9bc22537ff05, 0xfde2efe0a59a2219, 0x228dadbce344431c, 0x8a33ec2f665ca8f8, 0x0ad96874a43fa6c8,
    ],
    [
        0x3106c65c9bfa89b4, 0x14602f4ac32cffe3, 0x835b56df55b203a5, 0xa7707e7f1296c763, 0x8471a782186f2a2b, 0xcce71d5ef757adba,
        0x72c007eeba774799, 0xbda16a54fe6eb63d, 0xba221cdf550094aa, 0x7e79347292356a60, 0x5f12a72bd0e51188, 0xcbafb65179228bc7,
    ],
    [
        0xa6b597b6fea1bc12, 0xb78968856ffce9cc, 0x466de9ff4f970eb5, 0xaece8628d080f91c, 0x3a7412ab570609bf, 0x098c7a1a2f26a6d9,
        0x6219af99d11cb1ac, 0x6359a750b4868ba2, 0x29132d6d91d60e41, 0x23b5130c9b3591f7, 0xec4f2d6868262734, 0x6ad0edad492d72ba,
    ],
    [
        0xe73505746f32752f, 0x799fc5915e5fdc9f, 0x2408597cbe8dc0b0, 0x4a7efcbb69c0c324, 0x811ea39e1701e6f2, 0x561ff5a87792399e,
        0x858eb13ff7be0d45, 0x7e4688b8614ea79b, 0xa654d74bb3f30a02, 0x7b61b40fd575c46c, 0xe77323597ae1e83f, 0x4713a803185ad6b7,
    ],
    [
        0x82db66a74ea4e1d4, 0x9d492b103fdac5d8, 0x141ed0066ff65482, 0x3a8e3501b447cc55, 0x36953e0c0c483b1e, 0x543ee5d924fe8bb4,
        0x831ccdf38e38c52b, 0xd07466b32a311a72, 0xcada26ebbad287bb, 0xe2bbe3a3174da302, 0x653a69ff037cc063, 0xb4361ee3f9d7c37d,
    ],
    [
        0xaf67122d4106227b, 0x5313b11ccbcda9cd, 0xb2518866cf663f18, 0xc877a99e1a91b5fb, 0xdaac7365bca634c3, 0xc1c5cdce5709f6b1,
        0x217bebe61541e5b3, 0x18e77a7278812b9a, 0xc6e8da6362313dd3, 0x5584027dddb6e21b, 0x6f45c25fd87e99e0, 0xd5bd4df410bda27f,
    ],
    [
        0xf1b940efeb038c80, 0x022574f6111e3730, 0xf4952fbd504e3469, 0xf52e211e7570fcc2, 0x230a460421b69cbe, 0xa240f53c51db89c9,
        0xa98cc0598dd05a58, 0x0e8785d03794f6f8, 0xbb0d98cdede34f7b, 0xd5361514fd693303, 0xf72a211de2ad9bf9, 0xb8a26b9e1033f13a,
    ],
    [
        0x981e79a9eaa70d7d, 0xf1d308add211c4f6, 0x6582b7237a9fd689, 0xd292dc2f56a08709, 0xb095940547ed789e, 0x0521056e281ecf43,
        0x05a970b47e752f2e, 0xa214b824dafa5494, 0xcbf04044fe558b00, 0x59f28b34114ee0f5, 0x89fbc9114fcc29e7, 0x4d01375dd86365ba,
    ],
    [
        0x7798a16c242fd27c, 0xd5ab45050e2da713, 0x285a1a3ccfee8b2b, 0x89b86b6cbbd26a53, 0x5c57652dd5ce4c40, 0x3644830240012edb,
        0x702bde721cdac823, 0x7ef1b4eb3651f6ab, 0x086684921007f4ff, 0x653cf8f392901b4e, 0xe7285075ec3d1b96, 0x745378aab516f46e,
    ],
    [
        0xef0a300385aec7e9, 0x1cc8e04f8c9c6c99, 0x1e59fb9e304410c6, 0x6ecb31b2e9004faf, 0xd548334a85210f16, 0xa51916b0ffc57efc,
        0x932c4249271f6c19, 0xbb58ffaef8051166, 0xcad3690fa54c18b6, 0x133cf768c90b34ff, 0xc00a6784becae804, 0xa1ed250ffbace781,
    ],
    [
        0x5bb2d7479d9c7362, 0x679e23d49200c883, 0x03e8561b854aed2b, 0x765dd0c5ae56a9ca, 0x9b99a2157b8800c2, 0xaab98297d9d8326b,
        0x938a161474e18c5d, 0x5d1470bca684d98f, 0x449594ef211e1c18, 0x30804669bbd972bc, 0x7998673171bbe18e, 0x49b8494adf0361cd,
    ],
    [
        0x403510a0d96bd099, 0xee63c8ca896915a1, 0x3fa1e94c1d080dda, 0x210f86367fa3fcda, 0xb2b64b0e2d15259b, 0xdbaf8b63f2839839,
        0x0b0d61ad7f0d349e, 0xc951e5e63355ca82, 0x334bec1ce19171c1, 0x9df22277e5f707fd, 0xaa7c8b991d5f29f5, 0xa72efa4493ad8e29,
    ],
    [
        0xd5d6860b8d5dd145, 0x7ab16756315e0e99, 0xb0718830a9038b3c, 0x4c3a6509d11ac9fb, 0x51111dd3c73076b7, 0x3dc28a232a598785,
        0x3bbdd6ea8357f5b8, 0x97a1a44a70e55d7a, 0x3883d10eb857ad9b, 0xb544452d4d831607, 0x9e33848d2d298865, 0x6e1d429ab7dcefa9,
    ],
    [
        0xfba2d36b9829ee1b, 0x2069cfc3c58a96ae, 0xc9e523d98d18d75d, 0xf14515923c4733e0, 0x8ccbd90a4772be70, 0x470311fa8dbb60d0,
        0x65749a2180029a28, 0xf3cfbcfd02e4628d, 0xa3d03792ec02e2d8, 0x644f52ab5b0938dd, 0x3f39be4a4eb06854, 0xec621fda5731c2bb,
    ],
    [
        0xcfb29be90db28204, 0x2961764acdcb5eb2, 0xda65e2e61e67404b, 0x04ef80c759149298, 0xac04c8dd7c5ddc23, 0x44b808d168fb7dbb,
        0x03d7a260b86e4199, 0x3fe979fd0b384128, 0x27f265973721ac0f, 0x048cbeb4669bb633, 0xc9bb7d18b2536347, 0x7f17296bb500cac2,
    ],
    [
        0xdbf5b73a14e51179, 0x6c6ba04153fcbec9, 0x1793f171b39c5024, 0x875fff2b5c226226, 0x9880b1cf0b6a89ea, 0x93386d72cb6739da,
        0xcc42ba36771092c1, 0x11e427d823411eb6, 0x71c6aa4d3e0e5180, 0xff5b28922014fc0a, 0xf7b2db1d6a52d4fe, 0xeadc26a5a6323ab1,
    ],
    [
        0x151ccce17207bb98, 0x69baea59c4c307c5, 0x4f3aa574166d84dc, 0xade50fa4384b66f7, 0xb65610eb067c6c27, 0x75f351628a492b3c,
        0xe40ff0268c4bc0d9, 0xec6d06389d81cc71, 0x4bcb781b6364d998, 0x52d53c33a212d08f, 0x3978aedcd225ac02, 0x167e2b2798b13330,
    ],
    [
        0x27d8284c62c5f8f1, 0xcdd9550a707e89c5, 0x388dd626ed2fb0ab, 0x8b3edbd8717ff346, 0x3c4696a1d54317fe, 0xe2b62c00332ce998,
        0x35ae4b45b8738beb, 0xcabdb89f9f744578, 0xe15d3ec8d49ad056, 0x12cfdfc7ad6f5327, 0x4a72607afe7ff926, 0xd16df6fe9054be23,
    ],
    [
        0xd4d44df2f2ffb84e, 0x3a7f8bcb0811f4d7, 0xe4f8679214cbfb58, 0x28dc7d12cd6ca1e0, 0x732e44758858372b, 0x70576bdf6ba72a9d,
        0xcb20ec17ce6b19b9, 0x90ad80f328bf6c50, 0x57f85f9665dd59d9, 0xc548409fa0858601, 0xe7ce53b740ddd694, 0xe963f77cb30ba6ba,
    ],
    [
        0x3a7e121ef865fd98, 0x50b65fcf65f9a2f0, 0x8c8da37a0691e2e0, 0x6be6c132885c5f1e, 0xa5445c75ee345503, 0xe77dc081180ca3cd,
        0xb78e41753901d68c, 0x8a27462f4f4d1f90, 0xce5753bf41c9c7e4, 0x1dd00fe22aa5b42a, 0x674db5dfab68dd57, 0x40e4165d3b90d3f0,
    ],
    [
        0x18887b72c68766ec, 0x6b6df592fc4ed7de, 0x50d4895898061ff8, 0x6cd64d15701fed78, 0x02e3db2ba4ada0f1, 0x1aaa44c082002956,
        0xde0596b89e2c827c, 0x8941033beebf87c3, 0xd87b31a9b6634ff2, 0x584c5eab17895ea2, 0x86586001c1642fe6, 0x009ac6deaa64f9d9,
    ],
    [
        0xfc8d6b184baeeb3f, 0x783bd69104240c64, 0xebe7d98a8d417ccf, 0x3791c2fcea964776, 0x9673daf604d0babf, 0x35ae20cb8ac8ecff,
        0xc4e2ef2de1b2516f, 0x81b1af526bfe14b9, 0x9fd7c30069a22916, 0x5bc64242ea4a107f, 0xd4e6014e30690800, 0x1d84a362ffd27b38,
    ],
    [
        0x629e46542b59c614, 0xe9c54e561e5276f6, 0xa096ff9e22105907, 0xd0a30f555ac34d59, 0xa5f7297c8ff02969, 0xa1fc7d811a124356,
        0x72dcbc8da7c4d635, 0x3feaca2dcdd44beb, 0x25957b7a79f7e5a5, 0xe0ae54c2f89537ee, 0x1830fb53018bc840, 0xf321ea777f3aafb4,
    ],
    [
        0xd616ae2a24d4e0e1, 0x307ea5fc87cc2567, 0xb2f42fba067fdca8, 0x3c10553d1cfa3f73, 0xb95eae630b012e94, 0xc8d9d165077ccc45,
        0xcdff5e55224777cb, 0x5f4e55ad175fe513, 0x1c6a6f80a45f548d, 0x84b31d2ad1ce4a5f, 0x94e60c24536c6cb4, 0xa83751fa1c4afa9c,
    ],
    [
        0x465ead47abd19987, 0xcdb2b3b966032993, 0x4678c8485fd041df, 0xabc42b70d480cfd4, 0xc4ecf3bf0879b267, 0x6e3d91258e1e1040,
        0x75f1042db43cd8f1, 0xa688d5cd9be71144, 0x584d4d5f1a97280a, 0xa00455458e8f920b, 0xec62d4c6b97939f3, 0x68d6503e425c5593,
    ],
    [
        0x61cbd7b42e1d9322, 0x9e2b675212618899, 0x98760d9be327c2df, 0x2dad6f8da69545c9, 0x68fc9c26f83903e5, 0xe2b9c2e02ddee95a,
        0xf5381fd83400ea95, 0x364f10edd61ae9bd, 0x470466def8508501, 0xd18f009afdb33984, 0x8379af12c03f67f9, 0xd9b708bf485b3ac9,
    ],
    [
        0xe86198d299b36020, 0xfd1162e7c07abc54, 0x72e1804a7030b9f0, 0x0047ec21500cedc0, 0xe0f5aebfa0e54a2e, 0xb8ee3752c8e64678,
        0xd1fe3ec4e589ca4f, 0xb277980be409cd13, 0x015771af47dd2d89, 0x08bab6d33bbc8411, 0x2086c63fcbbbecc7, 0xbff861f40226b623,
    ],
    [
        0x5735f47846e661cc, 0x40728daf79f5ea86, 0xa0d485335842d244, 0x3b0dc7ce618cb820, 0xe455a63c6fec2569, 0xc62e869e06215295,
        0xc8b741c69b0dbbc8, 0xc8fcce6f498394a5, 0xc0d448070dfc5aac, 0xd74bf08cb71b428b, 0x03c01b73faecc60b, 0x126c765f4a9ed32e,
    ],
];

//WK, WQ, BK, BQ
#[rustfmt::skip]
const CASTLE_HASH: [u64; 4] = [
    0x6a1cf5e498eae033, 0x6aac61dc2cf2f40b, 0x743ca763de70e2d1, 0xf5052c6f0a2c1756
];

#[rustfmt::skip]
const ENPASSANT_FILE_HASH: [u64; 8] = [
    0x1dfbdce12e695571, 0x2a2aa72cc8a22849, 0xbe0d170a3ac7390b, 0xf20083dd53ab2a52,
    0xec2ff44e3391fd9a, 0x31c9290c9edf4efc, 0x370a9bccafc439ca, 0xe8bc3bc62b419caa,
];

#[rustfmt::skip]
const SIDE_HASH: [u64; 1] = [
    0x708def0327105062
];
