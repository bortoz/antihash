use rand::Rng;
use std::collections::HashMap;

fn gen_string(len: u64, alphabet: &Vec<String>) -> String {
    let mut rng = rand::thread_rng();
    let mut word = String::new();
    for _ in 0..len {
        let idx = rng.gen_range(0, alphabet.len());
        word.push_str(&alphabet[idx]);
    }
    return word;
}

fn get_hash(word: &String, base: u64, module: u64) -> u64 {
    let mut res = 0;
    for c in word.chars() {
        res = (res * base + c as u64) % module;
    }
    return res;
}

fn find_single_collision(
    base: u64, module: u64, alphabet: &Vec<String>,
) -> Option<(String, String)> {
    let bound = (module as f64).sqrt() as usize;
    let mut samples = HashMap::with_capacity(bound);
    for len in 6..64 {
        samples.clear();
        for _ in 0..bound {
            let word = gen_string(len, alphabet);
            let hash = get_hash(&word, base, module);
            if let Some(coll) = samples.insert(hash, word.clone()) {
                if word != coll {
                    return Some((word, coll));
                }
            }
        }
    }
    None
}

pub fn find_collision(
    bases: Vec<u64>, modules: Vec<u64>, init_alphabet: Vec<String>,
) -> Option<(String, String)> {
    let mut alphabet = init_alphabet;
    for (&b, &m) in bases.iter().zip(modules.iter()) {
        if let Some((fi, se)) = find_single_collision(b, m, &alphabet) {
            alphabet = vec![fi, se];
        } else {
            return None;
        }
    }
    let se = alphabet.remove(1);
    let fi = alphabet.remove(0);
    Some((fi, se))
}
