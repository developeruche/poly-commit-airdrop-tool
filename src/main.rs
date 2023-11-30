use std::collections::HashMap;
use csv;
use ethers::types::{Address, U256};
use ethers::abi::{encode, Token};
use ethers::utils::keccak256;
use std::str::FromStr;
use ark_poly::{
    univariate::DensePolynomial, EvaluationDomain, Evaluations, Polynomial,
    Radix2EvaluationDomain as D,
};
use ark_bls12_381::{Bls12_381, Fr};
use ark_ec::pairing::Pairing;
use ark_poly_commit::kzg10::{KZG10, Powers, Proof, VerifierKey};
use ark_std::test_rng;
use serde::{Deserialize, Serialize};
use poly_commit_airdrop_tool::write_to_file;
use std::borrow::BorrowMut;



type UniPoly_381 = DensePolynomial<<Bls12_381 as Pairing>::ScalarField>;



#[derive(Serialize, Deserialize, Default)]
struct OutPut {
    commitment: String,
    verification_key: VerificationOutput,
    proofs: HashMap<String, OutputProof>
}

#[derive(Serialize, Deserialize, Default)]
struct VerificationOutput {
    g: String,
    gamma_g: String,
    h: String,
    beta_h: String,
}


#[derive(Serialize, Deserialize, Default)]
struct OutputProof {
    index: u32,
    address: String,
    amount: String,
    proof: String
}



impl OutputProof {
    fn new(index: u32, address: String, amount: String) -> Self {
        Self {
            index,
            address,
            amount,
            ..Default::default()
        }
    }
}





fn main() {
    let mut rdr = csv::Reader::from_path("./src/assets/data.csv").unwrap();
    let mut protocol_outputs =  OutPut::default();


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
        let temp_output = OutputProof::new(x_now.as_u32(), addr.to_string(), amount.to_string());
        protocol_outputs.proofs.insert(addr.to_string(), temp_output);
        
        x_now += U256::one();
    }


    let poly = Evaluations::from_vec_and_domain(y_fs.clone(), D::new(y_fs.len()).unwrap()).interpolate();



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




    // committing to the polynomial
    let (comm, r) = KZG10::<Bls12_381, UniPoly_381>::commit(&powers, &poly, None, None).expect("Commitment failed");


    // =============================================
    // Generating proofs for all evaluations
    // =============================================
    protocol_outputs.commitment = comm.0.to_string();



    let ty: Vec<_>  = protocol_outputs.proofs.iter_mut().map(|mut proof| {
        if proof.1.index < 10 {
            let ppp = KZG10::<Bls12_381, UniPoly_381>::open(&powers, &poly, Fr::from(proof.1.index), &r).unwrap();
            println!("proof: {:?}", ppp.w.to_string());
            proof.1.borrow_mut().proof = ppp.w.to_string();
        } else {
            // proof generation is time consuming so we are only generating proofs for the first 10 evaluations
        }
    }).collect();
    



    // ===============================================
    // saving the output for this protocol to a file
    // ===============================================
    let o_vk = VerificationOutput {
        g: params.powers_of_g[0].to_string(),
        gamma_g: params.powers_of_gamma_g[&0].to_string(),
        h: params.h.to_string(),
        beta_h: params.beta_h.to_string(),
    };
    protocol_outputs.verification_key = o_vk;
    write_to_file("./src/assets/main.json", serde_json::to_string(&protocol_outputs).unwrap().as_str());





































    // preforming a simple test (THIS IS SIMILAR TO WHAT WOULD BE GOING ON THE VERIFIER SMART CONTRACT)
    let value = poly.evaluate(&Fr::from(1));
    let proof = KZG10::<Bls12_381, UniPoly_381>::open(&powers, &poly, Fr::from(1), &r).unwrap();

    let check = KZG10::<Bls12_381, UniPoly_381>::check(
        &vk,
        &comm,
        Fr::from(1),
        value,
        &proof,
    ).unwrap();

    println!("check: {:?}", check);
}
