use crate::math::{ F64, FiniteField };
use crate::stark::{ StarkProof, TraceTable, TraceState, ConstraintCoefficients };
use crate::utils::{ uninit_vector, CopyInto };
use super::{ decoder::Decoder, stack::Stack, MAX_CONSTRAINT_DEGREE };

// TYPES AND INTERFACES
// ================================================================================================
pub struct Evaluator {
    decoder         : Decoder,
    stack           : Stack,

    coefficients    : ConstraintCoefficients<F64>,
    domain_size     : usize,
    extension_factor: usize,

    t_constraint_num: usize,
    t_degree_groups : Vec<(u64, Vec<usize>)>,
    t_evaluations   : Vec<Vec<u64>>,

    b_constraint_num: usize,
    program_hash    : [u64; 4],
    inputs          : Vec<u64>,
    outputs         : Vec<u64>,
    b_degree_adj    : u64,
}

// EVALUATOR IMPLEMENTATION
// ================================================================================================
impl Evaluator {

    pub fn from_trace(trace: &TraceTable, trace_root: &[u64; 4], inputs: &[u64], outputs: &[u64]) -> Evaluator {

        let stack_depth = trace.max_stack_depth();
        let program_hash = trace.get_program_hash();
        let trace_length = trace.unextended_length();
        let extension_factor = MAX_CONSTRAINT_DEGREE;
        let t_constraint_degrees = get_transition_constraint_degrees(stack_depth);

        // if we are in debug mode, initialize vectors to hold individual evaluations
        // of transition constraints
        let domain_size = trace_length * extension_factor;
        let t_evaluations = if cfg!(debug_assertions) {
            t_constraint_degrees[..].iter().map(|_| uninit_vector(domain_size)).collect()
        }
        else {
            Vec::new()
        };

        return Evaluator {
            decoder         : Decoder::new(trace_length, extension_factor),
            stack           : Stack::new(stack_depth),
            coefficients    : ConstraintCoefficients::new(trace_root),
            domain_size     : domain_size,
            extension_factor: extension_factor,
            t_constraint_num: t_constraint_degrees.len(),
            t_degree_groups : group_transition_constraints(t_constraint_degrees, trace_length),
            t_evaluations   : t_evaluations,
            b_constraint_num: inputs.len() + outputs.len() + program_hash.len(),
            program_hash    : program_hash,
            inputs          : inputs.to_vec(),
            outputs         : outputs.to_vec(),
            b_degree_adj    : get_boundary_constraint_adjustment_degree(trace_length),
        };
    }

    pub fn from_proof(proof: &StarkProof, program_hash: &[u8; 32], inputs: &[u64], outputs: &[u64]) -> Evaluator {
        
        let stack_depth = proof.stack_depth();
        let trace_length = proof.trace_length();
        let extension_factor = proof.options().extension_factor();
        let t_constraint_degrees = get_transition_constraint_degrees(stack_depth);

        return Evaluator {
            decoder         : Decoder::new(trace_length, extension_factor),
            stack           : Stack::new(stack_depth),
            coefficients    : ConstraintCoefficients::new(proof.trace_root()),
            domain_size     : proof.domain_size(),
            extension_factor: extension_factor,
            t_constraint_num: t_constraint_degrees.len(),
            t_degree_groups : group_transition_constraints(t_constraint_degrees, trace_length),
            t_evaluations   : Vec::new(),
            b_constraint_num: inputs.len() + outputs.len() + program_hash.len(),
            program_hash    : program_hash.copy_into(),
            inputs          : inputs.to_vec(),
            outputs         : outputs.to_vec(),
            b_degree_adj    : get_boundary_constraint_adjustment_degree(trace_length),
        };
    }

    pub fn constraint_count(&self) -> usize {
        return self.t_constraint_num + self.b_constraint_num;
    }

    pub fn domain_size(&self) -> usize {
        return self.domain_size;
    }

    pub fn trace_length(&self) -> usize {
        return self.domain_size / self.extension_factor;
    }

    pub fn extension_factor(&self) -> usize {
        return self.extension_factor;
    }

    pub fn transition_evaluations(&self) -> &Vec<Vec<u64>> {
        return &self.t_evaluations;
    }

    pub fn get_x_at_last_step(&self) -> u64 {
        let trace_root = F64::get_root_of_unity(self.trace_length());
        return F64::exp(trace_root, (self.trace_length() - 1) as u64);
    }

    // CONSTRAINT EVALUATORS
    // -------------------------------------------------------------------------------------------

    /// Computes pseudo-random linear combination of transition constraints D_i at point x as:
    /// cc_{i * 2} * D_i + cc_{i * 2 + 1} * D_i * x^p for all i, where cc_j are the coefficients
    /// used in the linear combination and x^p is a degree adjustment factor (different for each degree).
    pub fn evaluate_transition(&self, current: &TraceState<F64>, next: &TraceState<F64>, x: u64, step: usize) -> u64 {
        
        // evaluate transition constraints
        let mut evaluations = vec![0; self.t_constraint_num];
        self.decoder.evaluate(&current, &next, step, &mut evaluations);
        self.stack.evaluate(&current, &next, &mut evaluations[self.decoder.constraint_count()..]);

        // when in debug mode, save transition evaluations before they are combined
        #[cfg(debug_assertions)]
        self.save_transition_evaluations(&evaluations, step);

        // if the constraints should evaluate to all zeros at this step,
        // make sure they do, and return
        if self.should_evaluate_to_zero_at(step) {
            let step = step / self.extension_factor;
            for i in 0..evaluations.len() {
                assert!(evaluations[i] == 0, "transition constraint at step {} didn't evaluate to 0", step);
            }
            return 0;
        }

        // compute a pseudo-random linear combination of all transition constraints
        return self.combine_transition_constraints(&evaluations, x);
    }

    /// Computes pseudo-random liner combination of transition constraints at point x. This function
    /// is similar to the one above but it can also be used to evaluate constraints at any point
    /// in the filed (not just in the evaluation domain). However, it is also much slower.
    pub fn evaluate_transition_at(&self, current: &TraceState<F64>, next: &TraceState<F64>, x: u64) -> u64 {
        // evaluate transition constraints
        let mut evaluations = vec![0; self.t_constraint_num];
        self.decoder.evaluate_at(&current, &next, x, &mut evaluations);
        self.stack.evaluate(&current, &next, &mut evaluations[self.decoder.constraint_count()..]);

        // compute a pseudo-random linear combination of all transition constraints
        return self.combine_transition_constraints(&evaluations, x);
    }

    /// Computes pseudo-random linear combination of boundary constraints B_i at point x 
    /// separately for input and output constraints; the constraints are computed as:
    /// cc_{i * 2} * B_i + cc_{i * 2 + 1} * B_i * x^p for all i, where cc_j are the coefficients
    /// used in the linear combination and x^p is a degree adjustment factor.
    pub fn evaluate_boundaries(&self, current: &TraceState<F64>, x: u64) -> (u64, u64) {
        
        let cc = self.coefficients.inputs;
        let stack = current.get_stack();
        
        // compute adjustment factor
        let xp = F64::exp(x, self.b_degree_adj);

        // 1 ----- compute combination of input constraints ---------------------------------------
        let mut i_result = 0;
        let mut result_adj = 0;

        // separately compute P(x) - input for adjusted and un-adjusted terms
        for i in 0..self.inputs.len() {
            let val = F64::sub(stack[i], self.inputs[i]);
            i_result = F64::add(i_result, F64::mul(val, cc[i * 2]));
            result_adj = F64::add(result_adj, F64::mul(val, cc[i * 2 + 1]));
        }

        // raise the degree of adjusted terms and sum all the terms together
        i_result = F64::add(i_result, F64::mul(result_adj, xp));

        // 2 ----- compute combination of output constraints ---------------------------------------
        let mut f_result = 0;
        let mut result_adj = 0;

        // separately compute P(x) - output for adjusted and un-adjusted terms
        for i in 0..self.outputs.len() {
            let val = F64::sub(stack[i], self.outputs[i]);
            f_result = F64::add(f_result, F64::mul(val, cc[i * 2]));
            result_adj = F64::add(result_adj, F64::mul(val, cc[i * 2 + 1]));
        }

        // raise the degree of adjusted terms and sum all the terms together
        f_result = F64::add(f_result, F64::mul(result_adj, xp));

        // 3 ----- compute combination of program hash constraints --------------------------------
        let mut result_adj = 0;

        // because we check program hash at the last step, we add the constraints to the
        // constraint evaluations to the output constraint combination
        let program_hash = current.get_program_hash();
        for i in 0..self.program_hash.len() {
            let val = F64::sub(program_hash[i], self.program_hash[i]);
            f_result = F64::add(f_result, F64::mul(val, cc[i * 2]));
            result_adj = F64::add(result_adj, F64::mul(val, cc[i * 2 + 1]));
        }

        f_result = F64::add(f_result, F64::mul(result_adj, xp));

        return (i_result, f_result);
    }

    // HELPER METHODS
    // -------------------------------------------------------------------------------------------
    fn should_evaluate_to_zero_at(&self, step: usize) -> bool {
        return (step & (self.extension_factor - 1) == 0) // same as: step % extension_factor == 0
            && (step != self.domain_size - self.extension_factor);
    }

    fn combine_transition_constraints(&self, evaluations: &Vec<u64>, x: u64) -> u64 {
        let cc = self.coefficients.transition;
        let mut result = 0;

        let mut i = 0;
        for (incremental_degree, constraints) in self.t_degree_groups.iter() {

            // for each group of constraints with the same degree, separately compute
            // combinations of D(x) and D(x) * x^p
            let mut result_adj = 0;
            for &constraint_idx in constraints.iter() {
                let evaluation = evaluations[constraint_idx];
                result = F64::add(result, F64::mul(evaluation, cc[i * 2]));
                result_adj = F64::add(result_adj, F64::mul(evaluation, cc[i * 2 + 1]));
                i += 1;
            }

            // increase the degree of D(x) * x^p
            let xp = F64::exp(x, *incremental_degree);
            result = F64::add(result, F64::mul(result_adj, xp));
        }

        return result;
    }

    fn save_transition_evaluations(&self, evaluations: &[u64], step: usize) {
        unsafe {
            let mutable_self = &mut *(self as *const _ as *mut Evaluator);
            for i in 0..evaluations.len() {
                mutable_self.t_evaluations[i][step] = evaluations[i];
            }
        }
    }
}

// HELPER FUNCTIONS
// ================================================================================================
fn get_transition_constraint_degrees(stack_depth: usize) -> Vec<usize> {
    return [
        Decoder::constraint_degrees(),
        Stack::constraint_degrees(stack_depth)
    ].concat();
}

fn group_transition_constraints(degrees: Vec<usize>, trace_length: usize) -> Vec<(u64, Vec<usize>)> {
    let mut groups = [
        Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(),
        Vec::new(), Vec::new(), Vec::new(), Vec::new(),
    ];

    for (i, &degree) in degrees.iter().enumerate() {
        groups[degree].push(i);
    }

    let target_degree = get_transition_constraint_target_degree(trace_length);

    let mut result = Vec::new();
    for (degree, constraints) in groups.iter().enumerate() {
        if constraints.len() == 0 { continue; }
        let constraint_degree = (trace_length - 1) * degree;    
        let incremental_degree = (target_degree - constraint_degree) as u64;
        result.push((incremental_degree, constraints.clone()));
    }

    return result;
}

fn get_boundary_constraint_adjustment_degree(trace_length: usize) -> u64 {
    let target_degree = get_boundary_constraint_target_degree(trace_length);
    let boundary_constraint_degree = trace_length - 1;
    return (target_degree - boundary_constraint_degree) as u64;
}

/// target degree for boundary constraints is set so that when divided by boundary
/// constraint divisor (degree 1 polynomial), the degree will be equal to
/// deg(combination domain) - deg(trace)
fn get_boundary_constraint_target_degree(trace_length: usize) -> usize {
    let combination_degree = (MAX_CONSTRAINT_DEGREE - 1) * trace_length;
    let divisor_degree = 1;
    return combination_degree + divisor_degree;
}

/// target degree for transition constraints is set so when divided transition 
/// constraint divisor (deg(trace) - 1 polynomial), the degree will be equal to
/// deg(combination domain) - deg(trace)
fn get_transition_constraint_target_degree(trace_length: usize) -> usize {
    let combination_degree = (MAX_CONSTRAINT_DEGREE - 1) * trace_length;
    let divisor_degree = trace_length - 1;
    return combination_degree + divisor_degree;
}