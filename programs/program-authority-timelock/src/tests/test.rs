use {
    crate::{
        tests::simulator::TimelockSimulator,
        ErrorCode,
    },
    anchor_lang::prelude::ProgramError,
    solana_sdk::{
        instruction::InstructionError,
        signature::Keypair,
        signer::Signer,
        transaction::TransactionError,
    },
};

impl From<ErrorCode> for TransactionError {
    fn from(val: ErrorCode) -> Self {
        TransactionError::InstructionError(
            0,
            InstructionError::try_from(u64::from(ProgramError::from(
                anchor_lang::prelude::Error::from(val),
            )))
            .unwrap(),
        )
    }
}

#[tokio::test]
async fn test() {
    let (mut simulator, authority_keypair_1) = TimelockSimulator::new().await;
    let authority_keypair_2 = Keypair::new();

    simulator
        .check_program_authority_matches(&authority_keypair_1.pubkey())
        .await;

    simulator
        .commit(&authority_keypair_1, &authority_keypair_2.pubkey(), 0)
        .await
        .unwrap();
    simulator
        .check_program_authority_matches(
            &simulator.get_escrow_authority(&authority_keypair_2.pubkey(), 0),
        )
        .await;

    simulator
        .transfer(&authority_keypair_2.pubkey(), 0)
        .await
        .unwrap();
    simulator
        .check_program_authority_matches(&authority_keypair_2.pubkey())
        .await;

    simulator.warp_to_timestamp(1700000000).await.unwrap();

    assert_eq!(
        simulator
            .commit(
                &authority_keypair_2,
                &authority_keypair_1.pubkey(),
                1700000000 + 365 * 24 * 60 * 60 * 2
            )
            .await
            .unwrap_err()
            .unwrap(),
        ErrorCode::TimestampTooLate.into()
    );
    simulator
        .check_program_authority_matches(&authority_keypair_2.pubkey())
        .await;


    simulator
        .commit(
            &authority_keypair_2,
            &authority_keypair_1.pubkey(),
            1700000000 + 30,
        )
        .await
        .unwrap();
    simulator
        .check_program_authority_matches(
            &simulator.get_escrow_authority(&authority_keypair_1.pubkey(), 1700000000 + 30),
        )
        .await;

    assert_eq!(
        simulator
            .transfer(&authority_keypair_1.pubkey(), 1700000000 + 30)
            .await
            .unwrap_err()
            .unwrap(),
        ErrorCode::TimestampTooEarly.into()
    );
    simulator
        .check_program_authority_matches(
            &simulator.get_escrow_authority(&authority_keypair_1.pubkey(), 1700000000 + 30),
        )
        .await;

    simulator.warp_to_timestamp(1700000000 + 31).await.unwrap();

    simulator
        .transfer(&authority_keypair_1.pubkey(), 1700000000 + 30)
        .await
        .unwrap();
    simulator
        .check_program_authority_matches(&authority_keypair_1.pubkey())
        .await;
}
