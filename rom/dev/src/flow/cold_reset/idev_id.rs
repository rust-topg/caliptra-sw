/*++

Licensed under the Apache-2.0 license.

File Name:

    init_dev_id.rs

Abstract:

    File contains the implementation of DICE Initial Device Identity (IDEVID)
    layer.

--*/

use super::crypto::*;
use super::dice::*;
use super::x509::*;
use crate::cprintln;
use crate::flow::cold_reset::{KEY_ID_CDI, KEY_ID_FE, KEY_ID_IDEVID_PRIV_KEY, KEY_ID_UDS};
use crate::print::HexBytes;
use crate::rom_env::RomEnv;
use crate::rom_err_def;
use caliptra_drivers::*;
use caliptra_x509::*;

type InitDevIdCsr<'a> = Certificate<'a, { MAX_CSR_SIZE }>;

/// Initialization Vector used by Deobfuscation Engine during Unique Device Secret (UDS) decryption.
const DOE_UDS_IV: Array4x4 = Array4xN::<4, 16>([0xfb10365b, 0xa1179741, 0xfba193a1, 0x0f406d7e]);

/// Initialization Vector used by Deobfuscation Engine during Field Entropy decryption.
const DOE_FE_IV: Array4x4 = Array4xN::<4, 16>([0xfb10365b, 0xa1179741, 0xfba193a1, 0x0f406d7e]);

/// Key used to derive the Composite Device Identity(CDI) for Initial Device Identity (IDEVID)
const IDEVID_CDI_KEY: Array4x12 = Array4xN::<12, 48>([
    0x5bd3c575, 0x2ba359a2, 0x696c97f0, 0x56f594a3, 0x6130c106, 0xedcddddb, 0xd01044f6, 0xf2d302d8,
    0xeeefec92, 0xa0ebfaa0, 0x36bf2d20, 0x0535df6f,
]);

/// Maximum Certificate Signing Request Size
const MAX_CSR_SIZE: usize = 512;

rom_err_def! {
    InitDevId,
    InitDevIdErr
    {
        CsrBuilderInit= 0x1,
        CsrBuilderBuild= 0x2,
        CsrInvalid = 0x3,
        CsrVerify = 0x4,
    }
}

/// Dice Initial Device Identity (IDEVID) Layer
pub enum InitDevIdLayer {}

impl DiceLayer for InitDevIdLayer {
    /// Perform derivations for the DICE layer
    ///
    /// # Arguments
    ///
    /// * `env`   - ROM Environment
    /// * `_input` - DICE layer input
    ///
    /// # Returns
    ///
    /// * `DiceOutput` - DICE layer output
    fn derive(env: &RomEnv, _input: &DiceInput) -> CaliptraResult<DiceOutput> {
        cprintln!("[idev] ++");
        cprintln!("[idev] CDI.KEYID = {}", KEY_ID_CDI as u8);
        cprintln!("[idev] SUBJECT.KEYID = {}", KEY_ID_IDEVID_PRIV_KEY as u8);
        cprintln!("[idev] UDS.KEYID = {}", KEY_ID_UDS as u8);

        // Decrypt the UDS
        Self::decrypt_uds(env, KEY_ID_UDS)?;

        // Decrypt the Filed Entropy
        Self::decrypt_field_entropy(env, KEY_ID_FE)?;

        // Clear Deobfuscation Engine Secrets
        Self::clear_doe_secrets(env)?;

        // Derive the DICE CDI from decrypted UDS
        Self::derive_cdi(env, KEY_ID_UDS, KEY_ID_CDI)?;

        // Derive DICE Key Pair from CDI
        let key_pair = Self::derive_key_pair(env, KEY_ID_CDI, KEY_ID_IDEVID_PRIV_KEY)?;

        // Generate the Subject Serial Number and Subject Key Identifier.
        // This information will be used by next DICE Layer while generating
        // certificates
        let subj_sn = X509::subj_sn(env, &key_pair.pub_key)?;
        let subj_key_id = X509::idev_subj_key_id(env, &key_pair.pub_key)?;

        // Generate the output for next layer
        let output = DiceOutput {
            subj_key_pair: key_pair,
            subj_sn,
            subj_key_id,
        };

        // Generate the Initial DevID Certificate Signing Request (CSR)
        Self::generate_csr(env, &output)?;

        cprintln!("[idev] --");

        // Return the DICE Layer Output
        Ok(output)
    }
}

impl InitDevIdLayer {
    /// Decrypt Unique Device Secret (UDS)
    ///
    /// # Arguments
    ///
    /// * `env` - ROM Environment
    /// * `uds` - Key Vault slot to store the decrypted UDS in
    fn decrypt_uds(env: &RomEnv, uds: KeyId) -> CaliptraResult<()> {
        // Engage the Deobfuscation Engine to decrypt the UDS
        env.doe().map(|d| d.decrypt_uds(&DOE_UDS_IV, uds))?;
        Ok(())
    }

    /// Decrypt Field Entropy (FW)
    ///
    /// # Arguments
    ///
    /// * `env` - ROM Environment
    /// * `slot` - Key Vault slot to store the decrypted UDS in
    fn decrypt_field_entropy(env: &RomEnv, fe: KeyId) -> CaliptraResult<()> {
        // Engage the Deobfuscation Engine to decrypt the UDS
        env.doe().map(|d| d.decrypt_field_entropy(&DOE_FE_IV, fe))?;
        Ok(())
    }

    /// Clear Deobfuscation Engine secrets
    ///
    /// # Arguments
    ///
    /// * `env` - ROM Environment
    fn clear_doe_secrets(env: &RomEnv) -> CaliptraResult<()> {
        env.doe().map(|d| d.clear_secrets())
    }

    /// Derive Composite Device Identity (CDI) from Unique Device Secret (UDS)
    ///
    /// # Arguments
    ///
    /// * `env` - ROM Environment
    /// * `uds` - Key slot holding the UDS
    /// * `cdi` - Key Slot to store the generated CDI
    fn derive_cdi(env: &RomEnv, uds: KeyId, cdi: KeyId) -> CaliptraResult<()> {
        // CDI Key
        let key = Hmac384Key::Array4x12(&IDEVID_CDI_KEY);
        let data = Hmac384Data::Key(KeyReadArgs::new(uds));
        Crypto::hmac384_mac(env, key, data, cdi)?;

        cprintln!("[idev] Erasing UDS.KEYID = {}", uds as u8);
        env.key_vault().map(|k| k.erase_key(uds))?;
        Ok(())
    }

    /// Derive Dice Layer Key Pair
    ///
    /// # Arguments
    ///
    /// * `env`      - ROM Environment
    /// * `cdi`      - Composite Device Identity
    /// * `priv_key` - Key slot to store the private key into
    ///
    /// # Returns
    ///
    /// * `Ecc384KeyPair` - Derive DICE Layer Key Pair
    fn derive_key_pair(env: &RomEnv, cdi: KeyId, priv_key: KeyId) -> CaliptraResult<Ecc384KeyPair> {
        Crypto::ecc384_key_gen(env, cdi, priv_key)
    }

    /// Generate Local Device ID CSR
    ///
    /// # Arguments
    ///
    /// * `env`    - ROM Environment
    /// * `output` - DICE Output
    fn generate_csr(env: &RomEnv, output: &DiceOutput) -> CaliptraResult<()> {
        //
        // Generate the CSR if requested via Manufacturing Service Register
        //
        // A flag is asserted via JTAG interface to enble the generation of CSR
        if !env.mfg_state().map(|m| m.gen_idev_id_csr()) {
            return Ok(());
        }

        cprintln!("[idev] CSR upload requested");

        // Generate the CSR
        Self::make_csr(env, output)
    }

    /// Create Initial Device ID CSR
    ///
    /// # Arguments
    ///
    /// * `env`    - ROM Environment
    /// * `output` - DICE Output
    fn make_csr(env: &RomEnv, output: &DiceOutput) -> CaliptraResult<()> {
        let key_pair = &output.subj_key_pair;

        // CSR `To Be Signed` Parameters
        let params = InitDevIdCsrTbsParams {
            // Unique Endpoint Identifier
            ueid: &X509::ueid(env)?,

            // Subject Name
            subject_sn: &output.subj_sn,

            // Public Key
            public_key: &key_pair.pub_key.to_der(),
        };

        // Generate the `To Be Signed` portion of the CSR
        let tbs = InitDevIdCsrTbs::new(&params);

        cprintln!(
            "[idev] Signing CSR with SUBJECT.KEYID = {}",
            key_pair.priv_key as u8
        );

        // Sign the the `To Be Signed` portion
        let sig = Crypto::ecdsa384_sign(env, key_pair.priv_key, tbs.tbs())?;

        // Verify the signature of the `To Be Signed` portion
        if !Crypto::ecdsa384_verify(env, &key_pair.pub_key, tbs.tbs(), &sig)? {
            raise_err!(CsrVerify);
        }

        // [TODO] Due to printing of the CSR, rom sections are hitting max limits.
        // Add this back when CSR printing is removed from here and added to test cases.
        // let _pub_x: [u8; 48] = key_pair.pub_key.x.into();
        // let _pub_y: [u8; 48] = key_pair.pub_key.y.into();
        // cprint_slice!("[idev] PUB.X", _pub_x);
        // cprint_slice!("[idev] PUB.Y", _pub_y);

        let _sig_r: [u8; 48] = sig.r.into();
        let _sig_s: [u8; 48] = sig.s.into();
        cprintln!("[idev] SIG.R = {}", HexBytes(&_sig_r));
        cprintln!("[idev] SIG.S = {}", HexBytes(&_sig_s));

        // Build the CSR with `To Be Signed` & `Signature`
        let mut csr = [0u8; MAX_CSR_SIZE];
        let csr_bldr =
            Ecdsa384CsrBuilder::new(tbs.tbs(), &sig.to_ecdsa()).ok_or(err_u32!(CsrBuilderInit))?;
        let csr_len = csr_bldr.build(&mut csr).ok_or(err_u32!(CsrBuilderBuild))?;
        cprintln!("[idev] CSR = {}", HexBytes(&csr[..csr_len]));

        // Execute Send CSR Flow
        Self::send_csr(env, InitDevIdCsr::new(&csr, csr_len))
    }

    /// Send Initial Device ID CSR to SOC
    ///
    /// # Argument
    ///
    /// * `env` - ROM Environment
    /// * `csr` - Certificate Signing Request to send to SOC
    fn send_csr(env: &RomEnv, csr: InitDevIdCsr) -> CaliptraResult<()> {
        loop {
            // Create Mailbox send transaction to send the CSR
            if let Some(mut txn) = env.mbox().map(|m| m.try_start_send_txn()) {
                // Copy the CSR to mailbox
                txn.send_request(0, csr.get().ok_or(err_u32!(CsrInvalid))?)?;

                // Signal the JTAG/SOC that Initial Device ID CSR is ready
                env.flow_status().map(|f| f.set_idevid_csr_ready());

                // Wait for JTAG/SOC to consume the mailbox
                while env.mfg_state().map(|m| m.gen_idev_id_csr()) {}

                // Release access to the mailbox
                txn.complete()?;

                cprintln!("[idev] CSR uploaded");

                // exit the loop
                break Ok(());
            }
        }
    }
}
