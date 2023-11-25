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
use ark_bls12_381::{Bls12_381, Fr};
use ark_ec::pairing::Pairing;
use ark_poly_commit::kzg10::{KZG10, Powers, Proof, VerifierKey};
use ark_std::test_rng;

type UniPoly_381 = DensePolynomial<<Bls12_381 as Pairing>::ScalarField>;


fn main() {
    // reading data from CSV file
    let mut rdr = csv::Reader::from_path("./src/data.csv").unwrap();


    let mut y_fs : Vec<Fr> = Vec::new();
    let mut xs:  Vec<U256> = Vec::new();
    let mut x_now: U256 = U256::one();


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


        xs.push(x);
        y_fs.push(y_f);

        x_now += U256::one();

    }


    let poly = Evaluations::from_vec_and_domain(y_fs.clone(), D::new(y_fs.len()).unwrap()).interpolate();

    let e1 = poly.evaluate(&Fr::from(1));
    println!("e1: {:?}", e1.to_string());




    let rng = &mut test_rng();

    let params =  KZG10::<Bls12_381, UniPoly_381>::setup(poly.degree(), false, rng).unwrap();
    let powers_of_g = params.powers_of_g.to_vec();

    let powers_of_gamma_g = (0..=poly.degree())
        .map(|i| params.powers_of_gamma_g[&i])
        .collect();

    let powers = Powers {
        powers_of_g: ark_std::borrow::Cow::Owned(powers_of_g),
        powers_of_gamma_g: ark_std::borrow::Cow::Owned(powers_of_gamma_g),
    };

    let vk = VerifierKey {
        g: params.powers_of_g[0],
        gamma_g: params.powers_of_gamma_g[&0],
        h: params.h,
        beta_h: params.beta_h,
        prepared_h: params.prepared_h.clone(),
        prepared_beta_h: params.prepared_beta_h.clone(),
    };


    let (comm, r) = KZG10::<Bls12_381, UniPoly_381>::commit(&powers, &poly, None, None).expect("Commitment failed");


    let proof = KZG10::<Bls12_381, UniPoly_381>::open(&powers, &poly, Fr::from(1), &r).unwrap();

    println!("proof: {:?}", proof);
    let value = poly.evaluate(&Fr::from(1));

    let check = KZG10::<Bls12_381, UniPoly_381>::check(
        &vk,
        &comm,
        Fr::from(1),
        value,
        &proof,
    ).unwrap();

    println!("check: {:?}", check);
}
