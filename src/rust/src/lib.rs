use extendr_api::prelude::*;
use bit_vec::BitVec;

use fasthash::{MurmurHasher, FastHasher};
use std::hash::{Hash, Hasher};

use std::io::{Write, Read};
use std::fs::File;

use serde::{Serialize, Deserialize};
use serde_json;

#[extendr]
#[derive(Serialize, Deserialize, Debug)]
struct BloomFilter{
    bit_vec : BitVec,
    n_hashes : usize,
    n_bits : usize,
    n_stored : usize,
}

#[extendr]
impl BloomFilter {
    fn new(filter_size : u64, n_hashes : u64) -> Self{
        Self {
            bit_vec : BitVec::from_elem(filter_size as usize,false),
            n_hashes: n_hashes as usize,
            n_bits : filter_size as usize,
            n_stored : 0,
        }
    }

    /// this is a test
    fn add(&mut self, strings : Robj) {
        let strings = <Vec<String>>::from_robj(&strings).unwrap();
        self.n_stored += strings.len();
        for string in strings {
            for i in 0..self.n_hashes {
                 let mut s: MurmurHasher = MurmurHasher::with_seed(i as u32);

                 string.hash(&mut s);

                 let bit_loc = ((s.finish() % self.n_bits as u64) + self.n_bits as u64) % self.n_bits as u64;

                 self.bit_vec.set(bit_loc as usize, true)

            }
        }
    }

    fn check(&self, strings : Robj) -> Vec<bool>{
        let strings = <Vec<String>>::from_robj(&strings).unwrap();
        let mut out = vec![true; strings.len()];

        'outer : for (i, string) in strings.iter().enumerate() {
            for j in 0..self.n_hashes {
                 let mut s: MurmurHasher = MurmurHasher::with_seed(j as u32);

                 string.hash(&mut s);

                 let bit_loc = ((s.finish() % self.n_bits as u64) + self.n_bits as u64) % self.n_bits as u64;

                 if ! self.bit_vec.get(bit_loc as usize).unwrap() {
                     out[i] = false;
                     continue 'outer;
                 };
            }
        }

        out
    }

    fn frac_filled(&self) -> f64{
        self.bit_vec.iter().map(|x| x as usize).sum::<usize>() as f64 / self.n_bits as f64
    }

    fn print(&self) {
        println!("A BloomFilter with a capacity of {} bits.", self.n_bits);
        println!("Currently Storing {} values with {} hashes", self.n_stored, self.n_hashes);
    }

    fn clear(&mut self) {
        self.bit_vec.clear()
    }

    fn save(&self, path : &str){
        let mut f = File::create(path).unwrap();
        let buf = serde_json::to_vec(&self).unwrap();
        f.write_all(&buf[..]).unwrap();
    }

    fn load(path : &str) -> Self {
        let mut file = File::open(path).expect("failed to open file");

        let mut buf = vec![];
        file.read_to_end(&mut buf).expect("could not read file");

        serde_json::from_slice(&buf[..]).expect("could not parse saved filter")
    }
}


// Macro to generate exports.
// This ensures exported functions are registered with R.
// See corresponding C code in `entrypoint.c`.
extendr_module! {
    mod BloomeR;
    impl BloomFilter;
}
