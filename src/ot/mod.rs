use blake3::Hasher;
use curve25519_dalek::constants::RISTRETTO_BASEPOINT_TABLE;
use curve25519_dalek::ristretto::RistrettoPoint;
use curve25519_dalek::scalar::Scalar;

// Chou-Orlandi Oblivious Transfer

pub struct Sender {
    y: Scalar,
    pub s: RistrettoPoint,
    t: RistrettoPoint,
}
pub struct Receiver {
    choice: Option<u8>,
    x: Scalar,
    r: Option<RistrettoPoint>,
    s: Option<RistrettoPoint>,
}

fn hash(contents: Vec<RistrettoPoint>) -> [u8; 32] {
    let mut hasher = Hasher::new();
    for content in contents {
        hasher.update(content.compress().as_bytes());
    }
    let digest = hasher.finalize();
    // return digest.as_bytes().to_vec();
    return digest.into();
}

impl Sender {
    pub fn new() -> Self {
        let y = Scalar::random(&mut rand::thread_rng());
        let s = &y * RISTRETTO_BASEPOINT_TABLE;
        let t = &y * s;
        Self { y, s, t }
    }

    pub fn derive_keys(&self, blinded_choice: RistrettoPoint) -> ([u8; 32], [u8; 32]) {
        let p1 = (self.y * blinded_choice) - Scalar::from(0 as u32) * self.t;
        let p2 = (self.y * blinded_choice) - Scalar::from(1 as u32) * self.t;
        let digest1 = hash(vec![self.s, blinded_choice, p1]);
        let digest2 = hash(vec![self.s, blinded_choice, p2]);

        return (digest1, digest2);
    }

    pub fn encrypt(
        &self,
        key1: Vec<u8>,
        key2: Vec<u8>,
        m1: &[u8],
        m2: &[u8],
    ) -> (Vec<u8>, Vec<u8>) {
        if m1.len() != key1.len() || m2.len() != key2.len() {
            panic!("Message lengths must match key lengths");
        }
        let mut ciphertext = vec![0u8; m1.len() + 32];
        ciphertext[..m1.len()].copy_from_slice(&m1);
        for i in 0..m1.len() {
            ciphertext[i] ^= key1[i];
        }
        ciphertext[m1.len()..].copy_from_slice(&key1[..32]);

        let mut ciphertext2 = vec![0u8; m2.len() + 32];
        ciphertext2[..m2.len()].copy_from_slice(&m2);
        for i in 0..m2.len() {
            ciphertext2[i] ^= key2[i];
        }
        ciphertext2[m2.len()..].copy_from_slice(&key2[..32]);

        return (ciphertext, ciphertext2);
    }
}

impl Receiver {
    pub fn new() -> Self {
        let x = Scalar::random(&mut rand::thread_rng());
        Self {
            x,
            r: None,
            choice: None,
            s: None,
        }
    }

    pub fn choose(&mut self, s: &RistrettoPoint, choice: u8) -> RistrettoPoint {
        let scalar_choice = Scalar::from(choice as u64);
        let r = &self.x * RISTRETTO_BASEPOINT_TABLE + &scalar_choice * s;
        self.r = Some(r);
        self.choice = Some(choice);
        self.s = Some(s.clone());
        return r;
    }

    pub fn derive_key(&mut self) -> [u8; 32] {
        if let Some(s) = self.s {
            if let Some(r) = self.r {
                let xs = self.x * s;
                let digest = hash(vec![s, r, xs]);
                digest
            } else {
                panic!("Receiver has not chosen a value yet");
            }
        } else {
            panic!("Receiver has not chosen a value yet");
        }
    }

    fn decrypt(&self, key: &[u8], ciphertext: &[u8]) -> Option<Vec<u8>> {
        if ciphertext.len() < 32 || key[..32] != ciphertext[ciphertext.len() - 32..] {
            panic!("Invalid ciphertext {} {}", ciphertext.len(), key.len());
        } else {
            let mut message = ciphertext[..ciphertext.len() - 32].to_vec();
            for i in 0..message.len() {
                message[i] ^= key[i];
            }
            Some(message)
        }
    }

    pub fn decrypt_ciphertexts(
        &self,
        key: [u8; 32],
        ciphertexts: (Vec<u8>, Vec<u8>),
    ) -> Option<Vec<u8>> {
        if self.choice == Some(0) {
            return self.decrypt(&key, &ciphertexts.0);
        } else if self.choice == Some(1) {
            return self.decrypt(&key, &ciphertexts.1);
        } else {
            panic!("Receiver has not chosen a value yet");
        }
    }
}
