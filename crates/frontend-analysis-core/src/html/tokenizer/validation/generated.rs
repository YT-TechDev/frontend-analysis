pub(super) const MAX_GENERATED_CASES: usize = 4_096;
pub(super) const MAX_SOURCE_BYTES: usize = 64;

const ALPHABET: &[&str] = &[
    "a", "Z", "0", "9", "<", ">", "/", "=", "&", "!", "?", " ", "\t", "\n",
    "\r", "\u{000c}", "\0", "'", "\"", "é", "界", "ß",
];

const SEEDS: &[&str] = &[
    "", "<", "</", "<a", "<a>", "&", "<!", "<?", "\0", "\r\n", "é",
];

pub(super) fn generated_inputs() -> Vec<String> {
    let mut values = Vec::with_capacity(MAX_GENERATED_CASES);
    for seed in SEEDS {
        push_unique_bounded(&mut values, (*seed).to_owned());
    }

    let mut ordinal = 0usize;
    while values.len() < MAX_GENERATED_CASES {
        let mut value = String::new();
        let mut cursor = ordinal;
        let width = 1 + ordinal % 5;
        for _ in 0..width {
            let symbol = ALPHABET[cursor % ALPHABET.len()];
            if value.len() + symbol.len() > MAX_SOURCE_BYTES {
                break;
            }
            value.push_str(symbol);
            cursor /= ALPHABET.len();
        }
        push_unique_bounded(&mut values, value);
        ordinal = ordinal.checked_add(1).expect("bounded generator ordinal");
    }
    values
}

pub(super) fn minimization_candidates(input: &str) -> Vec<String> {
    let mut candidates = Vec::new();
    candidates.push(String::new());

    for (offset, _) in input.char_indices().skip(1) {
        candidates.push(input[..offset].to_owned());
    }

    let boundaries: Vec<usize> = input
        .char_indices()
        .map(|(offset, _)| offset)
        .chain(std::iter::once(input.len()))
        .collect();
    for window in boundaries.windows(2) {
        let mut candidate = String::with_capacity(input.len() - (window[1] - window[0]));
        candidate.push_str(&input[..window[0]]);
        candidate.push_str(&input[window[1]..]);
        candidates.push(candidate);
    }

    candidates.sort_by(|left, right| {
        left.len()
            .cmp(&right.len())
            .then_with(|| left.as_bytes().cmp(right.as_bytes()))
    });
    candidates.dedup();
    candidates.retain(|value| value.len() <= MAX_SOURCE_BYTES);
    candidates
}

fn push_unique_bounded(values: &mut Vec<String>, value: String) {
    if value.len() <= MAX_SOURCE_BYTES && !values.iter().any(|existing| existing == &value) {
        values.push(value);
    }
}
