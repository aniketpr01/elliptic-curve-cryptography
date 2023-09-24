use num_bigint::BigUint;
use std::{clone, iter::Product};

#[derive(Debug, Clone, PartialEq)]
enum Point {
    CoOrdinates(BigUint, BigUint),
    Identity,
}

struct EllipticCurve {
    // y^2 = x^3 + a * x + b
    a: BigUint,
    b: BigUint,
    p: BigUint,
}

impl EllipticCurve {
    fn point_add(&self, m: &Point, n: &Point) -> Point {
        assert!(self.is_it_on_curve(m), "Point is not in curve");
        assert!(self.is_it_on_curve(n), "Point is not in curve");

        match (m, n) {
            (Point::Identity, _) => n.clone(),
            (_, Point::Identity) => m.clone(),
            (Point::CoOrdinates(x1, y1), Point::CoOrdinates(x2, y2)) => {
                let y1_plus_y2 = FiniteField::add(&y1, &y2, &self.p);
                if x1 == x2 && y1_plus_y2 == BigUint::from(0u32) {
                    return Point::Identity;
                }
                // s = (y2 - y1) / (x2 -x1) mod p
                // x3 = s^2 - x1 - x2 mod p
                // y3 = s(x1 - x3) - y1 mod p
                let numerator = FiniteField::subtract(y2, y1, &self.p);
                let denominator = FiniteField::subtract(x2, x1, &self.p);
                let s = FiniteField::division(&numerator, &denominator, &self.p);

                let (x3, y3) = self.compute_x3_y3(&x1, &y1, &x2, &s);
                Point::CoOrdinates(x3, y3)
            }
        }
    }

    fn point_double(&self, m: &Point) -> Point {
        assert!(self.is_it_on_curve(m), "Point is not on curve");

        match m {
            Point::Identity => Point::Identity,
            Point::CoOrdinates(x1, y1) => {
                // s = (y2 - y1) / (x2 -x1) mod p
                // x3 = s^2 - x1 - x2 mod p
                // y3 = s(x1 - x3) - y1 mod p
                let numerator = x1.modpow(&BigUint::from(2u32), &self.p);
                let numerator = FiniteField::mul(&BigUint::from(3u32), &numerator, &self.p);
                let numerator = FiniteField::add(&self.a, &numerator, &self.p);
                let denominator = FiniteField::mul(&BigUint::from(2u32), y1, &self.p);
                let s = FiniteField::division(&numerator, &denominator, &self.p);

                let (x3, y3) = self.compute_x3_y3(&x1, &y1, &x1, &s);
                Point::CoOrdinates(x3, y3)
            }
        }
    }

    fn compute_x3_y3(
        &self,
        x1: &BigUint,
        y1: &BigUint,
        x2: &BigUint,
        s: &BigUint,
    ) -> (BigUint, BigUint) {
        let s2 = s.modpow(&BigUint::from(2u32), &self.p);
        let s2_minus_x1 = FiniteField::subtract(&s2, x1, &self.p);
        let x3 = FiniteField::subtract(&s2_minus_x1, x2, &self.p);
        let x1_minus_x3 = FiniteField::subtract(x1, &x3, &self.p);
        let s_x1_minus_x3 = FiniteField::mul(&s, &x1_minus_x3, &self.p);
        let y3 = FiniteField::subtract(&s_x1_minus_x3, &y1, &self.p);
        (x3, y3)
    }

    fn point_scalar_mul(&self, m: &Point, n: &BigUint) -> Point {
        // addition/doubling algorithm
        // B = d * A
        // square and multiply algorithm essentially, also called as double and add
        // T = A
        // for i in range(0..bits of d - 1)
        //      T = 2 * T
        //      if bit i of d == 1
        //          T = T + A
        // return T
        let mut t = m.clone();
        for i in (0..(n.bits() - 1)).rev() {
            t = self.point_double(&t);
            if n.bit(i) {
                t = self.point_add(&t, &m);
            }
        }
        t
    }

    fn is_it_on_curve(&self, m: &Point) -> bool {
        // y^2 = x^3 + a * x + b
        // either m is identity or it is a scalar coordinates
        match m {
            Point::CoOrdinates(x, y) => {
                let y2 = y.modpow(&BigUint::from(2u32), &self.p);
                let x3 = x.modpow(&BigUint::from(3u32), &self.p);
                let ax = FiniteField::mul(&self.a, x, &self.p);
                let x3_plus_ax = FiniteField::add(&x3, &ax, &self.p);
                y2 == FiniteField::add(&x3_plus_ax, &self.b, &self.p)
            }
            Point::Identity => true,
        }
        // let y2 = m
    }
}

struct FiniteField {}

impl FiniteField {
    fn add(m: &BigUint, n: &BigUint, p: &BigUint) -> BigUint {
        // add -> m + n = r mod p
        let r = m + n;
        r.modpow(&BigUint::from(1u32), p)
        // or we can use below expression which also works
        // r % p
    }

    fn inv_add(m: &BigUint, p: &BigUint) -> BigUint {
        // -c mod p
        assert!(m < p, "number: {} is bigger or equal than p: {}", m, p);
        p - m
    }

    fn subtract(m: &BigUint, n: &BigUint, p: &BigUint) -> BigUint {
        // m - n
        let n_inv = FiniteField::inv_add(n, p);
        FiniteField::add(m, &n_inv, p)
    }

    fn mul(m: &BigUint, n: &BigUint, p: &BigUint) -> BigUint {
        // mul -> m * n = r mod p
        let r = m * n;
        r.modpow(&BigUint::from(1u32), p)
        // or we can use below expression which also works
        // r % p
    }

    fn inv_mul(m: &BigUint, p: &BigUint) -> BigUint {
        // function is only valid for a prime p
        // using fermat's little theorem
        // c^(-1) mod p = c^(p-2) mod p
        m.modpow(&(p - BigUint::from(2u32)), p)
    }

    fn division(m: &BigUint, n: &BigUint, p: &BigUint) -> BigUint {
        // m / n
        let n_inv = FiniteField::inv_mul(n, p);
        FiniteField::mul(m, &n_inv, p)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_add_1() {
        let m = BigUint::from(4u32);
        let n = BigUint::from(10u32);
        let p = BigUint::from(11u32);
        let r = FiniteField::add(&m, &n, &p);
        assert_eq!(r, BigUint::from(3u32));
    }

    #[test]
    fn test_add_2() {
        let m = BigUint::from(4u32);
        let n = BigUint::from(10u32);
        let p = BigUint::from(31u32);
        let r = FiniteField::add(&m, &n, &p);
        assert_eq!(r, BigUint::from(14u32));
    }

    #[test]
    fn test_mul_1() {
        let m = BigUint::from(4u32);
        let n = BigUint::from(10u32);
        let p = BigUint::from(11u32);
        let r = FiniteField::mul(&m, &n, &p);
        assert_eq!(r, BigUint::from(7u32));
    }

    #[test]
    fn test_mul_2() {
        let m = BigUint::from(4u32);
        let n = BigUint::from(10u32);
        let p = BigUint::from(61u32);
        let r = FiniteField::mul(&m, &n, &p);
        assert_eq!(r, BigUint::from(40u32));
    }

    #[test]
    fn test_subtract() {
        let m = BigUint::from(4u32);
        let p = BigUint::from(61u32);
        assert_eq!(FiniteField::subtract(&m, &m, &p), BigUint::from(0u32));
    }

    #[test]
    fn test_divide() {
        let m = BigUint::from(4u32);
        let p = BigUint::from(61u32);
        assert_eq!(FiniteField::division(&m, &m, &p), BigUint::from(1u32));
    }

    #[test]
    fn test_inv_add_1() {
        let m = BigUint::from(4u32);
        let p = BigUint::from(51u32);
        let r = FiniteField::inv_add(&m, &p);
        assert_eq!(r, BigUint::from(47u32));
    }

    #[test]
    #[should_panic]
    fn test_inv_add_2() {
        let m = BigUint::from(52u32);
        let p = BigUint::from(51u32);
        let r = FiniteField::inv_add(&m, &p);
        assert_eq!(r, BigUint::from(47u32));
    }

    #[test]
    fn test_inv_add_identity() {
        let m = BigUint::from(4u32);
        let p = BigUint::from(51u32);
        let m_inv = FiniteField::inv_add(&m, &p);
        assert_eq!(FiniteField::add(&m, &m_inv, &p), BigUint::from(0u32));
    }

    #[test]
    fn test_inv_mul_identity() {
        let m = BigUint::from(4u32);
        let p = BigUint::from(11u32);
        let m_inv = FiniteField::inv_mul(&m, &p);
        assert_eq!(FiniteField::mul(&m, &m_inv, &p), BigUint::from(1u32));
    }

    #[test]
    fn test_ec_point_addition() {
        // y^2 = x^3 + 2x + 2 mod 17
        let curve = EllipticCurve {
            a: BigUint::from(2u32),
            b: BigUint::from(2u32),
            p: BigUint::from(17u32),
        };
        // (6,3) + (5,1) = (10, 6)
        let m = Point::CoOrdinates(BigUint::from(6u32), BigUint::from(3u32));
        let n = Point::CoOrdinates(BigUint::from(5u32), BigUint::from(1u32));
        let r = Point::CoOrdinates(BigUint::from(10u32), BigUint::from(6u32));
        let res = curve.point_add(&m, &n);
        assert_eq!(res, r);
    }

    #[test]
    fn test_ec_point_addition_identity() {
        // y^2 = x^3 + 2x + 2 mod 17
        let curve = EllipticCurve {
            a: BigUint::from(2u32),
            b: BigUint::from(2u32),
            p: BigUint::from(17u32),
        };
        // (6,3) + (5,1) = (10, 6)
        let m = Point::CoOrdinates(BigUint::from(6u32), BigUint::from(3u32));
        let n = Point::Identity;
        let r = m.clone();

        let res = curve.point_add(&m, &n);
        assert_eq!(res, r);

        let res = curve.point_add(&n, &m);
        assert_eq!(res, r);
    }

    #[test]
    fn test_ec_point_doubling() {
        // y^2 = x^3 + 2x + 2 mod 17
        let curve = EllipticCurve {
            a: BigUint::from(2u32),
            b: BigUint::from(2u32),
            p: BigUint::from(17u32),
        };
        // (5,1) + (5,1) = 2(5, 1) = (6,3)
        let m = Point::CoOrdinates(BigUint::from(5u32), BigUint::from(1u32));
        let r = Point::CoOrdinates(BigUint::from(6u32), BigUint::from(3u32));

        let res = curve.point_double(&m);
        assert_eq!(res, r);
    }

    #[test]
    fn test_ec_scalar_multiplication() {
        // y^2 = x^3 + 2x + 2 mod 17        |G| = 19     19 * A = I
        let curve = EllipticCurve {
            a: BigUint::from(2u32),
            b: BigUint::from(2u32),
            p: BigUint::from(17u32),
        };

        let c = Point::CoOrdinates(BigUint::from(5u32), BigUint::from(1u32));
        // 2 (5,1) = (6,3)
        let m = Point::CoOrdinates(BigUint::from(6u32), BigUint::from(3u32));
        let res = curve.point_scalar_mul(&c, &BigUint::from(2u32));
        assert_eq!(res, m);
    }

    #[test]
    fn test_ec_secp256k1() {
        /*
        Prime number
        p = FFFFFFFFF FFFFFFFFF FFFFFFFFF FFFFFFFjjFF FFFFFFFF FFFFFFFF FFFFFFFF FFFFFC2F
        Order of the group, also a prime number, means every element will be generator of the group
        n = FFFFFFFFF FFFFFFFFF FFFFFFFFF FFFFFFFFF BAAEDCE6 AF48A03B BFD25E8C D0364141
        Generator point
        G = (
            X = 79BE667E F9DCBBAC 55A06295 CE870B07 029BFCDB 2DCE28D9 59F2815B 16F81798,
            y = 483ADA77 26A3C465 5DA4FBFC 0E1108A8 FD178448 A6855419 9C47D08F FB10D4B8
        )
        a = 00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000
        b = 00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000007
        y^2 = x^3 + 7
        */

        // n * G = I(Identity)
        let p = BigUint::parse_bytes(
            b"FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F",
            16,
        )
        .expect("could not convert p");
        let n = BigUint::parse_bytes(
            b"FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141",
            16,
        )
        .expect("could not convert n");

        let gx = BigUint::parse_bytes(
            b"79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798",
            16,
        )
        .expect("could not convert gx");
        let gy = BigUint::parse_bytes(
            b"483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8",
            16,
        )
        .expect("could not convert gy");

        let ec = EllipticCurve {
            a: BigUint::from(0u32),
            b: BigUint::from(7u32),
            p: p,
        };

        let g = Point::CoOrdinates(gx, gy);
        let res = ec.point_scalar_mul(&g, &n); // n * G

        assert_eq!(res, Point::Identity);
    }
}
