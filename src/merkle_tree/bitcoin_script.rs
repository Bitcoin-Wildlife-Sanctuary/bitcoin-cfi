use crate::treepp::*;
use crate::utils::{hash_felt_gadget, limb_to_be_bits_toaltstack};

/// Gadget for verifying a regular binary Merkle tree.
pub struct MerkleTreeGadget;

impl MerkleTreeGadget {
    pub(crate) fn query_and_verify_internal(logn: usize, is_sibling: bool) -> Script {
        script! {
            OP_DEPTH OP_1SUB OP_ROLL
            OP_DEPTH OP_1SUB OP_ROLL
            OP_DEPTH OP_1SUB OP_ROLL
            OP_DEPTH OP_1SUB OP_ROLL

            // copy-paste the 4 elements
            //     ABCD -> CDAB -> CDABAB -> ABABCD-> ABABCDCD
            //  -> ABCDCDAB -> ABCDABCD

            OP_2SWAP
            OP_2DUP
            OP_2ROT
            OP_2DUP
            OP_2ROT
            OP_2SWAP

            hash_felt_gadget

            if is_sibling {
                OP_DEPTH OP_1SUB OP_ROLL
                OP_FROMALTSTACK OP_NOTIF OP_SWAP OP_ENDIF
                OP_CAT OP_SHA256

                for _ in 1..logn {
                    OP_DEPTH OP_1SUB OP_ROLL
                    OP_FROMALTSTACK OP_IF OP_SWAP OP_ENDIF
                    OP_CAT OP_SHA256
                }
            } else {
                for _ in 0..logn {
                    OP_DEPTH OP_1SUB OP_ROLL
                    OP_FROMALTSTACK OP_IF OP_SWAP OP_ENDIF
                    OP_CAT OP_SHA256
                }
            }

            5 OP_ROLL
            OP_EQUALVERIFY
        }
    }

    /// Query and verify using the Merkle path as a hint.
    /// input:
    ///   root_hash
    ///   pos
    ///
    /// output:
    ///   v (qm31 -- 4 elements)
    pub fn query_and_verify(logn: usize) -> Script {
        script! {
            { limb_to_be_bits_toaltstack(logn as u32) }
            { Self::query_and_verify_internal(logn, false) }
        }
    }

    /// Query and verify using the Merkle path as a hint, but for its sibling instead.
    pub fn query_and_verify_sibling(logn: usize) -> Script {
        script! {
            { limb_to_be_bits_toaltstack(logn as u32) }
            { Self::query_and_verify_internal(logn, true) }
        }
    }
}

#[cfg(test)]
mod test {

    use crate::treepp::*;
    use crate::utils::get_rand_qm31;
    use crate::{
        merkle_tree::{MerkleTree, MerkleTreeGadget},
        tests_utils::report::report_bitcoin_script_size,
    };
    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaCha20Rng;
    use rust_bitcoin_m31::qm31_equalverify;

    #[test]
    fn test_merkle_tree_verify() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        for logn in 12..=20 {
            let verify_script = MerkleTreeGadget::query_and_verify(logn);

            report_bitcoin_script_size(
                "MerkleTree",
                format!("verify(2^{})", logn).as_str(),
                verify_script.len(),
            );

            let mut last_layer = vec![];
            for _ in 0..(1 << logn) {
                last_layer.push(get_rand_qm31(&mut prng));
            }

            let merkle_tree = MerkleTree::new(last_layer.clone());

            let mut pos: u32 = prng.gen();
            pos &= (1 << logn) - 1;

            let proof = merkle_tree.query(pos as usize);

            let script = script! {
                { proof }
                { merkle_tree.root_hash }
                { pos }
                { verify_script.clone() }
                { last_layer[pos as usize] }
                qm31_equalverify
                OP_TRUE
            };

            let exec_result = execute_script(script);
            assert!(exec_result.success);
        }
    }

    #[test]
    fn test_merkle_tree_verify_sibling() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        for logn in 12..=20 {
            let verify_script = MerkleTreeGadget::query_and_verify_sibling(logn);

            let mut last_layer = vec![];
            for _ in 0..(1 << logn) {
                last_layer.push(get_rand_qm31(&mut prng));
            }

            let merkle_tree = MerkleTree::new(last_layer.clone());

            let mut pos: u32 = prng.gen();
            pos &= (1 << logn) - 1;

            let proof = merkle_tree.query((pos ^ 1) as usize);

            let script = script! {
                { proof }
                { merkle_tree.root_hash }
                { pos }
                { verify_script.clone() }
                { last_layer[(pos ^ 1) as usize] }
                qm31_equalverify
                OP_TRUE
            };

            let exec_result = execute_script(script);
            assert!(exec_result.success);
        }
    }
}
