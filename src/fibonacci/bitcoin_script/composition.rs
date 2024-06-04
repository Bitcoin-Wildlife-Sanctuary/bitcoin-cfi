use crate::constraints::ConstraintsGadget;
use crate::treepp::*;
use num_traits::One;
use rust_bitcoin_m31::{
    qm31_add, qm31_copy, qm31_dup, qm31_equalverify, qm31_from_bottom, qm31_fromaltstack, qm31_mul,
    qm31_mul_m31, qm31_roll, qm31_square, qm31_sub, qm31_swap, qm31_toaltstack,
};
use stwo_prover::core::circle::{CirclePoint, Coset};
use stwo_prover::core::fields::m31::M31;
use stwo_prover::core::fields::qm31::QM31;
use stwo_prover::core::fields::FieldExpOps;
use stwo_prover::examples::fibonacci::Fibonacci;

/// Gadget for Fibonacci composition polynomial-related operations.
pub struct FibonacciCompositionGadget;

impl FibonacciCompositionGadget {
    ///Hint
    #[allow(dead_code)]
    fn step_constraint_eval_quotient_by_mask_hint(
        log_size: u32,
        claim: M31,
        z: CirclePoint<QM31>,
        fz: QM31,
        fgz: QM31,
        fggz: QM31,
    ) -> Script {
        let fib = Fibonacci::new(log_size, claim);

        script! {
            { fib.air.component.step_constraint_eval_quotient_by_mask(z, &[fz,fgz,fggz]) }
        }
    }

    ///Computes the step constraint f(z)^2 + f(G z)^2 - f(G^2 z)
    ///hint:
    /// num/denom
    ///input:
    /// f(G^2 z)
    /// f(Gz)
    /// f(z)
    /// z.x
    /// z.y
    ///output:
    /// num/denom
    #[allow(dead_code)]
    fn step_constraint_eval_quotient_by_mask(log_size: u32) -> Script {
        let constraint_zero_domain = Coset::subgroup(log_size);

        script! {
            { qm31_copy(1) }
            { qm31_copy(1) }
            qm31_toaltstack
            qm31_toaltstack
            qm31_toaltstack
            qm31_toaltstack

            qm31_square
            qm31_swap
            qm31_square
            qm31_add

            qm31_swap
            qm31_sub // f(z)^2 + f(G z)^2 - f(G^2 z)

            qm31_fromaltstack
            qm31_fromaltstack
            {
                ConstraintsGadget::pair_vanishing(
                    constraint_zero_domain
                        .at(constraint_zero_domain.size() - 2)
                        .into_ef(),
                    constraint_zero_domain
                        .at(constraint_zero_domain.size() - 1)
                        .into_ef()
                )
            }
            qm31_mul // num

            qm31_fromaltstack
            qm31_fromaltstack
            { ConstraintsGadget::coset_vanishing(constraint_zero_domain) } // denom

            qm31_from_bottom // num/denom
            qm31_dup
            qm31_toaltstack

            qm31_mul // denom*(num/denom)

            qm31_equalverify
            qm31_fromaltstack // num/denom

        }
    }

    ///Hint
    #[allow(dead_code)]
    fn boundary_constraint_eval_quotient_by_mask_hint(
        log_size: u32,
        claim: M31,
        z: CirclePoint<QM31>,
        fz: QM31,
    ) -> Script {
        let fib = Fibonacci::new(log_size, claim);

        let res = fib
            .air
            .component
            .boundary_constraint_eval_quotient_by_mask(z, &[fz]);

        script! {
            { res }
        }
    }

    /// Computes the bonudary constraint f(0)=1, f(end)=claim
    /// hint:
    ///  num/denom
    /// input:
    ///  f(z)
    ///  z.x
    ///  z.y
    /// output:
    ///  num/denom
    #[allow(dead_code)]
    fn boundary_constraint_eval_quotient_by_mask(log_size: u32, claim: M31) -> Script {
        let constraint_zero_domain = Coset::subgroup(log_size);
        let p = constraint_zero_domain.at(constraint_zero_domain.size() - 1);
        script! {
            qm31_dup
            qm31_toaltstack
            { qm31_roll(1) }
            qm31_toaltstack //stack: f(z), z.y; altstack: z.y, z.x

            { (claim - M31::one()) * p.y.inverse() }
            qm31_mul_m31

            { QM31::one() }
            qm31_add //linear = QM31::one() + z.y * (self.claim - M31::one()) * p.y.inverse();

            qm31_sub //num = f(z) - linear

            qm31_fromaltstack //bring back z.x from altstack
            qm31_fromaltstack //bring back z.y from altstack
            { ConstraintsGadget::pair_vanishing(p.into_ef(), CirclePoint::zero())} //denom

            qm31_from_bottom //pull num/denom from hint

            qm31_dup
            qm31_toaltstack //store num/denom in altstack

            qm31_mul //(num/denom)*denom

            qm31_equalverify //check that num==(num/denom)*denom

            qm31_fromaltstack //return num/denom
        }
    }

    ///Hint
    #[allow(dead_code)]
    fn eval_composition_polynomial_at_point_hint(
        log_size: u32,
        claim: M31,
        z: CirclePoint<QM31>,
        fz: QM31,
        fgz: QM31,
        fggz: QM31,
    ) -> Script {
        script! {
            { Self::boundary_constraint_eval_quotient_by_mask_hint(log_size, claim, z, fz) }
            { Self::step_constraint_eval_quotient_by_mask_hint(log_size, claim, z, fz, fgz, fggz) }
        }
    }

    ///Computes the composition polynomial of Fibonacci
    ///input:
    /// alpha
    /// f(G^2 z)
    /// f(Gz)
    /// f(z) (QM31)
    /// z.x
    /// z.y
    ///output:
    /// alpha*step_constraint(f(z),f(Gz),f(G^2 z),z) + boundary_constraint(f(z),z,claim)
    #[allow(dead_code)]
    fn eval_composition_polynomial_at_point(log_size: u32, claim: M31) -> Script {
        script! {
            { qm31_copy(2) }
            { qm31_copy(2) }
            { qm31_copy(2) }
            { Self::boundary_constraint_eval_quotient_by_mask(log_size,claim) }
            qm31_toaltstack

            { Self::step_constraint_eval_quotient_by_mask(log_size) }
            qm31_mul

            qm31_fromaltstack
            qm31_add
        }
    }
}

#[cfg(test)]
mod test {
    use itertools::Itertools;
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;
    use rust_bitcoin_m31::qm31_equalverify;
    use std::iter::zip;
    use stwo_prover::core::fields::m31::M31;
    use stwo_prover::{
        core::{
            air::{AirExt, ComponentTrace},
            circle::CirclePoint,
            poly::circle::CanonicCoset,
            ComponentVec,
        },
        examples::fibonacci::Fibonacci,
    };

    use crate::fibonacci::bitcoin_script::composition::FibonacciCompositionGadget;
    use crate::tests_utils::report::report_bitcoin_script_size;
    use crate::treepp::*;
    use crate::utils::get_rand_qm31;

    #[test]
    fn test_eval_composition_polynomial_at_point() {
        let log_size = 5;
        let claim = M31::from_u32_unchecked(443693538);

        let fib = Fibonacci::new(log_size, claim);
        let trace = fib.get_trace();
        let trace_poly = trace.interpolate();
        let trace_eval =
            trace_poly.evaluate(CanonicCoset::new(trace_poly.log_size() + 1).circle_domain());
        let trace = ComponentTrace::new(vec![&trace_poly], vec![&trace_eval]);

        let component_traces = vec![trace];

        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let composition_polynomial_script =
            FibonacciCompositionGadget::eval_composition_polynomial_at_point(log_size, claim);
        report_bitcoin_script_size(
            "Fibonacci",
            format!(
                "eval_composition_polynomial_at_point(log_size={})",
                log_size
            )
            .as_str(),
            composition_polynomial_script.len(),
        );

        for _ in 0..20 {
            let random_coeff = get_rand_qm31(&mut prng);

            let z = CirclePoint {
                x: get_rand_qm31(&mut prng),
                y: get_rand_qm31(&mut prng),
            };

            let points = fib.air.mask_points(z);
            let comp = zip(&component_traces[0].polys, &points[0])
                .map(|(poly, points)| {
                    points
                        .iter()
                        .map(|point| poly.eval_at_point(*point))
                        .collect_vec()
                })
                .collect_vec();

            let mut mask_values = ComponentVec(Vec::new());
            mask_values.push(comp.clone());

            let res = fib
                .air
                .eval_composition_polynomial_at_point(z, &mask_values, random_coeff);

            let script = script! {
                { FibonacciCompositionGadget::eval_composition_polynomial_at_point_hint(log_size, claim, z, comp[0][0], comp[0][1], comp[0][2]) } //hint
                { random_coeff }
                { comp[0][2] }
                { comp[0][1] }
                { comp[0][0] }
                { z.x }
                { z.y }
                { composition_polynomial_script.clone() }
                { res }
                qm31_equalverify
                OP_TRUE
            };
            let exec_result = execute_script(script);
            assert!(exec_result.success);
        }
    }

    #[test]
    fn test_boundary_constraint_eval_quotient_by_mask() {
        let log_size = 5;
        let claim = M31::from_u32_unchecked(443693538);
        let fib = Fibonacci::new(log_size, claim);

        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let boundary_constraint_script =
            FibonacciCompositionGadget::boundary_constraint_eval_quotient_by_mask(log_size, claim);
        report_bitcoin_script_size(
            "Fibonacci",
            format!(
                "boundary_constraint_eval_quotient_by_mask(log_size={})",
                log_size
            )
            .as_str(),
            boundary_constraint_script.len(),
        );

        for _ in 0..20 {
            let z = CirclePoint {
                x: get_rand_qm31(&mut prng),
                y: get_rand_qm31(&mut prng),
            };

            let fz = get_rand_qm31(&mut prng);

            let res = fib
                .air
                .component
                .boundary_constraint_eval_quotient_by_mask(z, &[fz]);

            let script = script! {
                { FibonacciCompositionGadget::boundary_constraint_eval_quotient_by_mask_hint(log_size, claim, z, fz) } //hint
                { fz }
                { z.x }
                { z.y }
                { boundary_constraint_script.clone() }
                { res }
                qm31_equalverify
                OP_TRUE
            };
            let exec_result = execute_script(script);
            assert!(exec_result.success);
        }
    }

    #[test]
    fn test_step_constraint_eval_quotient_by_mask() {
        let log_size = 5;
        let claim = M31::from_u32_unchecked(443693538);
        let fib = Fibonacci::new(log_size, claim);

        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let step_constraint_script =
            FibonacciCompositionGadget::step_constraint_eval_quotient_by_mask(log_size);
        report_bitcoin_script_size(
            "Fibonacci",
            format!(
                "step_constraint_eval_quotient_by_mask(log_size={})",
                log_size
            )
            .as_str(),
            step_constraint_script.len(),
        );

        for _ in 0..20 {
            let z = CirclePoint {
                x: get_rand_qm31(&mut prng),
                y: get_rand_qm31(&mut prng),
            };

            let fz = get_rand_qm31(&mut prng);

            let fgz = get_rand_qm31(&mut prng);

            let fggz = get_rand_qm31(&mut prng);

            let res = fib
                .air
                .component
                .step_constraint_eval_quotient_by_mask(z, &[fz, fgz, fggz]);

            let script = script! {
                { FibonacciCompositionGadget::step_constraint_eval_quotient_by_mask_hint(log_size, claim, z, fz, fgz, fggz) } //hint
                { fggz }
                { fgz }
                { fz }
                { z.x }
                { z.y }
                { step_constraint_script.clone() }
                { res }
                qm31_equalverify
                OP_TRUE
            };
            let exec_result = execute_script(script);
            assert!(exec_result.success);
        }
    }
}
