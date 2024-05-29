use crate::channel_commit::Commitment;
use crate::channel_extract::{Extraction5M31, ExtractionQM31, Extractor};
use crate::utils::trim_m31;
use sha2::{Digest, Sha256};
use stwo_prover::core::fields::qm31::QM31;

mod bitcoin_script;
pub use bitcoin_script::*;

/// A channel.
pub struct Channel {
    /// Current state of the channel.
    pub state: [u8; 32],
}

impl Channel {
    /// Initialize a new channel.
    pub fn new(hash: [u8; 32]) -> Self {
        Self { state: hash }
    }

    /// Absorb a commitment.
    pub fn absorb_commitment(&mut self, commitment: &Commitment) {
        let mut hasher = Sha256::new();
        Digest::update(&mut hasher, commitment.0);
        Digest::update(&mut hasher, self.state);
        self.state.copy_from_slice(hasher.finalize().as_slice());
    }

    /// Absorb a qm31 element.
    pub fn absorb_qm31(&mut self, el: &QM31) {
        let mut hasher = Sha256::new();
        Digest::update(&mut hasher, Commitment::commit_qm31(*el).0);
        Digest::update(&mut hasher, self.state);
        self.state.copy_from_slice(hasher.finalize().as_slice());
    }

    /// Draw one qm31 and compute the hints.
    pub fn draw_qm31(&mut self) -> (QM31, ExtractionQM31) {
        let mut extract = [0u8; 32];

        let mut hasher = Sha256::new();
        Digest::update(&mut hasher, self.state);
        Digest::update(&mut hasher, [0u8]);
        extract.copy_from_slice(hasher.finalize().as_slice());

        let mut hasher = Sha256::new();
        Digest::update(&mut hasher, self.state);
        self.state.copy_from_slice(hasher.finalize().as_slice());

        Extractor::extract_qm31(&extract)
    }

    /// Draw five queries and compute the hints.
    pub fn draw_5queries(&mut self, logn: usize) -> ([usize; 5], Extraction5M31) {
        let mut extract = [0u8; 32];

        let mut hasher = Sha256::new();
        Digest::update(&mut hasher, self.state);
        Digest::update(&mut hasher, [0u8]);
        extract.copy_from_slice(hasher.finalize().as_slice());

        let mut hasher = Sha256::new();
        Digest::update(&mut hasher, self.state);
        self.state.copy_from_slice(hasher.finalize().as_slice());

        let mut res = Extractor::extract_5m31(&extract);
        for v in res.0.iter_mut() {
            v.0 = trim_m31(v.0, logn);
        }

        (
            [
                res.0[0].0 as usize,
                res.0[1].0 as usize,
                res.0[2].0 as usize,
                res.0[3].0 as usize,
                res.0[4].0 as usize,
            ],
            res.1,
        )
    }
}
