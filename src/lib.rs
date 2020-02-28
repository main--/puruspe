use std::f64::{EPSILON, MIN_POSITIVE};
use std::f64::consts::PI;

// =============================================================================
// Constants
// =============================================================================
const EPS: f64 = EPSILON;
const FPMIN: f64 = MIN_POSITIVE / EPS;
const G: f64 = 5f64;
const N: usize = 7;
const ASWITCH: usize = 100;
const NGAU: usize = 18;
const Y: [f64; 18] = [
    0.0021695375159141994, 0.011413521097787704, 0.027972308950302116,
    0.051727015600492421, 0.082502225484340941, 0.12007019910960293,
    0.16415283300752470, 0.21442376986779355, 0.27051082840644336, 
    0.33199876341447887, 0.39843234186401943, 0.46931971407375483, 
    0.54413605556657973, 0.62232745288031077, 0.70331500465597174, 
    0.78649910768313447, 0.87126389619061517, 0.95698180152629142
];
const W: [f64; 18] = [
    0.0055657196642445571, 0.012915947284065419, 0.020181515297735382,
    0.027298621498568734, 0.034213810770299537, 0.040875750923643261,
    0.047235083490265582, 0.053244713977759692, 0.058860144245324798,
    0.064039797355015485, 0.068745323835736408, 0.072941885005653087,
    0.076598410645870640, 0.079687828912071670, 0.082187266704339706,
    0.084078218979661945, 0.085346685739338721, 0.085983275670394821
];

// =============================================================================
// Incomplete Gamma function
// =============================================================================
/// Incomplete Gamma function P(a,x)
pub fn gammp(a: f64, x: f64) -> f64 {
    assert!(x >= 0f64 && a > 0f64, "Bad args in gammp");
    if x == 0f64 {
        0f64
    } else if (a as usize) >= ASWITCH {
        // Quadrature
        gammpapprox(a,x,IncGamma::P)
    } else if x < a + 1f64 {
        // Series representation
        gser(a,x)
    } else {
        // Continued fraction representation
        1f64 - gcf(a,x)
    }
}

/// Incomplete Gamma function Q(a,x)
pub fn gammq(a: f64, x: f64) -> f64 {
    assert!(x >= 0f64 && a > 0f64, "Bad args in gammp");
    if x == 0f64 {
        1f64
    } else if (a as usize) >= ASWITCH {
        // Quadrature
        gammpapprox(a,x,IncGamma::Q)
    } else if x < a + 1f64 {
        // Series representation
        1f64 - gser(a,x)
    } else {
        // Continued fraction representation
        gcf(a,x)
    }
}

/// Series expansion
fn gser(a: f64, x: f64) -> f64 {
    let gln = ln_gamma_approx(a);
    let mut ap = a;
    let mut del = 1f64 / a;
    let mut sum = 1f64 / a;
    loop {
        ap += 1f64;
        del *= x/ap;
        sum += del;
        if del.abs() < sum.abs() * EPS {
            return sum * (-x + a * x.ln() - gln).exp();
        }
    }
}

fn gcf(a: f64, x: f64) -> f64 {
    let gln = ln_gamma_approx(a);
    let mut b = x + 1f64 - a;
    let mut c = 1f64 / FPMIN;
    let mut d = 1f64 / b;
    let mut h = d;
    let mut an = 0f64;
    for i in 1 .. {
        an = -i as f64 * (i as f64 - a);
        b += 2f64;
        d = an*d + b;
        if d.abs() < FPMIN {
            d = FPMIN;
        }
        c = b + an / c;
        if c.abs() < FPMIN {
            c = FPMIN;
        }
        d = 1f64 / d;
        let del = d * c;
        h *= del;
        if (del - 1f64).abs() < EPS {
            break;
        }
    }
    (-x + a * x.ln() - gln).exp() * h
}

/// Kinds of Incomplete Gamma function
#[derive(Debug, Copy, Clone)]
enum IncGamma {
    P,
    Q
}

/// Gauss Legendre Quadrature (order of 18)
fn gammpapprox(a: f64, x: f64, psig: IncGamma) -> f64 {
    let a1 = a - 1f64;
    let lna1 = a1.ln();
    let sqrta1 = a1.sqrt();
    let gln = ln_gamma_approx(a);
    let xu = if x > a1 {
        (a1 + 11.5 * sqrta1).max(x + 6f64 * sqrta1)
    } else {
        0f64.max((a1 - 7.5 * sqrta1).min(x - 5f64 * sqrta1))
    };
    let mut sum = 0f64;
    let mut t = 0f64;
    for j in 0 .. NGAU {
        t = x + (xu - x) * Y[j];
        sum += W[j] * (-(t-a1) + a1*(t.ln() - lna1)).exp();
    }
    let ans = sum * (xu - x) * (a1 * (lna1 - 1f64).exp() - gln);
    match psig {
        IncGamma::P => {
            if ans > 0f64 {
                1f64 - ans
            } else {
                -ans
            }
        }
        IncGamma::Q => {
            if ans >= 0f64 {
                ans
            } else {
                1f64 + ans
            }
        }
    }
}

// =============================================================================
// Lanczos approximation of Gamma
// =============================================================================
/// Lanczos g=5, n=7
const LG5N7: [f64; 7] = [
    1.000000000189712,
    76.18009172948503,
    -86.50532032927205,
    24.01409824118972,
    -1.2317395783752254,
    0.0012086577526594748,
    -0.00000539702438713199
];

/// Logarithm Gamma
fn ln_gamma_approx(z: f64) -> f64 {
    let z = z - 1f64;
    let base = z + G + 0.5;
    let mut s = 0f64;
    for i in 1 .. N {
        s += LG5N7[i] / (z + i as f64);
    }
    s += LG5N7[0];
    (2f64 * PI).sqrt().ln() + s.ln() - base + base.ln() * (z + 0.5)
}

/// Gamma function
pub fn gamma_approx(z: f64) -> f64 {
    if z > 1f64 {
        let z_int = z as usize;
        if z - (z_int as f64) == 0f64 {
            return factorial(z_int-1) as f64;
        }
    }

    if z < 0.5 {
        PI / ((PI * z).sin() * gamma_approx(1f64 - z))
    } else {
        ln_gamma_approx(z).exp()
    }
}

/// Just factorial
pub fn factorial(n: usize) -> usize {
    let mut p = 1usize;
    for i in 1..(n + 1) {
        p *= i;
    }
    p
}