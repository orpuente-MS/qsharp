// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use qsc_hir::{
    hir::{Item, ItemKind},
    ty::{Prim, Ty},
};

/// If the item is a callable, returns the information that would
/// be needed to generate a circuit for it.
///
/// If the item is not a callable, returns `None`.
/// If the callable takes any non-qubit parameters, returns `None`.
///
/// If the callable only takes qubit parameters, (including qubit arrays):
///
/// The first element of the return tuple is a vector,
/// where each element corresponds to a parameter, and the
/// value is the number of dimensions of the parameter.
///
/// For example, for input parameters
/// `(Qubit, Qubit[][], Qubit[])` the parameter info is `vec![0, 2, 1]`.
///
/// The second element of the return tuple is the total number of qubits that would
/// need be allocated to run this operation for the purposes of circuit generation.
#[must_use]
pub fn qubit_param_info(item: &Item) -> Option<(Vec<u32>, u32)> {
    if let ItemKind::Callable(decl) = &item.kind {
        let (qubit_param_dimensions, total_num_qubits) = get_qubit_param_info(&decl.input.ty);

        if !qubit_param_dimensions.is_empty() {
            return Some((qubit_param_dimensions, total_num_qubits));
        }
    }
    None
}

/// Returns an entry expression to directly invoke the operation
/// for the purposes of generating a circuit for it.
///
/// `operation_expr` is the source for the expression that refers to the operation,
/// e.g. "Test.Foo" or "qs => H(qs[0])".
///
/// If the item is not a callable, returns `None`.
/// If the callable takes any non-qubit parameters, returns `None`.
#[must_use]
pub fn entry_expr_for_qubit_operation(item: &Item, operation_expr: &str) -> Option<String> {
    if let Some((qubit_param_dimensions, total_num_qubits)) = qubit_param_info(item) {
        return Some(operation_circuit_entry_expr(
            operation_expr,
            &qubit_param_dimensions,
            total_num_qubits,
        ));
    }
    None
}

/// Generates the entry expression to call the operation described by `params`.
/// The expression allocates qubits and invokes the operation.
#[must_use]
fn operation_circuit_entry_expr(
    operation_expr: &str,
    qubit_param_dimensions: &[u32],
    total_num_qubits: u32,
) -> String {
    let alloc_qubits = format!("use qs = Qubit[{total_num_qubits}];");

    let mut qs_start = 0;
    let mut call_args = vec![];
    for dim in qubit_param_dimensions {
        let dim = *dim;
        let qs_len = NUM_QUBITS.pow(dim);
        // Q# ranges are end-inclusive
        let qs_end = qs_start + qs_len - 1;
        if dim == 0 {
            call_args.push(format!("qs[{qs_start}]"));
        } else {
            // Array argument - use a range to index
            let mut call_arg = format!("qs[{qs_start}..{qs_end}]");
            for _ in 1..dim {
                // Chunk the array for multi-dimensional array arguments
                call_arg = format!("Microsoft.Quantum.Arrays.Chunks({NUM_QUBITS}, {call_arg})");
            }
            call_args.push(call_arg);
        }
        qs_start = qs_end + 1;
    }

    let call_args = call_args.join(", ");

    // We don't reset the qubits since we don't want reset gates
    // included in circuit output.
    // We also don't measure the qubits but we have to return a result
    // array to satisfy Base Profile.
    format!(
        r#"{{
            {alloc_qubits}
            ({operation_expr})({call_args});
            let r: Result[] = [];
            r
        }}"#
    )
}

/// The number of qubits to allocate for each qubit array
/// in the operation arguments.
const NUM_QUBITS: u32 = 2;

fn get_qubit_param_info(input: &Ty) -> (Vec<u32>, u32) {
    match input {
        Ty::Prim(Prim::Qubit) => return (vec![0], 1),
        Ty::Array(ty) => {
            if let Some(element_dim) = get_array_dimension(ty) {
                let dim = element_dim + 1;
                return (vec![dim], NUM_QUBITS.pow(dim));
            }
        }
        Ty::Tuple(tys) => {
            let params = tys.iter().map(get_array_dimension).collect::<Vec<_>>();

            if params.iter().all(Option::is_some) {
                return params.into_iter().map(Option::unwrap).fold(
                    (vec![], 0),
                    |(mut dims, mut total_qubits), dim| {
                        dims.push(dim);
                        total_qubits += NUM_QUBITS.pow(dim);
                        (dims, total_qubits)
                    },
                );
            }
        }
        _ => {}
    }
    (vec![], 0)
}

/// If `Ty` is a qubit or a qubit array, returns the number of dimensions of the array.
/// A qubit is considered to be a 0-dimensional array.
/// For example, for a `Qubit` it returns `Some(0)`, for a `Qubit[][]` it returns `Some(2)`.
/// For a non-qubit type, returns `None`.
fn get_array_dimension(input: &Ty) -> Option<u32> {
    match input {
        Ty::Prim(Prim::Qubit) => Some(0),
        Ty::Array(ty) => get_array_dimension(ty).map(|d| d + 1),
        _ => None,
    }
}
