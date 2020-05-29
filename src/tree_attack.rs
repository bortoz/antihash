use binary_heap_plus::{BinaryHeap, MinComparator};
use rand::Rng;
use std::collections::{HashSet, VecDeque};
use std::cmp::PartialEq;

#[derive(Clone)]
struct TreeAttackInternalNode {
    sum: i128,
    idx: usize,
    rev_left: bool,
    rev_right: bool,
    pos_left: usize,
    pos_right: usize,
}

#[derive(Clone)]
struct TreeAttackLeafNode<'a> {
    sum: i128,
    idx: usize,
    word1: &'a String,
    word2: &'a String,
}

#[derive(Clone)]
enum TreeAttackNode<'a> {
    Internal(TreeAttackInternalNode),
    Leaf(TreeAttackLeafNode<'a>),
}

impl<'a> TreeAttackNode<'a> {
    fn new_internal(
        sum: i128, idx: usize, rev_left: bool, rev_right: bool, pos_left: usize, pos_right: usize,
    ) -> TreeAttackNode<'a> {
        TreeAttackNode::Internal(TreeAttackInternalNode {
            sum,
            idx,
            rev_left,
            rev_right,
            pos_left,
            pos_right,
        })
    }

    fn new_leaf(
        idx: usize, word1: &'a String, word2: &'a String, base: i128, module: i128, pot: i128,
    ) -> TreeAttackNode<'a> {
        let mut hash = 0;
        for (c1, c2) in word1.chars().zip(word2.chars()) {
            hash = (hash * base + c1 as i128 - c2 as i128 + module) % module;
        }
        let sum = hash * pot % module;
        TreeAttackNode::Leaf(TreeAttackLeafNode {
            sum,
            idx,
            word1,
            word2,
        })
    }

    fn get_sum(&self) -> i128 {
        match self {
            TreeAttackNode::Internal(n) => n.sum,
            TreeAttackNode::Leaf(n) => n.sum,
        }
    }
}

impl<'a> PartialEq for TreeAttackNode<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.get_sum().eq(&other.get_sum())
    }
}

struct TreeAttack<'a> {
    alphabet: &'a Vec<String>,
    word_len: usize,
    base: i128,
    module: i128,
    cluster_size: usize,
    tree: Vec<Vec<TreeAttackNode<'a>>>,
    heap: BinaryHeap<(i128, usize, usize, bool), MinComparator>,
    added: HashSet<(usize, usize, bool)>,
}

impl<'a> TreeAttack<'a> {
    fn new(
        base: u64, module: u64, cluster_size: usize, alphabet: &'a Vec<String>,
    ) -> TreeAttack<'a> {
        TreeAttack {
            alphabet: alphabet,
            word_len: alphabet[0].len(),
            base: base as i128,
            module: module as i128,
            cluster_size: cluster_size,
            tree: Vec::new(),
            heap: BinaryHeap::with_capacity_min(3 * cluster_size),
            added: HashSet::with_capacity(5 * cluster_size),
        }
    }

    fn init_attack(&mut self, len: usize) {
        self.tree.resize(2 * len, Vec::with_capacity(self.cluster_size));
        let mut pot = 1i128;
        for i in (0..len).rev() {
            self.tree[i].clear();
            for a in 0..self.alphabet.len() {
                for b in 0..self.alphabet.len() {
                    if a != b {
                        self.tree[i + len].push(TreeAttackNode::new_leaf(
                            i,
                            &self.alphabet[a],
                            &self.alphabet[b],
                            self.base,
                            self.module,
                            pot,
                        ))
                    }
                }
            }
            self.tree[i + len].sort_unstable_by_key(|k| k.get_sum());
            self.tree[i + len].dedup();
            for _ in 0..self.word_len {
                pot = pot * self.base % self.module;
            }
        }
    }

    fn calc_sum(&self, l: usize, r: usize, pl: usize, pr: usize) -> i128 {
        self.tree[l][pl].get_sum() + self.tree[r][pr].get_sum()
    }

    fn calc_diff(&self, l: usize, r: usize, pl: usize, pr: usize) -> i128 {
        (self.tree[l][pl].get_sum() - self.tree[r][pr].get_sum()).abs()
    }

    fn run_phase(&mut self, p: usize) -> Option<usize> {
        let z = 1 << p;
        self.tree[2 * z..4 * z].sort_unstable_by_key(|c| c[0].get_sum());
        for i in z..2 * z {
            self.heap.clear();
            self.added.clear();
            let (l, r) = (2 * i, 2 * i + 1);
            let mut pr = 0;
            for pl in 0..self.tree[l].len() {
                while pr + 1 < self.tree[r].len()
                    && self.calc_diff(l, r, pl, pr + 1) < self.calc_diff(l, r, pl, pr)
                {
                    pr += 1
                }
                let s = self.calc_diff(l, r, pl, pr);
                self.heap.push((s, pl, pr, false));
                self.added.insert((pl, pr, false));
            }
            {
                let s = self.calc_sum(l, r, 0, 0);
                self.heap.push((s, 0, 0, true));
            }
            let mut last_sum = -1;
            while self.tree[i].len() < self.cluster_size {
                if let Some((s, pl, pr, b)) = self.heap.pop() {
                    if b {
                        if s != last_sum {
                            self.tree[i].push(TreeAttackNode::new_internal(s, i, false, false, pl, pr));
                            last_sum = s;
                        }
                        if pl + 1 < self.tree[l].len() && self.added.insert((pl + 1, pr, true)) {
                            let s = self.calc_sum(l, r, pl + 1, pr);
                            self.heap.push((s, pl + 1, pr, true));
                        }
                        if pr + 1 < self.tree[r].len() && self.added.insert((pl, pr + 1, true)) {
                            let s = self.calc_sum(l, r, pl, pr + 1);
                            self.heap.push((s, pl, pr + 1, true));
                        }
                        if pl + 1 < self.tree[l].len()
                            && pr + 1 < self.tree[r].len()
                            && self.added.insert((pl + 1, pr + 1, true))
                        {
                            let s = self.calc_sum(l, r, pl + 1, pr + 1);
                            self.heap.push((s, pl + 1, pr + 1, true));
                        }
                    } else {
                        let (mut ml, mut mr) = (true, false);
                        if self.tree[l][pl].get_sum() > self.tree[r][pr].get_sum() {
                            ml = !ml;
                            mr = !mr;
                        }
                        if s != last_sum {
                            self.tree[i].push(TreeAttackNode::new_internal(s, i, ml, mr, pl, pr));
                            last_sum = s;
                        }
                        if pr > 0 && self.added.insert((pl, pr - 1, false)) {
                            let s = self.calc_diff(l, r, pl, pr - 1);
                            self.heap.push((s, pl, pr - 1, false));
                        }
                        if pr + 1 < self.tree[r].len() && self.added.insert((pl, pr + 1, false)) {
                            let s = self.calc_diff(l, r, pl, pr + 1);
                            self.heap.push((s, pl, pr + 1, false));
                        }
                    }
                    if s == 0 {
                        return Some(i);
                    }
                } else {
                    break;
                }
            }
        }
        None
    }

    fn construct_solution(&mut self, len: usize, idx: usize) -> (String, String) {
        let mut words = Vec::new();
        words.resize(len, None);
        let mut queue = VecDeque::with_capacity(len);
        queue.push_back((idx, 0, false));
        while !queue.is_empty() {
            let (x, p, m) = queue.remove(0).unwrap();
            match &self.tree[x][p] {
                TreeAttackNode::Internal(n) => {
                    queue.push_back((2 * n.idx, n.pos_left, m != n.rev_left));
                    queue.push_back((2 * n.idx + 1, n.pos_right, m != n.rev_right));
                }
                TreeAttackNode::Leaf(n) => {
                    let (w1, w2) = (n.word1, n.word2);
                    if !m {
                        words[n.idx] = Some((w1, w2));
                    } else {
                        words[n.idx] = Some((w2, w1));
                    }
                }
            };
        }
        let mut rng = rand::thread_rng();
        let cap = len * self.word_len;
        let mut fi = String::with_capacity(cap);
        let mut se = String::with_capacity(cap);
        for word in words {
            if let Some((w1, w2)) = word {
                fi.push_str(w1);
                se.push_str(w2);
            } else {
                let idx = rng.gen_range(0, self.alphabet.len());
                fi.push_str(&self.alphabet[idx]);
                se.push_str(&self.alphabet[idx]);
            }
        }
        return (fi, se);
    }

    fn try_attack(&mut self, p: usize) -> Option<(String, String)> {
        let len = 1 << p;
        self.init_attack(len);
        for i in (0..p).rev() {
            if let Some(idx) = self.run_phase(i) {
                return Some(self.construct_solution(len, idx));
            }
        }
        None
    }
}

pub fn find_single_collision(
    base: u64, module: u64, cluster_size: usize, alphabet: &Vec<String>,
) -> Option<(String, String)> {
    let mut attack = TreeAttack::new(base, module, cluster_size, alphabet);
    for i in 3..12 {
        let coll = attack.try_attack(i);
        if coll.is_some() {
            return coll;
        }
    }
    None
}

pub fn find_collision(
    bases: Vec<u64>, modules: Vec<u64>, cluster_size: usize, init_alphabet: Vec<String>,
) -> Option<(String, String)> {
    let mut alphabet = init_alphabet;
    for (&b, &m) in bases.iter().zip(modules.iter()) {
        if let Some((fi, se)) = find_single_collision(b, m, cluster_size, &alphabet) {
            alphabet = vec![fi, se];
        } else {
            return None;
        }
    }
    let se = alphabet.remove(1);
    let fi = alphabet.remove(0);
    Some((fi, se))
}
