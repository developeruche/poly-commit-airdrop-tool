use csv;
use ethers::types::{Address, U256};
use ethers::abi::{encode, Token};
use ethers::utils::keccak256;
use std::str::FromStr;
use ark_ff::{FftField, Field, ToConstraintField, Zero};
use ark_poly::{
    univariate::DensePolynomial, EvaluationDomain, Evaluations, Polynomial,
    Radix2EvaluationDomain as D,
};
// use ark_poly_commit::Evaluations;
use ark_bls12_381::{Bls12_381, Fr, };






fn main() {
    let mut rdr = csv::Reader::from_path("./src/data.csv").unwrap();

    let mut ys:  Vec<U256> = Vec::new();
    let mut y_fs : Vec<Fr> = Vec::new();
    let mut xs:  Vec<U256> = Vec::new();
    let mut x_now: U256 = U256::zero();


    for result in rdr.records() {
        let record = result.unwrap();

        let addr = record.get(0).unwrap().trim();
        let amount = record.get(1).unwrap().trim();

        let encoded_data = encode(&[
            Token::Address(Address::from_str(addr).unwrap()),
            Token::Uint(U256::from_dec_str(amount).unwrap())]);

        let hash = keccak256(&encoded_data);

        let y = U256::from_big_endian(&hash.clone());
        let x = x_now.clone();
        let y_f = Fr::from_str(y.clone().to_string().as_str()).unwrap();

        println!("x {:?} y {:?} y_f {:?}", x, y, y_f.to_string());
        ys.push(y);
        xs.push(x);
        y_fs.push(y_f);

        x_now += U256::one();

        break;
    }


    let poly = Evaluations::from_vec_and_domain(y_fs.clone(), D::new(y_fs.len()).unwrap()).interpolate();

    let ee = poly.evaluate(&Fr::from_str("1").unwrap());
    println!("poly {:?}", ee.to_string());
}
