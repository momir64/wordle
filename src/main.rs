// 0 - nije isprobano
// 1 - nije u reči
// 2 - negde je u reči
// 3 - na dobrom je mestu

use std::cmp::min;
use std::fs;
use std::fs::File;
use serde_json::Value;
use std::time::Instant;
use std::io::prelude::*;
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::ptr::hash;

fn trie_to_dict() {
    let contents = fs::read_to_string("trie.json").expect("Bad file");
    let data: serde_json::Value = serde_json::from_str(&contents).expect("Bad json");
    let file = File::create("dict.txt").expect("Bad file");

    print_word(&"".to_string(), &data, &file);
    fn print_word(word: &String, data: &Value, mut file: &File) {
        if data.as_object() == None {
            writeln!(file, "{}", word).expect("File error");
        } else {
            for letter in data.as_object().unwrap().iter() {
                let mut tmp = word.clone();
                tmp.push_str(&letter.0.to_uppercase());
                tmp.push(' ');
                print_word(&tmp, letter.1, &file);
            }
        }
    }
}

fn generate_combinations() -> Vec<[i8; 5]> {
    fn recursive(n: i8, combination: [i8; 5], combinations: &mut Vec<[i8; 5]>) {
        if n == 0 {
            combinations.push(combination);
            return;
        }
        for i in 1..4 {
            let mut tmp = combination.clone();
            tmp[5 - n as usize] = i;
            recursive(n - 1, tmp, combinations);
        }
    }

    let mut combinations: Vec<[i8; 5]> = Vec::new();
    let combination: [i8; 5] = [0; 5];
    recursive(5, combination, &mut combinations);

    return combinations;
}

fn check_combination_match(word: &[char; 5], test_word: &[char; 5], combination: &[i8; 5]) -> bool {
    for i in 0..5 {
        if combination[i as usize] == 3 && word[i as usize] != test_word[i as usize] {
            return false;
        } else if combination[i as usize] == 2 && word[i as usize] == test_word[i as usize] {
            return false;
        } else if combination[i as usize] == 1 {
            for letter in word.clone() {
                if test_word[i as usize] == letter {
                    return false;
                }
            }
        } else if combination[i as usize] == 2 {
            let mut sum = 0;
            for letter in test_word.clone() {
                if test_word[i as usize] == letter {
                    sum += 1;
                }
            }
            for letter in word.clone() {
                if test_word[i as usize] == letter {
                    sum -= 1;
                }
            }
            if sum != 0 {
                return false;
            }
        }
    }

    return true;
}

fn get_percentage(guess: &[char; 5], combination: &[i8; 5], dict: &Vec<[char; 5]>, size: &usize, index: &HashMap<[char; 5], usize>, lookup: &Vec<[i8; 5]>) -> f64 {
    let mut sum = 0;
    for word in dict {
        if &lookup[index[word] * size + index[guess]] == combination {
            sum += 1;
        }
    }
    return sum as f64 / dict.len() as f64;
}

fn get_entropy(guess: &[char; 5], dict: &Vec<[char; 5]>, combinations: &Vec<[i8; 5]>, size: &usize, index: &HashMap<[char; 5], usize>, lookup: &Vec<[i8; 5]>) -> f64 {
    let mut sum = 0.0;
    for combination in combinations {
        let p = get_percentage(guess, combination, dict, size, index, lookup);
        sum += if p != 0.0 { p * (1.0 / p).log2() } else { 0.0 };
    }
    return sum;
}

fn print_scoreboard(scoreboard: &Vec<([char; 5], f64)>) {
    print!("{esc}c", esc = 27 as char);
    for score in &scoreboard[0..min(20, scoreboard.len())] {
        println!("{:?}  {}", score.0, score.1);
    }
}

fn make_combination(word: &[char; 5], guess: &[char; 5]) -> [i8; 5] {
    let mut combination: [i8; 5] = [0; 5];
    for i in 0..5 {
        if word[i] == guess[i] {
            combination[i] = 3;
        } else {
            let mut tmp = 0;
            for j in 0..5 {
                if word[j] == guess[i] {
                    tmp += 1;
                }
                if guess[j] == guess[i] && j < i {
                    tmp -= 1;
                }
            }
            combination[i] = if tmp > 0 { 2 } else { 1 };
        }
    }
    return combination;
}

fn make_lookup(size: &usize, index: &HashMap<[char; 5], usize>, dict: &Vec<[char; 5]>) -> Vec<[i8; 5]> {
    let mut lookup = vec![[0, 0, 0, 0, 0]; size * size];
    let mut br = 0;
    let mut now = Instant::now();
    for word in dict {
        br += 1;

        for guess in dict {
            lookup[index[word] * size + index[guess]] = make_combination(word, guess);
        }

        if br % 200 == 0 {
            let elapsed = now.elapsed();
            print!("{esc}c", esc = 27 as char);
            println!("{}\tElapsed: {:.2?}", br, elapsed);
            now = Instant::now();
        }
    }
    return lookup;
}

fn main() {
    // trie_to_dict();
    let file = File::open("dict.txt").expect("Bad file");
    let lines: Vec<String> = BufReader::new(file).lines().collect::<Result<_, _>>().unwrap();
    let dict: Vec<[char; 5]> = lines.iter().map(|line| line.chars().collect::<Vec<_>>().try_into().unwrap()).collect();

    let combinations: Vec<[i8; 5]> = generate_combinations();

    let size = dict.len();
    let mut index: HashMap<[char; 5], usize> = HashMap::new();
    for i in 0..dict.len() {
        index.insert(dict[i], i);
    }
    let lookup: Vec<[i8; 5]> = make_lookup(&size, &index, &dict);

    serde_json::to_string(&lookup).unwrap();

    let mut scoreboard: Vec<([char; 5], f64)> = Vec::new();

    for word in dict.clone() {
        let now = Instant::now();
        scoreboard.push((word, get_entropy(&word, &dict, &combinations, &size, &index, &lookup)));
        // scoreboard.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap().reverse());
        // print_scoreboard(&scoreboard);
        let elapsed = now.elapsed();
        println!("Elapsed: {:.2?}", elapsed);
    }
}


