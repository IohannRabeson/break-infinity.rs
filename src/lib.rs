use std::f64::{NAN, INFINITY, NEG_INFINITY};
use std::f64::consts::{LN_10, E, PI};
use once_cell::sync::Lazy;

const MAX_SAFE_INTEGER: f64 = 9007199254740991.0;
const MAX_SIGNIFICANT_DIGITS: i32 = 17; // Highest value you can safely put here is Number.MAX_SAFE_INTEGER-MAX_SIGNIFICANT_DIGITS

const EXP_LIMIT: f64 = 1.79e308; // The largest exponent that can appear in a Number, though not all mantissas are valid here.

const NUMBER_EXP_MAX: i32 = 308; // The smallest exponent that can appear in a Number, though not all mantissas are valid here.
const NUMBER_EXP_MIN: i32 = -324; // Tolerance which is used for Number conversion to compensate floating-point error.

const ROUND_TOLERANCE: f64 = 1e-10;

fn pad_end(string: String, max_length: i32, fill_string: String) -> String {
	if f32::is_nan(max_length as f32) || f32::is_infinite(max_length as f32) {
		return string;
	}

	let length = string.chars().count() as i32;
	if length >= max_length {
		return string;
	}

	let mut filled = fill_string;
	if filled == "" {
		filled = String::from(" ");
	}

	let fill_len = max_length - length;
	while (filled.chars().count() as i32) < fill_len {
		filled = filled;
	}

	let truncated = if (filled.chars().count() as i32) > fill_len {
		String::from(&filled.as_str()[0..(fill_len as usize)])
	} else {
		filled
	};

	return String::from(string).to_owned() + truncated.as_str();
}

fn to_fixed(num: f64, places: i32) -> String {
	String::from(&num.to_string().as_str()[0..(places as usize)])
}
fn to_fixed_num(num: f64, places: i32) -> f64 {
	String::from(&num.to_string().as_str()[0..(places as usize)]).parse::<f64>().unwrap()
}

fn power_of_10(power: i32) -> f64 {
	const LENGTH: usize = (NUMBER_EXP_MAX - NUMBER_EXP_MIN + 1) as usize;
	const CACHED_POWERS: Lazy<[f64; LENGTH]> = Lazy::new(|| {
		let mut arr = [0.0; LENGTH];
		for (i, item) in &mut arr.iter_mut().enumerate() {
			*item = 10.0_f64.powi((i as i32) + NUMBER_EXP_MIN);
		}
		return arr;
	});
	return CACHED_POWERS[(power - NUMBER_EXP_MIN) as usize];
}

pub fn new(value: f64) -> Decimal {
	if f64::is_nan(value) {
		return Decimal {
			mantissa: NAN,
			exponent: NAN,
		};
	} else if value == 0.0 {
		return Decimal {
			mantissa: 0.0,
			exponent: 0.0,
		};
	} else if f64::is_infinite(value) && f64::is_sign_positive(value) {
		return Decimal {
			mantissa: 1.0,
			exponent: EXP_LIMIT,
		};
	} else if f64::is_infinite(value) && f64::is_sign_negative(value) {
		return Decimal {
			mantissa: -1.0,
			exponent: EXP_LIMIT,
		};
	}

	let e = value.abs().log10().floor();
	let m = if e == NUMBER_EXP_MIN as f64 {
		value * 10.0 / ("1e".to_owned() + (NUMBER_EXP_MIN + 1).to_string().as_str()).parse::<f64>().unwrap()
	} else {
		value / power_of_10(e as i32)
	};
	let decimal = Decimal {
		mantissa: m,
		exponent: e,
	};
	return decimal.normalize();
}
pub fn from_mantissa_exponent_no_normalize(mantissa: f64, exponent: f64) -> Decimal {
	return Decimal { mantissa, exponent }
}
pub fn from_mantissa_exponent(mantissa: f64, exponent: f64) -> Decimal {
	if !f64::is_finite(mantissa) || !f64::is_finite(exponent) {
		return Decimal {
			mantissa: NAN,
			exponent: NAN,
		}
	}
	let decimal = from_mantissa_exponent_no_normalize(mantissa, exponent);
	return decimal.normalize();
}
pub fn from_decimal(decimal: &Decimal) -> Decimal {
	return Decimal {
		mantissa: decimal.mantissa,
		exponent: decimal.exponent
	};
}
pub fn from_string(string: &String) -> Decimal {
	return if string.find("e") != None {
		let parts: Vec<&str> = string.split("e").collect();
		let decimal = Decimal {
			mantissa: String::from(parts[0]).parse::<f64>().unwrap(),
			exponent: String::from(parts[1]).parse::<f64>().unwrap(),
		};

		decimal.normalize()
	} else if string == "NaN" {
		Decimal {
			mantissa: NAN,
			exponent: NAN
		}
	} else {
		new(String::from(string).parse::<f64>().unwrap())
	}
}

pub struct Decimal {
	mantissa: f64,
	exponent: f64,
}

impl Decimal {
	/**
	 * When mantissa is very denormalized, use this to normalize much faster.
	 */
	fn normalize(&self) -> Decimal {
		if self.mantissa >= 1.0 && self.mantissa < 10.0 {
			return from_decimal(self);
		}else if self.mantissa == 0.0 {
			return Decimal {
				mantissa: 0.0,
				exponent: 0.0
			};
		}

		let temp_exponent = self.mantissa.abs().log10().floor();
		return Decimal {
			mantissa: if (temp_exponent as i32) == NUMBER_EXP_MIN {
				self.mantissa * 10.0 / 1e-323
			} else {
				self.mantissa / power_of_10(temp_exponent as i32)
			},
			exponent: self.exponent + temp_exponent
		};
	}

	fn to_number(&self) -> f64 {
		//  Problem: new(116.0).to_number() returns 115.99999999999999.
		//  TODO: How to fix in general case? It's clear that if to_number() is
		//	VERY close to an integer, we want exactly the integer.
		//	But it's not clear how to specifically write that.
		//	So I'll just settle with 'exponent >= 0 and difference between rounded
		//	and not rounded < 1e-9' as a quick fix.
		//  var result = self.mantissa * 10.0_f64.powf(self.exponent);
		if !f64::is_finite(self.exponent) {
			return NAN;
		}

		if self.exponent > NUMBER_EXP_MAX as f64 {
			return  if self.mantissa > 0.0 {
				INFINITY
			} else {
				NEG_INFINITY
			}
		}

		if self.exponent < NUMBER_EXP_MIN as f64 {
			return 0.0;
		}


		if self.exponent == NUMBER_EXP_MIN as f64 {
			return if self.mantissa > 0.0 {
				5e-324
			} else {
				-5e-324
			};
		}

		let result: f64 = self.mantissa * power_of_10(self.exponent as i32);

		if !f64::is_finite(result) || self.exponent < 0.0 {
			return result;
		}

		let result_rounded = result.round();

		if (result_rounded - result).abs() < ROUND_TOLERANCE {
			return result_rounded;
		}

		return result;
	}
	fn to_string(&self) -> String {
		if f64::is_nan(self.mantissa) || f64::is_nan(self.exponent) {
			return String::from("NaN");
		} else if self.exponent >= EXP_LIMIT {
			return if self.mantissa > 0.0 {
				String::from("Infinity")
			} else {
				String::from("-Infinity")
			}
		} else if self.exponent <= -EXP_LIMIT || self.mantissa == 0.0 {
			return String::from("0");
		} else if self.exponent < 21.0 && self.exponent > -7.0 {
			return self.to_number().to_string();
		}

		return self.mantissa.to_string().to_owned() + "e" + (if self.exponent >= 0.0 {
			"+"
		} else {
			""
		}) + &*self.exponent.to_string();
	}
	fn to_exponential(&self, mut places: i32) -> String {
		if f64::is_nan(self.mantissa) || f64::is_nan(self.exponent) {
			return String::from("NaN");
		} else if self.exponent >= EXP_LIMIT {
			return if self.mantissa > 0.0 {
				String::from("Infinity")
			} else {
				String::from("-Infinity")
			};
		}

		let tmp = pad_end(String::from("."), places + 1, String::from("0"));
		// 1) exponent is < 308 and > -324: use basic to_fixed
		// 2) everything else: we have to do it ourselves!
		if self.exponent <= -EXP_LIMIT || self.mantissa == 0.0 {
			let str = if places > 0 {
				tmp.as_str()
			} else {
				""
			};
			return String::from("0".to_owned() + str + "e+0");
		} else if !f32::is_finite(places as f32) {
			places = MAX_SIGNIFICANT_DIGITS;
		}

		let len = places + 1;
		let num_digits = self.mantissa.abs().log10().max(1.0);
		let rounded = (self.mantissa * 10.0_f64.powi(len - num_digits as i32)).round() * 10.0_f64.powi(num_digits as i32 - len);
		return to_fixed(rounded, 0_i32.max(len - num_digits as i32).to_owned()) +  "e" + if self.exponent >= 0.0 {
			"+"
		} else {
			""
		} + self.exponent.to_string().as_str();
	}
	fn to_fixed(&self, places: i32) -> String {
		if f64::is_nan(self.mantissa) || f64::is_nan(self.exponent) {
			return String::from("NaN");
		} else if self.exponent >= EXP_LIMIT {
			return if self.mantissa > 0.0 {
				String::from("Infinity")
			} else {
				String::from("-Infinity")
			}
		}

		let tmp = pad_end(String::from("."), places + 1, String::from("0"));
		if self.exponent <= -EXP_LIMIT || self.mantissa == 0.0 {
			// Two Cases:
			// 1) exponent is 17 or greater: just print out mantissa with the appropriate number of zeroes after it
			// 2) exponent is 16 or less: use basic toFixed
			let str = if places > 0 {
				tmp.as_str()
			} else {
				""
			};
			return String::from("0".to_owned() + str);
		} else if self.exponent >= MAX_SIGNIFICANT_DIGITS as f64 {
			let str = pad_end(self.mantissa.to_string().replace(".", ""), (self.exponent + 1.0) as i32, String::from("0")).to_owned() +
				if places > 0 {
					tmp.as_str()
				} else {
					""
				};
			return String::from(str);
		}

		return to_fixed(self.to_number(), places);
	}
	fn to_precision(&self, places: i32) -> String {
		if self.exponent <= -7.0 {
			return self.to_exponential(places - 1);
		}

		if (places as f64) > self.exponent {
			return self.to_fixed((places as f64 - self.exponent - 1.0) as i32);
		}

		return self.to_exponential(places - 1);
	}

	fn mantissa_with_decimal_places(&self, places: i32) -> f64 {
		// https://stackoverflow.com/a/37425022
		if f64::is_nan(self.mantissa) || f64::is_nan(self.exponent) {
			return NAN;
		} else if self.mantissa == 0.0 {
			return 0.0
		}

		let len = places + 1;
		let num_digits = self.mantissa.abs().log10().ceil();
		let rounded = (self.mantissa * 10.0_f64.powi(len - num_digits as i32)).round() * 10.0_f64.powi(num_digits as i32 - len);
		return to_fixed_num(rounded, 0_i32.max(len - num_digits as i32));
	}

	fn value_of(&self) -> String {
		return self.to_string();
	}
	fn to_json(&self) -> String {
		return self.to_string();
	}
	fn to_string_with_decimal_places(&self, places: i32) -> String {
		return self.to_exponential(places);
	}

	fn abs(&self) -> Decimal {
		return from_mantissa_exponent_no_normalize(self.mantissa.abs(), self.exponent);
	}

	fn neg(&self) -> Decimal {
		return from_mantissa_exponent_no_normalize(-self.mantissa, self.exponent);
	}
	fn negate(&self) -> Decimal {
		return self.neg();
	}
	fn negated(&self) -> Decimal {
		return self.neg();
	}

	fn sign(&self) -> i32 {
		return if self.mantissa.is_sign_positive() {
			1
		} else if  self.mantissa.is_sign_negative() {
			-1
		} else {
			0
		};
	}
	fn sgn(&self) -> i32 {
		return self.sign();
	}

	fn round(&self) -> Decimal {
		if self.exponent < -1.0 {
			return new(0.0);
		} else if self.exponent < MAX_SIGNIFICANT_DIGITS as f64 {
			return new(self.to_number().round());
		}

		return from_decimal(self);
	}
	fn trunc(&self) -> Decimal {
		if self.exponent < 0.0 {
			return new(0.0);
		} else if self.exponent < MAX_SIGNIFICANT_DIGITS as f64 {
			return new(self.to_number().trunc());
		}

		return from_decimal(self);
	}
	fn floor(&self) -> Decimal {
		if self.exponent < -1.0 {
			return if self.sign() >= 0 {
				new(0.0)
			} else {
				new(-1.0)
			}
		} else if self.exponent < MAX_SIGNIFICANT_DIGITS as f64 {
			return new(self.to_number().floor());
		}

		return from_decimal(self);
	}
	fn ceil(&self) -> Decimal {
		if self.exponent < -1.0 {
			return if self.sign() > 0 {
				new(1.0)
			} else {
				new(0.0)
			};
		} else if self.exponent < MAX_SIGNIFICANT_DIGITS as f64 {
			return new(self.to_number().ceil());
		}

		return from_decimal(self);
	}

	fn add(&self, decimal: &Decimal) -> Decimal {
		// Figure out which is bigger, shrink the mantissa of the smaller
		// by the difference in exponents, add mantissas, normalize and return
		// TODO: Optimizations and simplification may be possible, see https://github.com/Patashu/break_infinity.js/issues/8
		if self.mantissa == 0.0 {
			return from_decimal(decimal);
		} else if decimal.mantissa == 0.0 {
			return from_decimal(self);
		}

		let bigger_decimal;
		let smaller_decimal;

		if self.exponent >= decimal.exponent {
			bigger_decimal = from_decimal(self);
			smaller_decimal = from_decimal(&decimal);
		} else {
			bigger_decimal = from_decimal(&decimal);
			smaller_decimal = from_decimal(self);
		}

		if bigger_decimal.exponent - smaller_decimal.exponent > MAX_SIGNIFICANT_DIGITS as f64 {
			return bigger_decimal;
		}

		return from_mantissa_exponent((1e14 * bigger_decimal.mantissa) + 1e14 * &smaller_decimal.mantissa * power_of_10((&smaller_decimal.exponent - bigger_decimal.exponent) as i32).round(), bigger_decimal.exponent - 14 as f64);
	}
	fn plus(&self, decimal: &Decimal) -> Decimal {
		return self.add(decimal);
	}

	fn sub(&self, decimal: &Decimal) -> Decimal {
		return self.add(&decimal.neg());
	}
	fn subtract(&self, decimal: &Decimal) -> Decimal {
		return self.sub(decimal);
	}
	fn minus(&self, decimal: &Decimal) -> Decimal {
		return self.sub(decimal);
	}

	fn mul(&self, decimal: &Decimal) -> Decimal {
		return from_mantissa_exponent(self.mantissa * decimal.mantissa, self.exponent + decimal.exponent);
	}
	fn multiply(&self, decimal: &Decimal) -> Decimal {
		return self.mul(decimal);
	}
	fn times(&self, decimal: &Decimal) -> Decimal {
		return self.mul(decimal);
	}

	fn div(&self, decimal: &Decimal) -> Decimal {
		return self.mul(&decimal.recip());
	}
	fn divide(&self, decimal: &Decimal) -> Decimal {
		return self.div(decimal);
	}
	fn divide_by(&self, decimal: &Decimal) -> Decimal {
		return self.div(decimal);
	}
	fn divided_by(&self, decimal: &Decimal) -> Decimal {
		return self.div(decimal);
	}

	fn recip(&self) -> Decimal {
		return from_mantissa_exponent(1.0 / self.mantissa, -self.exponent);
	}
	fn reciprocal(&self) -> Decimal {
		return self.recip();
	}
	fn reciprocate(&self) -> Decimal {
		return self.recip();
	}

	/**
	 * Returns -1 for less than value, 0 for equals value, 1 for greater than value
	 */
	fn cmp(&self, decimal: &Decimal) -> i32 {
		/*
		From smallest to largest:
		-3e100
		-1e100
		-3e99
		-1e99
		-3e0
		-1e0
		-3e-99
		-1e-99
		-3e-100
		-1e-100
		0
		1e-100
		3e-100
		1e-99
		3e-99
		1e0
		3e0
		1e99
		3e99
		1e100
		3e100
		*/

		if self.mantissa == 0.0 {
			if decimal.mantissa == 0.0 {
				return 0;
			} else if decimal.mantissa < 0.0 {
				return 1;
			} else if decimal.mantissa > 0.0 {
				return -1;
			}
		}

		if decimal.mantissa == 0.0 {
			if self.mantissa < 0.0 {
				return -1;
			} else if self.mantissa > 0.0 {
				return 1;
			}
		}

		if self.mantissa > 0.0 {
			if decimal.mantissa < 0.0 {
				return 1;
			} else if self.exponent > decimal.exponent {
				return 1;
			} else if self.exponent < decimal.exponent {
				return -1;
			} else if self.mantissa > decimal.mantissa {
				return 1;
			} else if self.mantissa < decimal.mantissa {
				return -1;
			}

			return 0;
		}

		if self.mantissa < 0.0 {
			if decimal.mantissa > 0.0 {
				return -1;
			} else if self.exponent > decimal.exponent {
				return -1;
			} else if self.exponent < decimal.exponent {
				return 1;
			} else if self.mantissa > decimal.mantissa {
				return 1;
			} else if self.mantissa < decimal.mantissa {
				return -1;
			}

			return 0;
		}

		return NAN as i32;
	}
	fn compare(&self, decimal: &Decimal) -> i32 {
		return self.cmp(decimal);
	}

	fn eq(&self, decimal: &Decimal) -> bool {
		return self.exponent == decimal.exponent && self.mantissa == decimal.exponent;
	}
	fn equals(&self, decimal: &Decimal) -> bool {
		return self.eq(decimal);
	}

	fn neq(&self, decimal: &Decimal) -> bool {
		return !self.eq(decimal);
	}
	fn not_equals(&self, decimal: &Decimal) -> bool {
		return !self.neq(decimal);
	}

	fn lt(&self, decimal: &Decimal) -> bool {
		if self.mantissa == 0.0 {
			return decimal.mantissa > 0.0;
		} else if decimal.mantissa == 0.0 {
			return self.mantissa <= 0.0;
		} else if self.exponent == decimal.exponent {
			return self.mantissa < decimal.mantissa;
		} else if self.mantissa > 0.0 {
			return decimal.mantissa > 0.0 && self.exponent < decimal.exponent;
		}

		return decimal.mantissa > 0.0 || self.exponent > decimal.exponent;
	}
	fn lte(&self, decimal: &Decimal) -> bool {
		return !self.gt(decimal);
	}

	fn gt(&self, decimal: &Decimal) -> bool {
		if self.mantissa == 0.0 {
			return decimal.mantissa < 0.0;
		} else if decimal.mantissa == 0.0 {
			return self.mantissa > 0.0;
		} else if self.exponent == decimal.exponent {
			return self.mantissa > decimal.mantissa;
		} else if self.mantissa > 0.0 {
			return decimal.mantissa < 0.0 || self.exponent > decimal.exponent;
		}

		return decimal.mantissa < 0.0 && self.exponent < decimal.exponent;
	}
	fn gte(&self, decimal: &Decimal) -> bool {
		return !self.lt(decimal);
	}

	fn less_than_or_equal_to(&self, other: &Decimal) -> bool {
		return self.cmp(other) < 1;
	}
	fn less_han(&self, other: &Decimal) -> bool{
		return self.cmp(other) < 0;
	}

	fn greater_than_or_equal_to(&self, other: &Decimal) -> bool {
		return self.cmp(other) > -1;
	}
	fn greater_than(&self, other: &Decimal) -> bool {
		return self.cmp(other) > 0;
	}

	fn min(&self, decimal: &Decimal) -> Decimal {
		return if self.gt(decimal) {
			from_decimal(decimal)
		} else {
			from_decimal(self)
		}
	}
	fn max(&self, decimal: &Decimal) -> Decimal{
		return if self.lt(decimal) {
			from_decimal(decimal)
		} else {
			from_decimal(self)
		}
	}

	fn clamp(&self, min: &Decimal, max: &Decimal) -> Decimal {
		return self.max(min).min(max);
	}
	fn clamp_min(&self, min: &Decimal) -> Decimal {
		return self.max(min);
	}
	fn clamp_max(&self, max: &Decimal) -> Decimal {
		return self.min(max);
	}

	fn cmp_tolerance(&self, decimal: &Decimal, tolerance: &Decimal) -> i32 {
		return if self.eq_tolerance(decimal, tolerance) {
			0
		} else {
			self.cmp(decimal)
		}
	}
	fn compare_tolerance(&self, decimal: &Decimal, tolerance: &Decimal) -> i32 {
		return self.cmp_tolerance(decimal, tolerance);
	}

	/**
	 * Tolerance is a relative tolerance, multiplied by the greater of the magnitudes of the two arguments.
	 * For example, if you put in 1e-9, then any number closer to the
	 * larger number than (larger number) * 1e-9 will be considered equal.
	 */
	fn eq_tolerance(&self, decimal: &Decimal, tolerance: &Decimal) -> bool {
		// return abs(a-b) <= tolerance * max(abs(a), abs(b))
		return self.sub(decimal).abs().lte(&self.abs().max(&decimal.abs().mul(tolerance)));
	}
	fn equals_tolerance(&self, decimal: &Decimal, tolerance: &Decimal) -> bool {
		return self.eq_tolerance(decimal, tolerance);
	}

	fn neq_tolerance(&self, decimal: &Decimal, tolerance: &Decimal) -> bool {
		return !self.eq_tolerance(decimal, tolerance);
	}
	fn not_equals_tolerance(&self, decimal: &Decimal, tolerance: &Decimal) -> bool {
		return self.neq_tolerance(decimal, tolerance);
	}

	fn lt_tolerance(&self, decimal: &Decimal, tolerance: &Decimal) -> bool {
		return !self.eq_tolerance(decimal, tolerance) && self.lt(decimal);
	}
	fn lte_tolerance(&self, decimal: &Decimal, tolerance: &Decimal) -> bool {
		return self.eq_tolerance(decimal, tolerance) || self.lt(decimal);
	}

	fn gt_tolerance(&self, decimal: &Decimal, tolerance: &Decimal) -> bool {
		return !self.eq_tolerance(decimal, tolerance) && self.gt(decimal);
	}
	fn gte_tolerance(&self, decimal: &Decimal, tolerance: &Decimal) -> bool {
		return self.eq_tolerance(decimal, tolerance) || self.gt(decimal);
	}

	fn log10(&self) -> f64 {
		return self.exponent + self.mantissa.log10();
	}
	fn abs_log10(&self) -> f64 {
		return self.exponent + self.mantissa.abs().log10();
	}
	fn p_log10(&self) -> f64 {
		return if self.mantissa <= 0.0 || self.exponent < 0.0 {
			0.0
		} else {
			self.log10()
		}
	}

	fn log(&self, base: f64) -> f64 {
		// UN-SAFETY: Most incremental game cases are log(number := 1 or greater, base := 2 or greater).
		// We assume this to be true and thus only need to return a number, not a Decimal,
		return LN_10 / base.ln() * self.log10();
	}
	fn logarithm(&self, base: f64) -> f64 {
		return self.log(base);
	}

	fn log2(&self) -> f64 {
		return 3.32192809488736234787 * self.log10();
	}
	fn ln(&self) -> f64{
		return LN_10 * self.log10();
	}

	fn pow(&self, decimal: &Decimal) -> Decimal {
		//  UN-SAFETY: Accuracy not guaranteed beyond ~9-11 decimal places.
		//  TODO: Decimal.pow(new Decimal(0.5), 0); or Decimal.pow(new Decimal(1), -1);
		//	makes an exponent of -0! Is a negative zero ever a problem?

		let number = decimal.to_number();
		//  TODO: Fast track seems about neutral for performance.
		//	It might become faster if an integer pow is implemented,
		//	or it might not be worth doing (see https://github.com/Patashu/break_infinity.js/issues/4 )
		//  Fast track: If (this.e*value) is an integer and mantissa ^ value
		//  fits in a Number, we can do a very fast method.

		let temp = self.exponent * number;
		let mut new_mantissa;

		if temp < MAX_SAFE_INTEGER {
			// Same speed and usually more accurate.
			new_mantissa = self.mantissa.powf(number);

			if f64::is_finite(new_mantissa) && new_mantissa != 0.0 {
				return from_mantissa_exponent(new_mantissa, temp);
			}
		}

		let new_exponent = temp.trunc();
		let residue = temp - new_exponent;
		new_mantissa = 10.0_f64.powf(number * self.mantissa.log10() + residue);

		if f64::is_finite(new_mantissa) && new_mantissa != 0.0 {
			//  return Decimal.exp(value*this.ln());
			//  UN-SAFETY: This should return NaN when mantissa is negative and value is non-integer.
			return from_mantissa_exponent(new_mantissa, new_exponent);
		}

		let result = new(10.0).pow(&new(number * self.abs_log10()));

		if self.sign() == -1 && number % 2.0 == 1.0 {
			return result.neg();
		}

		return result;
	}
	fn pow_base(&self, decimal: &Decimal) -> Decimal {
		return decimal.pow(self);
	}

	fn factorial(&self) -> Decimal {
		//  Using Stirling's Approximation.
		//  https://en.wikipedia.org/wiki/Stirling%27s_approximation#Versions_suitable_for_calculators
		let n = self.to_number() + 1.0;
		return new(n / E * (n * f64::sinh(1.0 / n) + 1.0 / (810.0 * n.powi(6)))).pow(&new(n))
			.mul(&new(f64::sqrt(2.0 * PI / n)));
	}
	fn exp(&self) -> Decimal {
		// Fast track: if -706 < this < 709, we can use regular exp.
		let number = self.to_number();
		if -706.0 < number && number < 709.0 {
			return new(f64::exp(number))
		}
		return new(E).pow(self);
	}

	fn sqr(&self) -> Decimal {
		return from_mantissa_exponent(self.mantissa.powi(2), self.exponent * 2.0);
	}
	fn sqrt(&self) -> Decimal {
		if self.mantissa < 0.0 {
			return new(NAN);
		} else if self.exponent % 2.0 != 0.0 {
			// Mod of a negative number is negative, so != means '1 or -1'
			return from_mantissa_exponent(f64::sqrt(self.mantissa) * 3.16227766016838, (self.exponent / 2.0).floor());
		}
		return from_mantissa_exponent(f64::sqrt(self.mantissa), (self.exponent / 2.0).floor());
	}
	fn cube(&self) -> Decimal {
		return from_mantissa_exponent(self.mantissa.powi(3), self.exponent * 3.0);
	}
	fn cbrt(&self) -> Decimal {
		let mut sign = 1;
		let mut mantissa = self.mantissa;

		if mantissa < 0.0 {
			sign = -1;
			mantissa = -mantissa;
		}

		let new_mantissa = sign as f64 * mantissa.powf((1 / 3) as f64);
		let remainder = (self.exponent % 3.0) as i32;

		if remainder == 1 || remainder == -1 {
			return from_mantissa_exponent(new_mantissa * 2.1544346900318837, (self.exponent / 3.0).floor());
		}

		if remainder != 0 {
			// remainder != 0 at this point means 'remainder == 2 || remainder == -2'
			return from_mantissa_exponent(new_mantissa * 4.6415888336127789, (self.exponent / 3.0).floor());
		}

		return from_mantissa_exponent(new_mantissa, (self.exponent / 3.0).floor());
	}

	// Some hyperbolic trigonometry functions that happen to be easy
	fn sinh(&self) -> Decimal {
		return self.exp().sub(&self.neg().exp()).div(&new(2.0));
	}
	fn cosh(&self) -> Decimal {
		return self.exp().add(&self.neg().exp()).div(&new(2.0));
	}
	fn tanh(&self) -> Decimal {
		return self.sinh().div(&self.cosh());
	}

	fn asinh(&self) -> f64 {
		return (self.add(&self.sqr().add(&new(1.0)).sqrt())).ln();
	}
	fn acosh(&self) -> f64 {
		return (self.add(&self.sqr().sub(&new(1.0)).sqrt())).ln();
	}
	fn atanh(&self) -> f64 {
		if self.abs().gte(&new(1.0)) {
			return NAN;
		}

		return (self.add(&new(1.0)).div(&new(1.0).sub(self))).ln() / 2.0;
	}

	fn dp(&self) -> i32 {
		if !f64::is_finite(self.mantissa) {
			return NAN as i32;
		} else if self.exponent >= MAX_SIGNIFICANT_DIGITS as f64 {
			return 0;
		}

		let mantissa = self.mantissa;
		let mut places = -self.exponent as i32;
		let mut e = 1.0;

		while (mantissa * e).round().abs() / e - mantissa > ROUND_TOLERANCE {
			e *= 10.0;
			places += 1;
		}

		return if places > 0 {
			places
		} else {
			0
		};
	}
	fn decimal_places(&self) -> i32 {
		return self.dp();
	}

	/**
	 * Joke function from Realm Grinder
	 */
	fn ascension_penalty(&self, ascensions: f64) -> Decimal {
		if ascensions == 0.0 {
			return from_decimal(self);
		}

		return self.pow(&new(10.0_f64.powf(-ascensions)));
	}

	/**
	 * Joke function from Cookie Clicker. It's 'egg'
	 */
	fn egg(&self) -> Decimal {
		return self.add(&new(9.0));
	}
}

pub fn normalize(decimal: &mut Decimal) -> Decimal {
	return decimal.normalize();
}

pub fn abs(decimal: &Decimal) -> Decimal {
	return decimal.abs();
}

pub fn neg(decimal: &Decimal) -> Decimal {
	return decimal.neg();
}
pub fn negate(decimal: &Decimal) -> Decimal {
	return decimal.neg();
}
pub fn negated(decimal: &Decimal) -> Decimal {
	return decimal.neg();
}

pub fn sgn(decimal: &Decimal) -> i32 {
	return decimal.sgn();
}
pub fn sign(decimal: &Decimal) -> i32 {
	return decimal.sign();
}

pub fn round(decimal: &Decimal) -> Decimal {
	return decimal.round();
}
pub fn trunc(decimal: &Decimal) -> Decimal {
	return decimal.trunc();
}
pub fn floor(decimal: &Decimal) -> Decimal {
	return decimal.floor();
}
pub fn ceil(decimal: &Decimal) -> Decimal {
	return decimal.ceil();
}

pub fn add(decimal: &Decimal, other: &Decimal) -> Decimal {
	return decimal.add(other);
}
pub fn plus(decimal: &Decimal, other: &Decimal) -> Decimal {
	return decimal.add(other);
}

pub fn sub(decimal: &Decimal, other: &Decimal) -> Decimal {
	return decimal.sub(other);
}
pub fn subtract(decimal: &Decimal, other: &Decimal) -> Decimal {
	return decimal.sub(other);
}
pub fn minus(decimal: &Decimal, other: &Decimal) -> Decimal {
	return decimal.sub(other);
}

pub fn mul(decimal: &Decimal, other: &Decimal) -> Decimal {
	return decimal.mul(other);
}
pub fn multiply(decimal: &Decimal, other: &Decimal) -> Decimal {
	return decimal.mul(other);
}
pub fn times(decimal: &Decimal, other: &Decimal) -> Decimal {
	return decimal.mul(other);
}

pub fn div(decimal: &Decimal, other: &Decimal) -> Decimal {
	return decimal.div(other);
}
pub fn divide(decimal: &Decimal, other: &Decimal) -> Decimal {
	return decimal.div(other);
}

pub fn recip(decimal: &Decimal) -> Decimal {
	return decimal.recip();
}
pub fn reciprocal(decimal: &Decimal) -> Decimal {
	return decimal.recip();
}
pub fn reciprocate(decimal: &Decimal) -> Decimal {
	return decimal.recip();
}

pub fn cmp(decimal: &Decimal, other: &Decimal) -> i32 {
	return decimal.cmp(other);
}
pub fn compare(decimal: &Decimal, other: &Decimal) -> i32 {
	return decimal.cmp(other);
}

pub fn eq(decimal: &Decimal, other: &Decimal) -> bool {
	return decimal.eq(other);
}
pub fn equals(decimal: &Decimal, other: &Decimal) -> bool {
	return decimal.eq(other);
}

pub fn neq(decimal: &Decimal, other: &Decimal) -> bool {
	return decimal.neq(other);
}
pub fn not_equals(decimal: &Decimal, other: &Decimal) -> bool {
	return decimal.neq(other);
}

pub fn lt(decimal: &Decimal, other: &Decimal) -> bool {
	return decimal.lt(other);
}
pub fn lte(decimal: &Decimal, other: &Decimal) -> bool {
	return decimal.lte(other);
}

pub fn gt(decimal: &Decimal, other: &Decimal) -> bool {
	return decimal.gt(other);
}
pub fn gte(decimal: &Decimal, other: &Decimal) -> bool {
	return decimal.gte(other);
}

pub fn min(decimal: &Decimal, other: &Decimal) -> Decimal {
	return decimal.min(other);
}
pub fn max(decimal: &Decimal, other: &Decimal) -> Decimal {
	return decimal.max(other);
}

pub fn clamp(decimal: &Decimal, min: &Decimal, max: &Decimal) -> Decimal {
	return decimal.clamp(min, max);
}
pub fn clamp_min(decimal: &Decimal, min: &Decimal) -> Decimal {
	return decimal.clamp_min(min);
}
pub fn clamp_max(decimal: &Decimal, max: &Decimal) -> Decimal {
	return decimal.clamp_max(max);
}

pub fn cmp_tolerance(decimal: &Decimal, other: &Decimal, tolerance: &Decimal) -> i32 {
	return decimal.cmp_tolerance(other, tolerance);
}
pub fn compare_tolerance(decimal: &Decimal, other: &Decimal, tolerance: &Decimal) -> i32 {
	return decimal.cmp_tolerance(other, tolerance);
}

pub fn eq_tolerance(decimal: &Decimal, other: &Decimal, tolerance: &Decimal) -> bool {
	return decimal.eq_tolerance(other, tolerance);
}
pub fn equals_tolerance(decimal: &Decimal, other: &Decimal, tolerance: &Decimal) -> bool {
	return decimal.equals_tolerance(other, tolerance);
}

pub fn neq_tolerance(decimal: &Decimal, other: &Decimal, tolerance: &Decimal) -> bool {
	return decimal.neq_tolerance(other, tolerance);
}
pub fn not_equals_tolerance(decimal: &Decimal, other: &Decimal, tolerance: &Decimal) -> bool {
	return decimal.not_equals_tolerance(other, tolerance);
}

pub fn lt_tolerance(decimal: &Decimal, other: &Decimal, tolerance: &Decimal) -> bool {
	return decimal.lt_tolerance(other, tolerance);
}
pub fn lte_tolerance(decimal: &Decimal, other: &Decimal, tolerance: &Decimal) -> bool {
	return decimal.lte_tolerance(other, tolerance);
}

pub fn gt_tolerance(decimal: &Decimal, other: &Decimal, tolerance: &Decimal) -> bool {
	return decimal.gt_tolerance(other, tolerance);
}
pub fn gte_tolerance(decimal: &Decimal, other: &Decimal, tolerance: &Decimal) -> bool {
	return decimal.gte_tolerance(other, tolerance);
}

pub fn log10(decimal: &Decimal) -> f64 {
	return decimal.log10();
}
pub fn abs_log10(decimal: &Decimal) -> f64 {
	return decimal.abs_log10();
}
pub fn p_log10(decimal: &Decimal) -> f64 {
	return decimal.p_log10();
}

pub fn log(decimal: &Decimal, base: f64) -> f64 {
	return decimal.log(base);
}
pub fn logarithm(decimal: &Decimal, base: f64) -> f64 {
	return decimal.log(base);
}

pub fn log2(decimal: &Decimal) -> f64 {
	return decimal.log2();
}
pub fn ln(decimal: &Decimal) -> f64 {
	return decimal.ln();
}

pub fn pow10(exp: f64) -> Decimal {
	return from_mantissa_exponent(10.0_f64.powf(exp % 1.0), exp.trunc());
}
pub fn pow(decimal: &Decimal, other: &Decimal) -> Decimal {
	return decimal.pow(other);
}

pub fn factorial(decimal: &Decimal) -> Decimal {
	return decimal.factorial();
}
pub fn exp(decimal: &Decimal) -> Decimal {
	return decimal.exp();
}

pub fn sqr(decimal: &Decimal) -> Decimal {
	return decimal.sqr();
}
pub fn sqrt(decimal: &Decimal) -> Decimal {
	return decimal.sqrt();
}

pub fn cube(decimal: &Decimal) -> Decimal {
	return decimal.cube();
}
pub fn cbrt(decimal: &Decimal) -> Decimal {
	return decimal.cbrt();
}

pub fn sinh(decimal: &Decimal) -> Decimal {
	return decimal.sinh();
}
pub fn cosh(decimal: &Decimal) -> Decimal {
	return decimal.cosh();
}
pub fn tanh(decimal: &Decimal) -> Decimal {
	return decimal.tanh();
}

pub fn asinh(decimal: &Decimal) -> f64 {
	return decimal.asinh();
}
pub fn acosh(decimal: &Decimal) -> f64 {
	return decimal.acosh();
}
pub fn atanh(decimal: &Decimal) -> f64 {
	return decimal.atanh();
}

pub fn dp(decimal: &Decimal) -> i32 {
	return decimal.dp();
}
pub fn decimal_places(decimal: &Decimal) -> i32 {
	return decimal.dp();
}

/**
 * If you're willing to spend 'resourcesAvailable' and want to buy something
 * with exponentially increasing cost each purchase (start at priceStart,
 * multiply by priceRatio, already own currentOwned), how much of it can you buy?
 * Adapted from Trimps source code.
 */
pub fn afford_geometric_series(resources_available: &Decimal, price_start:&Decimal, price_ratio: &Decimal, current_owned: &Decimal) -> Decimal {
	let actual_start = price_start.mul(&price_ratio.pow(current_owned));
	return new(resources_available.div(&actual_start).mul(&price_ratio.sub(&new(1.0)))
		.add(&new(1.0)).log10() / price_ratio.log10()).floor();
}

/**
 * How much resource would it cost to buy (numItems) items if you already have currentOwned,
 * the initial price is priceStart and it multiplies by priceRatio each purchase?
 */
pub fn sum_geometric_series(num_items: &Decimal, price_start: &Decimal, price_ratio: &Decimal, current_owned: &Decimal) -> Decimal {
	return price_start.mul(&price_ratio.pow(current_owned)).mul(&new(1.0).sub(&price_ratio.pow(num_items)))
		.div(&new(1.0).sub(price_ratio));
}

/**
 * If you're willing to spend 'resourcesAvailable' and want to buy something with additively
 * increasing cost each purchase (start at priceStart, add by priceAdd, already own currentOwned),
 * how much of it can you buy?
 */
pub fn afford_arithmetic_series(resources_available: &Decimal, price_start: &Decimal, price_add: &Decimal, current_owned: &Decimal) -> Decimal {
	// n = (-(a-d/2) + sqrt((a-d/2)^2+2dS))/d
	// where a is actual_start, d is price_add and S is resources_available
	// then floor it and you're done!
	let actual_start = price_start.add(&current_owned.mul(price_add));
	let b = actual_start.sub(&price_add.div(&new(2.0)));
	let b2 = b.pow(&new(2.0));
	return b.neg().add(&b2.add(&price_add.mul(resources_available).mul(&new(2.0))).sqrt())
		.div(price_add).floor();
}

/**
 * How much resource would it cost to buy (numItems) items if you already have currentOwned,
 * the initial price is priceStart and it adds priceAdd each purchase?
 * Adapted from http://www.mathwords.com/a/arithmetic_series.htm
 */
pub fn sum_arithmetic_series(num_items: &Decimal, price_start: &Decimal, price_add: &Decimal, current_owned: &Decimal) -> Decimal {
	let actual_start = price_start.add(&current_owned.mul(price_add)); // (n/2)*(2*a+(n-1)*d)

	return num_items.div(&new(2.0)).mul(&actual_start.mul(&new(2.0))
		.add(&num_items.sub(&new(1.0)).mul(price_add)));
}

/**
 * When comparing two purchases that cost (resource) and increase your resource/sec by (deltaRpS),
 * the lowest efficiency score is the better one to purchase.
 * From Frozen Cookies:
 * http://cookieclicker.wikia.com/wiki/Frozen_Cookies_(JavaScript_Add-on)#Efficiency.3F_What.27s_that.3F
 */
pub fn efficiency_of_purchase(cost: &Decimal, current_rp_s: &Decimal, delta_rp_s: &Decimal) -> Decimal {
	return cost.div(current_rp_s).add(&cost.div(delta_rp_s));
}


#[cfg(test)]
mod tests {

}
