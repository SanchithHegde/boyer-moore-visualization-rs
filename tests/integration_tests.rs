use anyhow::Result;
use boyer_moore::BoyerMoore;
use pretty_assertions::assert_eq;

const ALPHABET: &str = "ACGT";

#[test]
fn bad_char_rule() {
    let pattern = "TCAA";
    let bm = BoyerMoore::new(pattern, ALPHABET).unwrap();
    assert_eq!(bm.bad_char_rule(2, 'T').unwrap(), 2);
}

#[test]
fn good_suffix_rule() {
    let pattern = "ACTA";
    let bm = BoyerMoore::new(pattern, ALPHABET).unwrap();
    assert_eq!(bm.good_suffix_rule(0).unwrap(), 3);
}

#[test]
fn match_skip() {
    let pattern = "ACAC";
    let bm = BoyerMoore::new(pattern, ALPHABET).unwrap();
    assert_eq!(bm.match_skip(), 2);
}

fn boyer_moore_search(pattern: &str, bm: BoyerMoore, text: &str) -> Result<Vec<usize>> {
    let mut occurrences = Vec::new();

    let pattern = pattern.as_bytes();
    let text = text.as_bytes();
    let mut i = 0;

    while i < text.len() - pattern.len() + 1 {
        let mut shift = 1;
        let mut mismatched = false;

        for j in (0..pattern.len()).rev() {
            if pattern[j] != text[i + j] {
                let skip_bc = bm.bad_char_rule(j, text[i + j] as char)?;
                let skip_gs = bm.good_suffix_rule(j)?;
                shift = *[shift, skip_bc, skip_gs].iter().max().unwrap();
                mismatched = true;
                break;
            }
        }

        if !mismatched {
            occurrences.push(i);
            let skip_gs = bm.match_skip();
            shift = *[shift, skip_gs].iter().max().unwrap();
        }

        i += shift;
    }

    Ok(occurrences)
}

#[test]
fn search() {
    let text = "GCTAGCTCTACGAGTCTA";
    let pattern = "TCTA";
    let bm = BoyerMoore::new(pattern, ALPHABET).unwrap();

    assert_eq!(boyer_moore_search(pattern, bm, text).unwrap(), vec![6, 14]);
}
