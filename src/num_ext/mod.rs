use num::bigint::{ToBigUint, RandBigInt, BigUint};
use num::{Zero, One};
use num::integer::Integer;
use rand::thread_rng;
use std::sync::{Arc, mpsc};
use std::thread;

/// Cryptographically useful extensions to the provided BigUint functionality.
pub trait BigUintCrypto {
    /// Find the next prime from the current BigUint
    fn next_prime(&self) -> BigUint;

    /// Threaded version of the next_prime() operation, this is not recommended for use because it
    /// is slower than the unthreaded version.
    fn next_prime_threaded(&self) -> BigUint;
    /// use the extended euclidean algorithm to solve for (g,x,y) given (a,b) such that
    /// g = gcd(a,b) = a*x + b*y.
    fn gcdext(&self, other: &BigUint) -> (BigUint, BigUint, BigUint);

    /// Is this number a prime number. Uses a probablistic function to determine primality.
    fn is_prime(n: &BigUint) -> bool;

    /// perform the function (base^exponent) % modulus using exponentiation by sqauring
    fn mod_exp(base: &BigUint, exponent: &BigUint, modulus: &BigUint) -> BigUint;
}

impl BigUintCrypto for BigUint {
    fn next_prime(&self) -> BigUint {
        next_prime_helper(&self.clone(), false)
    }

    fn next_prime_threaded(&self) -> BigUint {
        next_prime_helper(&self.clone(), true)
    }

    fn gcdext(&self, other: &BigUint) -> (BigUint, BigUint, BigUint) {

        (Zero::zero(), Zero::zero(), Zero::zero())
    }

    fn is_prime(n: &BigUint) -> bool {
        is_prime_helper(n, false)
    }

    fn mod_exp(base: &BigUint, exponent: &BigUint, modulus: &BigUint) -> BigUint {
        let zero = Zero::zero();
        let one: BigUint = One::one();
        let two = &one + &one;
        let mut result: BigUint = One::one();
        let mut base_acc = base.clone();
        let mut exp_acc = exponent.clone();
        while exp_acc > zero {
            if (&exp_acc % &two) == one {
                result = (result * &base_acc) % modulus;
            }
            exp_acc = exp_acc >> 1;
            base_acc = (&base_acc * &base_acc) % modulus;
        }
        result
    }
}

fn next_prime_helper(n: &BigUint, thread: bool) -> BigUint {
    let one: BigUint = One::one();
    let two = 2.to_biguint().unwrap();
    let mut next_prime = n.clone();
    if &next_prime % &two == Zero::zero() {
        next_prime = &next_prime + &one;
    } else {
        next_prime = &next_prime + &two;
    }
    while !is_prime_helper(&next_prime, thread) {
        next_prime = &next_prime + &two;
    }
    next_prime
}

fn is_prime_helper(n: &BigUint, thread: bool) -> bool {
    let two = 2.to_biguint().unwrap();
    let three = 3.to_biguint().unwrap();
    if *n == three || *n == two {
        return true;
    }
    if *n < two || n % two == Zero::zero() {
        return false;
    }
    miller_rabin(n, 100, thread)
}
/// n must be greater than 3 and k indicates the number of rounds
fn miller_rabin(n: &BigUint, k: usize, thread: bool) -> bool{
    let one: BigUint = One::one();
    let (tx, rx) = mpsc::channel();

    let mut d: BigUint = n - &One::one();
    let mut s: BigUint = Zero::zero();
    while d.is_even() {
        d = d >> 1;
        s = s + &one;
    }
    if thread {
        let shared_n = Arc::new(n.clone());
        let shared_d = Arc::new(d);
        let shared_s = Arc::new(s);

        // miller rabin lends itself to being concurrent since a is completely random
        // here we spawn multiple threads to help speed up the process
        for _ in 0..8 {
            let tx = tx.clone();
            //let thread_n = n.clone();
            let shared_d = shared_d.clone();
            let shared_s = shared_s.clone();
            let shared_n = shared_n.clone();
            thread::spawn(move || {
                let in_n = shared_n;
                let in_d = shared_d;
                let in_s = shared_s;
                let result = miller_rabin_thread(&in_n, &in_d, &in_s, k/8);
                tx.send(result);
                });
        }

        let mut prime = true;
        for _ in 0..8 {
            if !rx.recv().ok().expect("A thread failed") {
                prime = false;
            }
        }
        prime
    } else {
        return miller_rabin_thread(n, &d, &s, k);
    }
}

fn miller_rabin_thread(n: &BigUint, d: &BigUint, s: &BigUint, k: usize) -> bool {
    let one: BigUint = One::one();
    let two: BigUint = &one + &one;

    for _ in 0..k {
        //println!("loop {} of {}", j, k);
        let a = thread_rng().gen_biguint_range(&two, &(n - &two));
        let mut x = mod_exp(&a, d, n);
        //let mut x = two.clone();
        if (x == one) || (x == (n - &one)) {
            continue;
        }

        // Use a while loop instead of for here because range does not accept BigUint
        let mut i: BigUint = Zero::zero();
        loop  {
            x = mod_exp(&x, &two, n);
            if x == one || i == (s - &one) {
                return false;
            }
            if x == (n - &one) {
                break;
            }
            i = i + &one;
        }
    }
    true
}

fn mod_exp(base: &BigUint, exponent: &BigUint, modulus: &BigUint) -> BigUint {
    let zero = Zero::zero();
    let one: BigUint = One::one();
    let two = &one + &one;
    let mut result: BigUint = One::one();
    let mut base_acc = base.clone();
    let mut exp_acc = exponent.clone();
    while exp_acc > zero {
        if (&exp_acc % &two) == one {
            result = (result * &base_acc) % modulus;
        }
        exp_acc = exp_acc >> 1;
        base_acc = (&base_acc * &base_acc) % modulus;
    }
    result
}

#[cfg(test)]
mod test_BigUint_crypto {
    use super::{BigUintCrypto, mod_exp, miller_rabin};
    use num::bigint::{ToBigUint, RandBigInt, BigUint};
    use num::One;
    use rand::thread_rng;
    use test::Bencher;
    use std::sync::{Arc, mpsc};
    use std::thread;

    #[test]
    fn next_prime_test() {
        let test_num = BigUint::
        parse_bytes("4829837983753984028472098472089547098728675098723407520875258".as_bytes(), 10).unwrap();

        let expected_next = BigUint::
        parse_bytes("4829837983753984028472098472089547098728675098723407520875297".as_bytes(), 10).unwrap();

        assert!(test_num.next_prime() == expected_next);
    }

    #[test]
    fn next_prime_threaded_test() {
        let test_num = BigUint::
        parse_bytes("4829837983753984028472098472089547098728675098723407520875258".as_bytes(), 10).unwrap();

        let expected_next = BigUint::
        parse_bytes("4829837983753984028472098472089547098728675098723407520875297".as_bytes(), 10).unwrap();

        assert!(test_num.next_prime_threaded() == expected_next);
    }

    #[test]
    fn mod_exp_test() {
        let base = 4.to_biguint().unwrap();
        let exponent = 13.to_biguint().unwrap();
        let modulus = 497.to_biguint().unwrap();
        let expected_result = 445.to_biguint().unwrap();

        assert!(mod_exp(&base, &exponent, &modulus) == expected_result);
    }

    #[test]
    fn is_prime_test() {
        let known_prime = BigUint::
        parse_bytes("4829837983753984028472098472089547098728675098723407520875297".as_bytes(), 10).unwrap();

        assert!(BigUint::is_prime(&known_prime));
    }

    #[test]
    #[should_panic(expected = "assertion failed")]
    fn is_prime_test_failuire() {
        let not_prime = BigUint::
        parse_bytes("359709793871987301975987296195681798740165298740176567105918720469720137416098423"
        .as_bytes(), 10).unwrap();

        assert!(BigUint::is_prime(&not_prime));
    }

    #[bench]
    fn bench_spawn_thread_move(bench: &mut Bencher) {

        bench.iter(|| {
            let a = thread_rng().gen_biguint(300);
            let b = thread_rng().gen_biguint(300);
            let c = thread_rng().gen_biguint(300);
            let shared_a = Arc::new(a);
            let shared_b = Arc::new(b);
            let shared_c = Arc::new(c);

            thread::spawn(move || {
                let in_a = shared_a;
                let in_b = shared_b;
                let in_c = shared_c;
                });
            });
    }

    #[bench]
    fn bench_rand_biguint(bench: &mut Bencher) {
        bench.iter(|| {
            thread_rng().gen_biguint(300)
            });
    }

    #[bench]
    fn bench_spawn_thread(bench: &mut Bencher) {
        bench.iter(|| {
            thread::spawn(|| {
                let a = 4;
                });
            });
    }

    #[bench]
    fn bench_mod_exp(bench: &mut Bencher) {
        let a = thread_rng().gen_biguint(300);
        let b = thread_rng().gen_biguint(300);
        let c = thread_rng().gen_biguint(300);
        bench.iter(|| {
            mod_exp(&a, &b, &c);
            });
    }

    #[bench]
    fn bench_miller_rabin(bench: &mut Bencher) {
        let known_prime = BigUint::
        parse_bytes("4829837983753984028472098472089547098728675098723407520875297".as_bytes(), 10).unwrap();
        bench.iter(|| {
            miller_rabin(&known_prime, 100, false)
            });
    }

    #[bench]
    fn bench_miller_rabin_thread(bench: &mut Bencher) {
        let known_prime = BigUint::
        parse_bytes("4829837983753984028472098472089547098728675098723407520875297".as_bytes(), 10).unwrap();
        bench.iter(|| {
            miller_rabin(&known_prime, 100, true)
            });
    }

    #[bench]
    fn bench_next_prime(bench: &mut Bencher) {
        let test_num = BigUint::
        parse_bytes("4829837983753984028472098472089547098728675098723407520875258".as_bytes(), 10).unwrap();

        bench.iter(|| {
            test_num.next_prime()
            });
    }

    #[bench]
    fn bench_next_prime_threaded(bench: &mut Bencher) {
        let test_num = BigUint::
        parse_bytes("4829837983753984028472098472089547098728675098723407520875258".as_bytes(), 10).unwrap();

        bench.iter(|| {
            test_num.next_prime_threaded()
            });
    }

}
