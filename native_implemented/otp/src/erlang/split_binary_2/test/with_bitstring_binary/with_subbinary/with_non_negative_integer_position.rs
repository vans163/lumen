use super::*;

#[test]
fn with_less_than_byte_len_returns_binary_prefix_and_suffix_bitstring() {
    with_process(|process| {
        let binary = bitstring!(1, 2 :: 2, &process);
        let position = process.integer(1);

        assert_eq!(
            result(process, binary, position),
            Ok(process.tuple_from_slice(&[
                process.binary_from_bytes(&[1]),
                bitstring!(2 :: 2, &process)
            ],))
        )
    })
}

#[test]
fn with_byte_len_without_bit_count_returns_subbinary_and_empty_suffix() {
    with_process(|process| {
        let original = process.binary_from_bytes(&[1]);
        let binary = process.subbinary_from_original(original, 0, 0, 1, 0);
        let position = process.integer(1);

        assert_eq!(
            result(process, binary, position),
            Ok(process.tuple_from_slice(&[binary, process.binary_from_bytes(&[])],))
        );
    });
}

#[test]
fn with_byte_len_with_bit_count_errors_badarg() {
    with_process(|process| {
        let binary = bitstring!(1, 2 :: 2, &process);
        let position = process.integer(2);

        assert_badarg!(result(process, binary, position), "bitstring (<<1,2:2>>) has 2 bits in its partial bytes, so the index (2) cannot equal the total byte length (2)");
    });
}

#[test]
fn with_greater_than_byte_len_errors_badarg() {
    with_process(|process| {
        let binary = bitstring!(1, 2 :: 2, &process);
        let position = process.integer(3);

        assert_badarg!(
            result(process, binary, position),
            "index (3) exceeds total byte length (2) of bitstring (<<1,2:2>>)"
        );
    });
}
