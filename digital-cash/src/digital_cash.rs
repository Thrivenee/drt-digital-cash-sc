#![no_std]
#![allow(unused_attributes)]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

mod deposit_info;

use deposit_info::{DepositInfo, Fee};

pub const SECONDS_PER_ROUND: u64 = 6;
pub use multiversx_sc::api::{ED25519_KEY_BYTE_LEN, ED25519_SIGNATURE_BYTE_LEN};

static NON_EXISTENT_KEY_ERR_MSG: &[u8] = b"non-existent key";
static FEES_NOT_COVERED_ERR_MSG: &[u8] = b"fees not covered";
static CANNOT_DEPOSIT_FUNDS_ERR_MSG: &[u8] =
    b"cannot deposit funds without covering the fee cost first";

#[multiversx_sc::contract]
pub trait DigitalCash {
    #[init]
    fn init(&self, fee: BigUint) {
        self.fee().set(fee);
    }

    // endpoints

    #[endpoint]
    #[payable("*")]
    fn fund(&self, address: ManagedAddress, valability: u64) {
        let deposit_mapper = self.deposit(&address);
        require!(!deposit_mapper.is_empty(), FEES_NOT_COVERED_ERR_MSG);
        let depositor = deposit_mapper.get().depositor_address;
        require!(
            self.blockchain().get_caller() == depositor,
            "invalid depositor"
        );

        let rewa_payment = self.call_value().rewa_value().clone_value();
        let dcdt_payment = self.call_value().all_dcdt_transfers().clone_value();
        let num_tokens = self.get_num_token_transfers(&rewa_payment, &dcdt_payment);
        require!(num_tokens > 0, "amount must be greater than 0");

        let fee = self.fee().get();
        deposit_mapper.update(|deposit| {
            require!(
                deposit.rewa_funds == 0 && deposit.dcdt_funds.is_empty(),
                "key already used"
            );
            require!(
                fee * num_tokens as u64 <= deposit.fees.value,
                CANNOT_DEPOSIT_FUNDS_ERR_MSG
            );

            deposit.fees.num_token_to_transfer += num_tokens;
            deposit.valability = valability;
            deposit.expiration_round = self.get_expiration_round(valability);
            deposit.dcdt_funds = dcdt_payment;
            deposit.rewa_funds = rewa_payment;
        });
    }

    #[endpoint]
    fn withdraw(&self, address: ManagedAddress) {
        let deposit_mapper = self.deposit(&address);
        require!(!deposit_mapper.is_empty(), NON_EXISTENT_KEY_ERR_MSG);

        let block_round = self.blockchain().get_block_round();
        let deposit = deposit_mapper.take();
        require!(
            deposit.expiration_round < block_round,
            "withdrawal has not been available yet"
        );

        let rewa_funds = deposit.rewa_funds + deposit.fees.value;
        if rewa_funds > 0 {
            self.send()
                .direct_rewa(&deposit.depositor_address, &rewa_funds);
        }

        if !deposit.dcdt_funds.is_empty() {
            self.send()
                .direct_multi(&deposit.depositor_address, &deposit.dcdt_funds);
        }
    }

    #[endpoint]
    fn claim(
        &self,
        address: ManagedAddress,
        signature: ManagedByteArray<Self::Api, ED25519_SIGNATURE_BYTE_LEN>,
    ) {
        let deposit_mapper = self.deposit(&address);
        require!(!deposit_mapper.is_empty(), NON_EXISTENT_KEY_ERR_MSG);

        let caller_address = self.blockchain().get_caller();
        self.require_signature(&address, &caller_address, signature);

        let block_round = self.blockchain().get_block_round();
        let fee = self.fee().get();
        let mut deposit = deposit_mapper.take();
        require!(deposit.expiration_round >= block_round, "deposit expired");

        let num_tokens_transfered = deposit.get_num_tokens();
        let fee_cost = fee * num_tokens_transfered as u64;
        deposit.fees.value -= &fee_cost;

        self.collected_fees()
            .update(|collected_fees| *collected_fees += fee_cost);

        if deposit.rewa_funds > 0 {
            self.send()
                .direct_rewa(&caller_address, &deposit.rewa_funds);
        }
        if !deposit.dcdt_funds.is_empty() {
            self.send()
                .direct_multi(&caller_address, &deposit.dcdt_funds);
        }
        if deposit.fees.value > 0 {
            self.send()
                .direct_rewa(&deposit.depositor_address, &deposit.fees.value);
        }
    }

    #[endpoint(claimFees)]
    #[only_owner]
    fn claim_fees(&self) {
        let fees = self.collected_fees().take();
        if fees == 0 {
            return;
        }

        let caller_address = self.blockchain().get_caller();
        self.send().direct_rewa(&caller_address, &fees);
    }

    #[endpoint(depositFees)]
    #[payable("EGLD")]
    fn deposit_fees(&self, address: ManagedAddress) {
        let payment = self.call_value().rewa_value().clone_value();
        let caller_address = self.blockchain().get_caller();
        let deposit_mapper = self.deposit(&address);
        if !deposit_mapper.is_empty() {
            deposit_mapper.update(|deposit| deposit.fees.value += payment);

            return;
        }

        let new_deposit = DepositInfo {
            depositor_address: caller_address,
            dcdt_funds: ManagedVec::new(),
            rewa_funds: BigUint::zero(),
            valability: 0,
            expiration_round: 0,
            fees: Fee {
                num_token_to_transfer: 0,
                value: payment,
            },
        };
        deposit_mapper.set(new_deposit);
    }

    #[endpoint]
    fn forward(
        &self,
        address: ManagedAddress,
        forward_address: ManagedAddress,
        signature: ManagedByteArray<Self::Api, ED25519_SIGNATURE_BYTE_LEN>,
    ) {
        let deposit_mapper = self.deposit(&forward_address);
        require!(!deposit_mapper.is_empty(), CANNOT_DEPOSIT_FUNDS_ERR_MSG);

        let caller_address = self.blockchain().get_caller();
        let fee = self.fee().get();
        self.require_signature(&address, &caller_address, signature);

        let mut forwarded_deposit = self.deposit(&address).take();
        let num_tokens = forwarded_deposit.get_num_tokens();
        deposit_mapper.update(|deposit| {
            require!(
                deposit.rewa_funds == BigUint::zero() && deposit.dcdt_funds.is_empty(),
                "key already used"
            );
            require!(
                &fee * num_tokens as u64 <= deposit.fees.value,
                "cannot forward funds without the owner covering the fee cost first"
            );

            deposit.fees.num_token_to_transfer += num_tokens;
            deposit.valability = forwarded_deposit.valability;
            deposit.expiration_round = self.get_expiration_round(forwarded_deposit.valability);
            deposit.dcdt_funds = forwarded_deposit.dcdt_funds;
            deposit.rewa_funds = forwarded_deposit.rewa_funds;
        });

        let forward_fee = &fee * num_tokens as u64;
        forwarded_deposit.fees.value -= &forward_fee;

        self.collected_fees()
            .update(|collected_fees| *collected_fees += forward_fee);

        if forwarded_deposit.fees.value > 0 {
            self.send().direct_rewa(
                &forwarded_deposit.depositor_address,
                &forwarded_deposit.fees.value,
            );
        }
    }

    // views

    #[view(getAmount)]
    fn get_amount(
        &self,
        address: ManagedAddress,
        token: RewaOrDcdtTokenIdentifier,
        nonce: u64,
    ) -> BigUint {
        let deposit_mapper = self.deposit(&address);
        require!(!deposit_mapper.is_empty(), NON_EXISTENT_KEY_ERR_MSG);

        let deposit = deposit_mapper.get();
        if token.is_rewa() {
            return deposit.rewa_funds;
        }

        for dcdt in deposit.dcdt_funds.into_iter() {
            if dcdt.token_identifier == token && dcdt.token_nonce == nonce {
                return dcdt.amount;
            }
        }

        BigUint::zero()
    }

    // private functions

    fn get_expiration_round(&self, valability: u64) -> u64 {
        let valability_rounds = valability / SECONDS_PER_ROUND;
        self.blockchain().get_block_round() + valability_rounds
    }

    fn get_num_token_transfers(
        &self,
        rewa_value: &BigUint,
        dcdt_transfers: &ManagedVec<DcdtTokenPayment>,
    ) -> usize {
        let mut amount = dcdt_transfers.len();
        if rewa_value > &0 {
            amount += 1;
        }

        amount
    }

    fn require_signature(
        &self,
        address: &ManagedAddress,
        caller_address: &ManagedAddress,
        signature: ManagedByteArray<Self::Api, ED25519_SIGNATURE_BYTE_LEN>,
    ) {
        let addr = address.as_managed_buffer();
        let message = caller_address.as_managed_buffer();
        self.crypto()
            .verify_ed25519(addr, message, signature.as_managed_buffer());
    }

    // storage

    #[view]
    #[storage_mapper("deposit")]
    fn deposit(&self, donor: &ManagedAddress) -> SingleValueMapper<DepositInfo<Self::Api>>;

    #[storage_mapper("fee")]
    fn fee(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("collectedFees")]
    fn collected_fees(&self) -> SingleValueMapper<BigUint>;
}
