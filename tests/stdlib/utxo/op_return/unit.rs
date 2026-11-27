use crate::common::case::DispatchCase;
use crate::common::core::Expect::AssertFailed;
use crate::utxo::op_return::case;
use crate::utxo::op_return::case::{
    empty_op_return_output, hash_of_pushed_data, negate_or_reserved, null_data_entries,
    op_x_minus_1, output_null_datum, pushed_data, reference, reference_tx,
};

const DEFAULT_DATA: &[u8] = &[0, 1];
const OP_PUSHNUM_NEG1: &[u8] = &[0x01, 0x4f];
const OP_RESERVED: &[u8] = &[0x01, 0x50];
const OP_1: &[u8] = &[0x01, 0x51];
const OP_1_TO_16: &[u8] = &[
    0x01, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5a, 0x5b, 0x5c, 0x5d, 0x5e, 0x5f,
    0x60,
];

const NONEXISTENT_OUTPUT: u32 = u32::MAX;
const OUT_OF_RANGE_ENTRY: u32 = u32::MAX;

// 0. get_entry_option
#[test]
fn get_entry_option() {
    // 0 = push data (0x12, 0x23), 1 = OP_1, 2 = OP_1NEGATE, 3 = OP_RESERVED.
    const MIXED_ENTRIES: &[u8] = &[1, 0x02, 0x12, 0x23, 0x51, 0x4f, 0x50];
    const PUSHED_DATA: &[u8] = &[0x12, 0x23];

    let entries = [
        pushed_data(0, hash_of_pushed_data(PUSHED_DATA)),
        op_x_minus_1(0),
        negate_or_reserved(0),
        negate_or_reserved(1),
    ];

    for (entry_index, entry) in (0..).zip(entries) {
        case::get_entry_option(0, entry_index, Some(entry))
            .op_return(MIXED_ENTRIES)
            .accepts();
    }
}

#[test]
fn get_entry_option_nonexistent_output() {
    case::get_entry_option(NONEXISTENT_OUTPUT, 0, None)
        .op_return(DEFAULT_DATA)
        .accepts();
}

#[test]
fn get_entry_option_not_op_return_output() {
    case::get_entry_option(0, 0, None).accepts();
    case::get_entry_option(1, 0, None)
        .op_return(DEFAULT_DATA)
        .accepts();
}

#[test]
fn get_entry_option_out_of_range_entry() {
    case::get_entry_option(0, 1, None)
        .op_return(DEFAULT_DATA)
        .accepts();
    case::get_entry_option(0, OUT_OF_RANGE_ENTRY, None)
        .op_return(DEFAULT_DATA)
        .accepts();
}

#[test]
fn get_entry_option_rejects_wrong_expectation() {
    case::get_entry_option(0, 0, None)
        .op_return(DEFAULT_DATA)
        .rejects(AssertFailed);
    case::get_entry_option(0, 0, Some(op_x_minus_1(0)))
        .op_return(DEFAULT_DATA)
        .rejects(AssertFailed);
}

// 1. pushed_data_from_entry
#[test]
fn pushed_data_from_entry() {
    let hash = hash_of_pushed_data(&DEFAULT_DATA[1..]);

    for indicator in 0..4 {
        case::pushed_data_from_entry(pushed_data(indicator, hash), Some((indicator, hash)))
            .accepts();
    }
}

#[test]
fn pushed_data_from_entry_other_entries_return_none() {
    case::pushed_data_from_entry(op_x_minus_1(0), None).accepts();
    case::pushed_data_from_entry(negate_or_reserved(0), None).accepts();
    case::pushed_data_from_entry(negate_or_reserved(1), None).accepts();
}

#[test]
fn pushed_data_from_entry_rejects_wrong_expectation() {
    let hash = hash_of_pushed_data(&DEFAULT_DATA[1..]);

    // Wrong indicator, wrong hash, and `None` for pushed data.
    case::pushed_data_from_entry(pushed_data(0, hash), Some((1, hash))).rejects(AssertFailed);
    case::pushed_data_from_entry(pushed_data(0, hash), Some((0, [0; 32]))).rejects(AssertFailed);
    case::pushed_data_from_entry(pushed_data(0, hash), None).rejects(AssertFailed);
}

// 2. get_pushed_data
#[test]
fn get_pushdata0() {
    let hash = hash_of_pushed_data(&DEFAULT_DATA[1..]);

    case::get_pushed_data(0, 0, Some((0, hash)))
        .op_return(DEFAULT_DATA)
        .accepts();
}

#[test]
fn get_pushdata1() {
    // A 76-byte push, which needs OP_PUSHDATA1.
    const DATA: &[u8] = &[0; 77];

    let hash = hash_of_pushed_data(&DATA[1..]);

    case::get_pushed_data(0, 0, Some((1, hash)))
        .op_return(DATA)
        .accepts();
}

#[test]
fn get_pushdata2() {
    // The last 3 bytes are the pushed data.
    const DATA: &[u8] = &[1, 0x4d, 0x03, 0x00, 0xde, 0xad, 0xbe];

    let hash = hash_of_pushed_data(&DATA[4..]);

    case::get_pushed_data(0, 0, Some((2, hash)))
        .op_return(DATA)
        .accepts();
}

#[test]
fn get_pushdata4() {
    // The last 3 bytes are the pushed data.
    const DATA: &[u8] = &[1, 0x4e, 0x03, 0x00, 0x00, 0x00, 0xde, 0xad, 0xbe];

    let hash = hash_of_pushed_data(&DATA[6..]);

    case::get_pushed_data(0, 0, Some((3, hash)))
        .op_return(DATA)
        .accepts();
}

#[test]
fn get_pushed_data_nonexistent_output() {
    case::get_pushed_data(NONEXISTENT_OUTPUT, 0, None)
        .op_return(DEFAULT_DATA)
        .accepts();
}

#[test]
fn get_pushed_data_not_op_return_output() {
    case::get_pushed_data(0, 0, None).accepts();
}

#[test]
fn get_pushed_data_out_of_range_entry() {
    case::get_pushed_data(0, OUT_OF_RANGE_ENTRY, None)
        .op_return(DEFAULT_DATA)
        .accepts();
}

#[test]
fn get_pushed_data_neg1_returns_none() {
    case::get_pushed_data(0, 0, None)
        .op_return(OP_PUSHNUM_NEG1)
        .accepts();
}

// 3. op_x_minus_1_from_entry
#[test]
fn op_x_minus_1_from_entry() {
    for value in 0..16 {
        case::op_x_minus_1_from_entry(op_x_minus_1(value), Some(value)).accepts();
    }
}

#[test]
fn op_x_minus_1_from_entry_returns_none() {
    case::op_x_minus_1_from_entry(negate_or_reserved(0), None).accepts();
    case::op_x_minus_1_from_entry(negate_or_reserved(1), None).accepts();
    case::op_x_minus_1_from_entry(pushed_data(0, [0; 32]), None).accepts();
}

#[test]
fn op_x_minus_1_from_entry_rejects_wrong_expectation() {
    case::op_x_minus_1_from_entry(op_x_minus_1(0), Some(1)).rejects(AssertFailed);
    case::op_x_minus_1_from_entry(op_x_minus_1(0), None).rejects(AssertFailed);
}

// 4. get_op_x_minus_1
#[test]
fn get_op_x_minus_1() {
    for value in 0..16_u8 {
        case::get_op_x_minus_1(0, u32::from(value), Some(value))
            .op_return(OP_1_TO_16)
            .accepts();
    }
}

#[test]
fn get_op_x_minus_1_nonexistent_output() {
    case::get_op_x_minus_1(NONEXISTENT_OUTPUT, 0, None)
        .op_return(OP_1)
        .accepts();
}

#[test]
fn get_op_x_minus_1_not_op_return_output() {
    case::get_op_x_minus_1(0, 0, None).accepts();
}

#[test]
fn get_op_x_minus_1_out_of_range_entry() {
    case::get_op_x_minus_1(0, OUT_OF_RANGE_ENTRY, None)
        .op_return(OP_1)
        .accepts();
}

#[test]
fn get_op_x_minus_1_returns_none() {
    case::get_op_x_minus_1(0, 0, None)
        .op_return(OP_RESERVED)
        .accepts();
    case::get_op_x_minus_1(0, 0, None)
        .op_return(DEFAULT_DATA)
        .accepts();
}

// 5. negate_or_reserved_from_entry
#[test]
fn negate_or_reserved_from_entry_negate() {
    case::negate_or_reserved_from_entry(negate_or_reserved(0), Some(0)).accepts();
}

#[test]
fn negate_or_reserved_from_entry_reserved() {
    case::negate_or_reserved_from_entry(negate_or_reserved(1), Some(1)).accepts();
}

#[test]
fn negate_or_reserved_from_entry_returns_none() {
    case::negate_or_reserved_from_entry(pushed_data(0, hash_of_pushed_data(&[1])), None).accepts();
    case::negate_or_reserved_from_entry(op_x_minus_1(0), None).accepts();
}

#[test]
fn negate_or_reserved_from_entry_rejects_wrong_expectation() {
    case::negate_or_reserved_from_entry(negate_or_reserved(0), Some(1)).rejects(AssertFailed);
    case::negate_or_reserved_from_entry(negate_or_reserved(1), None).rejects(AssertFailed);
}

// 6. get_negate_or_reserved
#[test]
fn get_negate_or_reserved_negate() {
    case::get_negate_or_reserved(0, 0, Some(0))
        .op_return(OP_PUSHNUM_NEG1)
        .accepts();
}

#[test]
fn get_negate_or_reserved_reserved() {
    case::get_negate_or_reserved(0, 0, Some(1))
        .op_return(OP_RESERVED)
        .accepts();
}

#[test]
fn get_negate_or_reserved_nonexistent_output() {
    case::get_negate_or_reserved(NONEXISTENT_OUTPUT, 0, None)
        .op_return(OP_RESERVED)
        .accepts();
}

#[test]
fn get_negate_or_reserved_not_op_return_output() {
    case::get_negate_or_reserved(0, 0, None).accepts();
}

#[test]
fn get_negate_or_reserved_out_of_range_entry() {
    case::get_negate_or_reserved(0, OUT_OF_RANGE_ENTRY, None)
        .op_return(OP_RESERVED)
        .accepts();
}

#[test]
fn get_negate_or_reserved_returns_none() {
    case::get_negate_or_reserved(0, 0, None)
        .op_return(DEFAULT_DATA)
        .accepts();
}

// 7. kind_of_entry
#[test]
fn kind_of_entry_pushed_data() {
    case::kind_of_entry(pushed_data(0, hash_of_pushed_data(&[1])), 0).accepts();
}

#[test]
fn kind_of_entry_op_x_minus_1() {
    for value in 0..16 {
        case::kind_of_entry(op_x_minus_1(value), 1).accepts();
    }
}

#[test]
fn kind_of_entry_negate_or_reserved() {
    // negate
    case::kind_of_entry(negate_or_reserved(0), 2).accepts();
    // reserved
    case::kind_of_entry(negate_or_reserved(1), 2).accepts();
}

#[test]
fn kind_of_entry_rejects_wrong_kind() {
    case::kind_of_entry(pushed_data(0, [0; 32]), 1).rejects(AssertFailed);
    case::kind_of_entry(op_x_minus_1(0), 2).rejects(AssertFailed);
    case::kind_of_entry(negate_or_reserved(0), 0).rejects(AssertFailed);
}

// 8. is_output_op_return
#[test]
fn is_output_op_return_true() {
    case::is_output_op_return(0, true)
        .op_return(DEFAULT_DATA)
        .accepts();
}

#[test]
fn is_output_op_return_nonexistent_output_false() {
    case::is_output_op_return(NONEXISTENT_OUTPUT, false)
        .op_return(DEFAULT_DATA)
        .accepts();
}

#[test]
fn is_output_op_return_false() {
    case::is_output_op_return(0, false).accepts();
    case::is_output_op_return(1, false)
        .op_return(DEFAULT_DATA)
        .accepts();
}

#[test]
fn is_output_op_return_rejects_wrong_expectation() {
    case::is_output_op_return(0, false)
        .op_return(DEFAULT_DATA)
        .rejects(AssertFailed);
    case::is_output_op_return(0, true).rejects(AssertFailed);
}

// 9. has_entry
#[test]
fn has_entry_true() {
    case::has_entry(0, 0, true)
        .op_return(DEFAULT_DATA)
        .accepts();
}

#[test]
fn has_entry_nonexistent_output_false() {
    case::has_entry(NONEXISTENT_OUTPUT, 0, false)
        .op_return(DEFAULT_DATA)
        .accepts();
}

#[test]
fn has_entry_out_of_range_entry_false() {
    case::has_entry(0, OUT_OF_RANGE_ENTRY, false)
        .op_return(DEFAULT_DATA)
        .accepts();
}

#[test]
fn has_entry_output_not_op_return_false() {
    case::has_entry(0, 0, false).accepts();
}

// 10. assert_output_is_op_return
#[test]
fn assert_output_is_op_return_pass() {
    case::assert_output_is_op_return(0)
        .op_return(DEFAULT_DATA)
        .accepts();
}

#[test]
fn assert_output_is_op_return_nonexistent_output_fails() {
    case::assert_output_is_op_return(NONEXISTENT_OUTPUT)
        .op_return(DEFAULT_DATA)
        .rejects(AssertFailed);
}

#[test]
fn assert_output_is_op_return_fail() {
    case::assert_output_is_op_return(0).rejects(AssertFailed);
}

// 11. assert_has_entry
#[test]
fn assert_has_entry_pass() {
    case::assert_has_entry(0, 0)
        .op_return(DEFAULT_DATA)
        .accepts();
}

#[test]
fn assert_has_entry_nonexistent_output_fails() {
    case::assert_has_entry(NONEXISTENT_OUTPUT, 0)
        .op_return(DEFAULT_DATA)
        .rejects(AssertFailed);
}

#[test]
fn assert_has_entry_output_not_op_return_fails() {
    case::assert_has_entry(0, 0).rejects(AssertFailed);
}

#[test]
fn assert_has_entry_out_of_range_entry_fails() {
    case::assert_has_entry(0, OUT_OF_RANGE_ENTRY)
        .op_return(DEFAULT_DATA)
        .rejects(AssertFailed);
}

// Empty OP_RETURN
#[test]
fn empty_op_return_is_op_return() {
    case::is_output_op_return(0, true)
        .outputs([empty_op_return_output()])
        .accepts();
    case::assert_output_is_op_return(0)
        .outputs([empty_op_return_output()])
        .accepts();
}

#[test]
fn empty_op_return_has_no_entries() {
    let empty = || [empty_op_return_output()];

    case::has_entry(0, 0, false).outputs(empty()).accepts();
    case::get_entry_option(0, 0, None)
        .outputs(empty())
        .accepts();
    case::get_pushed_data(0, 0, None).outputs(empty()).accepts();
    case::get_op_x_minus_1(0, 0, None)
        .outputs(empty())
        .accepts();
    case::get_negate_or_reserved(0, 0, None)
        .outputs(empty())
        .accepts();
}

#[test]
fn assert_has_entry_empty_op_return_fails() {
    case::assert_has_entry(0, 0)
        .outputs([empty_op_return_output()])
        .rejects(AssertFailed);
}

// Reference transaction
#[test]
fn reference_tx_null_data_model() {
    let tx = reference_tx();
    let entries = |output| {
        output_null_datum(&tx, output, 0).map(|_| {
            let script = &tx.outputs()[output as usize].script_pubkey;

            null_data_entries(script.as_bytes())
                .expect("null data")
                .len()
        })
    };

    assert_eq!(entries(reference::PUSHDATA0), Some(1));
    assert_eq!(entries(reference::NOT_OP_RETURN), None);
    assert_eq!(entries(reference::MIXED), Some(4));
    assert_eq!(entries(reference::EMPTY), Some(0));
    assert_eq!(entries(reference::PUSHDATA1), Some(1));
    assert_eq!(entries(reference::OP_1_TO_16), Some(16));
    assert_eq!(entries(reference::MALFORMED), None);
    assert_eq!(entries(reference::FEE), None);
    assert_eq!(entries(reference::OUTPUTS), None);
}

/// The contract agrees with [`output_null_datum`] on every output and entry of the reference
/// transaction, so fuzz cases can rely on the model.
#[test]
fn reference_tx_matches_model() {
    let tx = reference_tx();

    for output_index in 0..=reference::OUTPUTS {
        let datum = output_null_datum(&tx, output_index, 0);

        case::is_output_op_return(output_index, datum.is_some())
            .in_reference_tx()
            .accepts();

        let entries = tx
            .outputs()
            .get(output_index as usize)
            .and_then(|output| null_data_entries(output.script_pubkey.as_bytes()))
            .map_or(0, |entries| entries.len() as u32);

        for entry_index in 0..=entries {
            let expected = output_null_datum(&tx, output_index, entry_index).flatten();

            case::get_entry_option(output_index, entry_index, expected)
                .in_reference_tx()
                .accepts();
        }
    }
}

#[test]
fn is_output_op_return_malformed_false() {
    case::is_output_op_return(reference::MALFORMED, false)
        .in_reference_tx()
        .accepts();
    case::assert_output_is_op_return(reference::MALFORMED)
        .in_reference_tx()
        .rejects(AssertFailed);
}
