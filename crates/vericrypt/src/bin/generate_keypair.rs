use pasetors::keys::{Generate, AsymmetricKeyPair};

fn main() {
    let kp = AsymmetricKeyPair::<pasetors::version4::V4>::generate().expect("Failed to generate keypair");
    
    let secret_bytes = kp.secret.as_bytes();
    let public_bytes = kp.public.as_bytes();
    
    println!("PASETO_PRIVATE_KEY={}", hex::encode(secret_bytes));
    println!("PASETO_PUBLIC_KEY={}", hex::encode(public_bytes));
}