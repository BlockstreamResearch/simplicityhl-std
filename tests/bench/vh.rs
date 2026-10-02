//! "For computing `vH` it's plausibly cheaper to directly compute this by an
//! addition ladder. If the asset ID is known, you can embed a lookup table":
//! the cost of `v*H` alone, for a 64-bit `v`.
//!
//! The library computes `v*H` with `jet::scale` (`relations::amount_to_gej`,
//! `proofs::assert_exact_value*`), so that is the jet to beat. A check of
//! `c == v*H + vbf*G` built on `v*H` also pays `jet::generate`, which alone
//! costs more than `jet::linear_verify_1`, so that check is not measured here.
//!
//! The program is generated and compiled in memory. A table with `w`-bit
//! windows holds `64/w * (2^w - 1)` multiples of `H0` and adds one per nonzero
//! window of `v`; only the entries `v` selects survive pruning. Windows of 16
//! bits and more are too large to compile as SimplicityHL source.
//!
//! Ignored by default: the tables take ~10 minutes in a debug build. Run with
//! `cargo test --release --test bench vh -- --ignored --nocapture`.

use std::collections::HashMap;
use std::fmt::Write;
use std::sync::OnceLock;

use simplex::simplicityhl::num::U256;
use simplex::simplicityhl::str::WitnessName;
use simplex::simplicityhl::value::{UIntValue, ValueConstructible};
use simplex::simplicityhl::{CompiledProgram, Value, WitnessValues};

use crate::helpers::{
    Ge, blinded_generator, decode, random_amount, random_asset_id, random_scalar,
    unblinded_commitment, unblinded_generator,
};
use crate::{Cost, compile, execute, ge, mul, print_header, print_row};

/// L-BTC, `6f0279e9…381c526d` in display order, here in internal byte order.
const ASSET_ID: [u8; 32] = [
    0x6d, 0x52, 0x1c, 0x38, 0xec, 0x1e, 0xa1, 0x57, 0x34, 0xae, 0x22, 0xb7, 0xc4, 0x60, 0x64, 0x41,
    0x28, 0x29, 0xc0, 0xd0, 0x57, 0x9f, 0x0a, 0x71, 0x3d, 0x1c, 0x04, 0xed, 0xe9, 0x79, 0x02, 0x6f,
];

/// Window widths of the lookup tables.
const WINDOWS: [u32; 4] = [1, 2, 4, 8];

/// Which generator a candidate multiplies.
#[derive(Clone, Copy)]
enum Generator {
    /// Any `h`, read from the witness.
    Witness,
    /// `H0 = hash_to_curve(L-BTC)`, known when the contract is written.
    Known,
}

struct Candidate {
    name: String,
    generator: Generator,
    /// A statement that fails unless `c == v*h`.
    check: String,
}

fn candidates() -> Vec<Candidate> {
    let eq = |expr: &str| format!("assert!(jet::gej_ge_equiv({expr}, c));");
    let candidate = |name: &str, generator, check| Candidate {
        name: name.to_string(),
        generator,
        check,
    };

    let mut out = vec![
        candidate(
            "linear_verify_1 (v*h + 0*G)",
            Generator::Witness,
            "jet::linear_verify_1(((v256, h), 0), c);".to_string(),
        ),
        candidate("scale", Generator::Witness, eq("jet::scale(v256, (h, 1))")),
        candidate("double-and-add", Generator::Witness, eq("dbl_add_64(v, h)")),
        candidate(
            "hash_to_curve + scale",
            Generator::Known,
            eq(&format!(
                "jet::scale(v256, (jet::hash_to_curve({}), 1))",
                hex(&ASSET_ID)
            )),
        ),
        candidate(
            "scale, H0 embedded",
            Generator::Known,
            eq("jet::scale(v256, (h0(), 1))"),
        ),
    ];
    for w in WINDOWS {
        out.push(candidate(
            &format!("table, {w}-bit windows"),
            Generator::Known,
            eq(&format!("table_{w}(v)")),
        ));
    }
    for w in WINDOWS {
        out.push(candidate(
            &format!("table, {w}-bit windows, select entry"),
            Generator::Known,
            eq(&format!("select_{w}(v)")),
        ));
    }

    out
}

fn hex(bytes: &[u8]) -> String {
    let digits: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    format!("0x{digits}")
}

fn h0() -> Ge {
    static H0: OnceLock<Ge> = OnceLock::new();
    *H0.get_or_init(|| decode(unblinded_generator(ASSET_ID).serialize()))
}

/// `k*H0` as a SimplicityHL `Ge` literal.
fn multiple(k: u64) -> String {
    let (x, y) = ge(mul(h0(), k));
    format!("({}, {})", hex(&x), hex(&y))
}

/// `dbl_add_64(v, h)`; the generated program cannot import it.
const LADDER: &str = include_str!("../../simf/bench/ct/ladder.simf");

/// `let` statements splitting `n: u{width}` into bits `b{i}`, `b0` least significant.
fn split_bits(width: u32) -> String {
    let mut out = String::new();
    let mut parts = vec![("n".to_string(), width, 0)];

    while let Some((name, size, low)) = parts.pop() {
        let half = size / 2;
        let (hi, lo) = if half == 1 {
            (format!("b{}", low + 1), format!("b{low}"))
        } else {
            (format!("n{half}_{}", low + half), format!("n{half}_{low}"))
        };
        writeln!(
            out,
            "    let ({hi}, {lo}): (u{half}, u{half}) = <u{size}>::into({name});"
        )
        .unwrap();

        if half > 1 {
            parts.push((hi, half, low + half));
            parts.push((lo, half, low));
        }
    }

    out
}

/// How a window turns its bits into a point.
#[derive(Clone, Copy)]
enum Lookup {
    /// The match over the bits adds the entry to `acc`, carrying `acc` through
    /// every level.
    Add,
    /// The match over the bits returns the entry as an `Option<Ge>`, and one
    /// addition outside it adds it to `acc`.
    Select,
}

impl Lookup {
    fn prefix(self) -> &'static str {
        match self {
            Lookup::Add => "table",
            Lookup::Select => "select",
        }
    }
}

/// A match over the bits `b{bit}..b0` that, for the window value `k`, adds
/// `(k << shift)*H0` to `acc` or returns it, depending on `lookup`.
fn window_match(lookup: Lookup, bit: i32, prefix: u64, shift: u32, indent: usize) -> String {
    let pad = " ".repeat(indent);

    if bit < 0 {
        return match (lookup, prefix) {
            (Lookup::Add, 0) => "acc".to_string(),
            (Lookup::Add, k) => format!("jet::gej_ge_add(acc, {})", multiple(k << shift)),
            (Lookup::Select, 0) => "None".to_string(),
            (Lookup::Select, k) => format!("Some({})", multiple(k << shift)),
        };
    }

    format!(
        "match <u1>::into(b{bit}) {{\n{pad}    false => {},\n{pad}    true => {},\n{pad}}}",
        window_match(lookup, bit - 1, prefix << 1, shift, indent + 4),
        window_match(lookup, bit - 1, (prefix << 1) | 1, shift, indent + 4),
    )
}

/// `{prefix}_{w}(v) = v*H0` from `w`-bit windows. The windows are combined by
/// halving `v` recursively: a flat chain of `let acc` carries every shadowed
/// `acc` in its environment and costs several times more.
fn table(lookup: Lookup, w: u32) -> String {
    let mut s = String::new();
    let p = lookup.prefix();

    for i in 0..64 / w {
        let bits = if w == 1 {
            "    let b0: u1 = n;\n".to_string()
        } else {
            split_bits(w)
        };
        let shift = w * i;
        let entry = window_match(lookup, w as i32 - 1, 0, shift, 4);

        match lookup {
            Lookup::Add => writeln!(
                s,
                "/// `acc + n*2^{shift}*H0`\nfn {p}_window_{w}_{i}(acc: Gej, n: u{w}) -> Gej {{\n{bits}\n    {entry}\n}}\n",
            ),
            Lookup::Select => writeln!(
                s,
                "/// `n*2^{shift}*H0`, `None` for `n == 0`\nfn {p}_entry_{w}_{i}(n: u{w}) -> Option<Ge> {{\n{bits}\n    {entry}\n}}\n\n\
                 /// `acc + n*2^{shift}*H0`\nfn {p}_window_{w}_{i}(acc: Gej, n: u{w}) -> Gej {{\n    match {p}_entry_{w}_{i}(n) {{\n        Some(entry: Ge) => jet::gej_ge_add(acc, entry),\n        None => acc,\n    }}\n}}\n",
            ),
        }
        .unwrap();
    }

    let mut size = 2 * w;
    while size <= 64 {
        let half = size / 2;
        let part = |j: u32| match half == w {
            true => format!("{p}_window_{w}_{j}"),
            false => format!("{p}_{w}_{half}_{j}"),
        };
        for j in 0..64 / size {
            writeln!(
                s,
                "fn {p}_{w}_{size}_{j}(acc: Gej, x: u{size}) -> Gej {{\n    let (hi, lo): (u{half}, u{half}) = <u{size}>::into(x);\n    {}({}(acc, lo), hi)\n}}\n",
                part(2 * j + 1),
                part(2 * j),
            )
            .unwrap();
        }
        size *= 2;
    }

    writeln!(
        s,
        "/// `v*H0` from {w}-bit windows\nfn {p}_{w}(v: u64) -> Gej {{\n    {p}_{w}_64_0(jet::gej_infinity(), v)\n}}\n"
    )
    .unwrap();

    s
}

fn source() -> String {
    let mut s = format!("/// `H0`\nfn h0() -> Ge {{\n    {}\n}}\n", multiple(1));
    s += LADDER;
    for w in WINDOWS {
        s += &table(Lookup::Add, w);
        s += &table(Lookup::Select, w);
    }

    s += "
fn main() {
    let fn_idx: u8 = witness::FUNCTION_INDEX;

    let v: u64 = witness::V;
    let h: Ge = witness::H;
    let c: Ge = witness::C;
    let v256: u256 = <(u128, u128)>::into((0, <(u64, u64)>::into((0, v))));

    // Baseline: everything except computing `v*h`.
    match jet::eq_8(0, fn_idx) {
        true => {
            assert!(jet::gej_ge_equiv((c, 1), c));
        },
        false => (),
    };
";
    for (i, candidate) in candidates().iter().enumerate() {
        writeln!(
            s,
            "    match jet::eq_8({}, fn_idx) {{\n        true => {{\n            {}\n        }},\n        false => (),\n    }};",
            i + 1,
            candidate.check
        )
        .unwrap();
    }
    s += "}\n";

    s
}

fn program() -> &'static CompiledProgram {
    static PROGRAM: OnceLock<CompiledProgram> = OnceLock::new();
    PROGRAM.get_or_init(|| compile(&source()).unwrap())
}

/// `c == v*h` for the generator of `candidate`, claiming `c == claimed*h`.
fn witness_claiming(index: usize, generator: Generator, v: u64, claimed: u64) -> WitnessValues {
    let h = match generator {
        Generator::Witness => blinded_generator(random_asset_id(), &random_scalar()),
        Generator::Known => unblinded_generator(ASSET_ID),
    };
    let point = |(x, y): Ge| {
        Value::tuple([
            Value::from(UIntValue::U256(U256::from_byte_array(x))),
            Value::from(UIntValue::U256(U256::from_byte_array(y))),
        ])
    };
    let name = WitnessName::from_str_unchecked;

    WitnessValues::from(HashMap::from([
        (
            name("FUNCTION_INDEX"),
            Value::from(UIntValue::U8(index as u8)),
        ),
        (name("V"), Value::from(UIntValue::U64(claimed))),
        (name("H"), point(decode(h.serialize()))),
        (
            name("C"),
            point(decode(unblinded_commitment(v, h).serialize())),
        ),
    ]))
}

fn witness(index: usize, generator: Generator, v: u64) -> WitnessValues {
    witness_claiming(index, generator, v, v)
}

/// The cost of `candidate` above the baseline for `v`.
fn cost(index: usize, generator: Generator, v: u64) -> Cost {
    let w = witness(index, generator, v);
    let base = execute(program(), witness(0, generator, v)).unwrap();
    let cost = execute(program(), w).unwrap();

    cost - base
}

/// Small, typical and worst-case values: a table's cost grows with the number
/// of nonzero windows of `v`.
fn values() -> [u64; 4] {
    [1, random_amount(), rand::random(), u64::MAX]
}

#[test]
#[ignore = "slow; run with --release"]
fn candidates_are_correct() {
    for (i, candidate) in candidates().iter().enumerate() {
        let (index, name) = (i + 1, &candidate.name);

        for v in values().into_iter().chain([15, 16, 255, 256]) {
            execute(program(), witness(index, candidate.generator, v))
                .unwrap_or_else(|e| panic!("{name} rejected `{v}*h`: {e}"));
        }

        assert!(
            execute(
                program(),
                witness_claiming(index, candidate.generator, 7, 8)
            )
            .is_err(),
            "{name} accepted a wrong value"
        );
    }
}

#[test]
#[ignore = "slow; run with --release"]
fn candidates_cost() {
    print_header("Compute `v*h` for a 64-bit `v`");

    for (i, candidate) in candidates().iter().enumerate() {
        let costs: Vec<Cost> = values()
            .into_iter()
            .map(|v| cost(i + 1, candidate.generator, v))
            .collect();

        print_row(&candidate.name, &costs);
    }
}
