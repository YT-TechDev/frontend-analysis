use std::fmt::Debug;

pub(super) const GENERATED_INPUT_COUNT: usize = 4_096;
pub(super) const GENERATED_MAX_BYTES: usize = 64;
pub(super) const GENERATOR_REVISION: &str = "css-tokenizer-valid-utf8-lcg-v1";
pub(super) const GENERATOR_SEED: u64 = 0x4353_532d_3133_3521;

const ALPHABET: &[&str] = &[
    "a", "Z", "0", "9", " ", "\t", "\n", "\r", "\u{000c}", "\0", "\"", "'", "\\", "/", "*", "#",
    "@", "+", "-", ".", "%", "!", "(", ")", "[", "]", "{", "}", ":", ";", ",", "é", "中", "😀",
];

pub(super) fn generated_inputs() -> Vec<String> {
    let mut state = GENERATOR_SEED;
    let mut inputs = Vec::with_capacity(GENERATED_INPUT_COUNT);

    for case_index in 0..GENERATED_INPUT_COUNT {
        state = next(state ^ case_index as u64);
        let target_units = (state as usize % 48) + 1;
        let mut source = String::new();

        for unit_index in 0..target_units {
            state = next(state ^ unit_index as u64);
            let candidate = ALPHABET[state as usize % ALPHABET.len()];
            if source.len() + candidate.len() > GENERATED_MAX_BYTES {
                break;
            }
            source.push_str(candidate);
        }

        inputs.push(source);
    }

    inputs
}

pub(super) fn assert_deterministic_replay<T, F>(inputs: &[String], runner: F)
where
    T: Debug + Eq,
    F: Fn(&str) -> T,
{
    for source in inputs {
        let first = runner(source);
        let second = runner(source);
        assert_eq!(first, second, "deterministic replay mismatch");
    }
}

pub(super) fn alphabet_contains_required_boundaries() -> bool {
    [
        "\n", "\r", "\u{000c}", "\0", "\"", "'", "\\", "/", "*", "#", "@", "+", "-", ".", "%", "!",
        "(", ")", "[", "]", "{", "}", ":", ";", ",", "é",
    ]
    .into_iter()
    .all(|required| ALPHABET.contains(&required))
}

const fn next(value: u64) -> u64 {
    value
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407)
}
