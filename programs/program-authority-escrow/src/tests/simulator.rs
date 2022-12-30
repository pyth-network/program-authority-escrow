use {
    anchor_lang::{
        prelude::{
            Pubkey,
            Rent,
            UpgradeableLoaderState,
        },
        AccountDeserialize,
        InstructionData,
        ProgramData,
        ToAccountMetas,
    },
    solana_program_test::{
        read_file,
        BanksClient,
        BanksClientError,
        ProgramTest,
    },
    solana_sdk::{
        account::Account,
        bpf_loader_upgradeable,
        hash::Hash,
        instruction::Instruction,
        signature::{
            Keypair,
            Signer,
        },
        stake_history::Epoch,
        transaction::Transaction,
    },
    std::path::PathBuf,
};


pub struct EscrowSimulator {
    banks_client:       BanksClient,
    recent_blockhash:   Hash,
    genesis_keypair:    Keypair,
    helloworld_address: Pubkey,
    escrow_address:     Pubkey,
}

impl EscrowSimulator {
    /// Deploys the executor program as upgradable
    pub async fn new() -> (EscrowSimulator, Keypair) {
        let mut bpf_data = read_file(PathBuf::from("../../tests/fixtures/helloworld.so"));

        let escrow_address = crate::id();

        let mut program_test = ProgramTest::new("program_authority_escrow", escrow_address, None);
        let upgrade_authority = Keypair::new();

        let helloworld_address = add_program_as_upgradable(
            &mut bpf_data,
            &upgrade_authority.pubkey(),
            &mut program_test,
        );

        let (banks_client, genesis_keypair, recent_blockhash) = program_test.start().await;

        (
            EscrowSimulator {
                banks_client,
                recent_blockhash,
                genesis_keypair,
                helloworld_address,
                escrow_address,
            },
            upgrade_authority,
        )
    }
}

pub fn add_program_as_upgradable(
    data: &mut Vec<u8>,
    upgrade_authority: &Pubkey,
    program_test: &mut ProgramTest,
) -> Pubkey {
    let program_key = Pubkey::new_unique();
    let (programdata_key, _) =
        Pubkey::find_program_address(&[&program_key.to_bytes()], &bpf_loader_upgradeable::id());


    let program_deserialized = UpgradeableLoaderState::Program {
        programdata_address: programdata_key,
    };
    let programdata_deserialized = UpgradeableLoaderState::ProgramData {
        slot:                      1,
        upgrade_authority_address: Some(*upgrade_authority),
    };

    // Program contains a pointer to progradata
    let program_vec = bincode::serialize(&program_deserialized).unwrap();
    // Programdata contains a header and the binary of the program
    let mut programdata_vec = bincode::serialize(&programdata_deserialized).unwrap();
    programdata_vec.append(data);

    let program_account = Account {
        lamports:   Rent::default().minimum_balance(program_vec.len()),
        data:       program_vec,
        owner:      bpf_loader_upgradeable::ID,
        executable: true,
        rent_epoch: Epoch::default(),
    };
    let programdata_account = Account {
        lamports:   Rent::default().minimum_balance(programdata_vec.len()),
        data:       programdata_vec,
        owner:      bpf_loader_upgradeable::ID,
        executable: false,
        rent_epoch: Epoch::default(),
    };

    // Add both accounts to program test, now the program is deployed as upgradable
    program_test.add_account(program_key, program_account);
    program_test.add_account(programdata_key, programdata_account);

    program_key
}


impl EscrowSimulator {
    async fn process_ix(
        &mut self,
        instruction: Instruction,
        signers: &Vec<&Keypair>,
    ) -> Result<(), BanksClientError> {
        let mut transaction =
            Transaction::new_with_payer(&[instruction], Some(&self.genesis_keypair.pubkey()));

        let blockhash = self.banks_client.get_latest_blockhash().await.unwrap();
        self.recent_blockhash = blockhash;

        transaction.partial_sign(&[&self.genesis_keypair], self.recent_blockhash);
        transaction.partial_sign(signers, self.recent_blockhash);
        self.banks_client.process_transaction(transaction).await
    }

    pub async fn propose(
        &mut self,
        current_authority_keypair: &Keypair,
        new_authority: &Pubkey,
    ) -> Result<(), BanksClientError> {
        let account_metas = crate::accounts::Propose::create(
            &current_authority_keypair.pubkey(),
            new_authority,
            &self.helloworld_address,
            &self.escrow_address,
        )
        .to_account_metas(None);

        let instruction = Instruction {
            program_id: self.escrow_address,
            accounts:   account_metas,
            data:       crate::instruction::Propose.data(),
        };

        self.process_ix(instruction, &vec![current_authority_keypair])
            .await
    }

    pub async fn revert(
        &mut self,
        current_authority_keypair: &Keypair,
        new_authority: &Pubkey,
    ) -> Result<(), BanksClientError> {
        let account_metas = crate::accounts::Propose::create(
            &current_authority_keypair.pubkey(),
            new_authority,
            &self.helloworld_address,
            &self.escrow_address,
        )
        .to_account_metas(None);

        let instruction = Instruction {
            program_id: self.escrow_address,
            accounts:   account_metas,
            data:       crate::instruction::Revert.data(),
        };

        self.process_ix(instruction, &vec![current_authority_keypair])
            .await
    }

    pub async fn accept(
        &mut self,
        current_authority: &Pubkey,
        new_authority_keypair: &Keypair,
    ) -> Result<(), BanksClientError> {
        let account_metas = crate::accounts::Accept::create(
            current_authority,
            &new_authority_keypair.pubkey(),
            &self.helloworld_address,
            &self.escrow_address,
        )
        .to_account_metas(None);

        let instruction = Instruction {
            program_id: self.escrow_address,
            accounts:   account_metas,
            data:       crate::instruction::Accept.data(),
        };

        self.process_ix(instruction, &vec![new_authority_keypair])
            .await
    }

    pub async fn get_program_data(&mut self) -> ProgramData {
        let program_data = Pubkey::find_program_address(
            &[self.helloworld_address.as_ref()],
            &bpf_loader_upgradeable::id(),
        )
        .0;

        let account = self
            .banks_client
            .get_account(program_data)
            .await
            .unwrap()
            .unwrap();
        return ProgramData::try_deserialize(&mut account.data.as_slice()).unwrap();
    }

    pub fn get_escrow_authority(
        &self,
        current_authority: &Pubkey,
        new_authority: &Pubkey,
    ) -> Pubkey {
        Pubkey::find_program_address(
            &[current_authority.as_ref(), new_authority.as_ref()],
            &self.escrow_address,
        )
        .0
    }
}

impl crate::accounts::Propose {
    pub fn create(
        current_authority: &Pubkey,
        new_authority: &Pubkey,
        program_account: &Pubkey,
        escrow_address: &Pubkey,
    ) -> Self {
        let escrow_authority = Pubkey::find_program_address(
            &[current_authority.as_ref(), new_authority.as_ref()],
            escrow_address,
        )
        .0;
        let program_data = Pubkey::find_program_address(
            &[program_account.as_ref()],
            &bpf_loader_upgradeable::id(),
        )
        .0;
        crate::accounts::Propose {
            current_authority: *current_authority,
            new_authority: *new_authority,
            escrow_authority,
            program_account: *program_account,
            program_data,
            bpf_upgradable_loader: bpf_loader_upgradeable::id(),
        }
    }
}

impl crate::accounts::Accept {
    pub fn create(
        current_authority: &Pubkey,
        new_authority: &Pubkey,
        program_account: &Pubkey,
        escrow_address: &Pubkey,
    ) -> Self {
        let escrow_authority = Pubkey::find_program_address(
            &[current_authority.as_ref(), new_authority.as_ref()],
            escrow_address,
        )
        .0;
        let program_data = Pubkey::find_program_address(
            &[program_account.as_ref()],
            &bpf_loader_upgradeable::id(),
        )
        .0;
        crate::accounts::Accept {
            current_authority: *current_authority,
            new_authority: *new_authority,
            escrow_authority,
            program_account: *program_account,
            program_data,
            bpf_upgradable_loader: bpf_loader_upgradeable::id(),
        }
    }
}
