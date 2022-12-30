use {
    crate::tests::simulator::EscrowSimulator,
    solana_sdk::{
        signature::Keypair,
        signer::Signer,
    },
};

#[tokio::test]
async fn test() {
    let (mut simulator, authority_keypair_1) = EscrowSimulator::new().await;
    let authority_keypair_2 = Keypair::new();

    let program_data = simulator.get_program_data().await;
    assert_eq!(
        program_data.upgrade_authority_address,
        Some(authority_keypair_1.pubkey())
    );
    simulator
        .propose(&authority_keypair_1, &authority_keypair_2.pubkey())
        .await
        .unwrap();

    let program_data = simulator.get_program_data().await;
    assert_eq!(
        program_data.upgrade_authority_address,
        Some(
            simulator
                .get_escrow_authority(&authority_keypair_1.pubkey(), &authority_keypair_2.pubkey())
        )
    );
    simulator
        .revert(&authority_keypair_1, &authority_keypair_2.pubkey())
        .await
        .unwrap();

    let program_data = simulator.get_program_data().await;
    assert_eq!(
        program_data.upgrade_authority_address,
        Some(authority_keypair_1.pubkey())
    );
    simulator
        .propose(&authority_keypair_1, &authority_keypair_2.pubkey())
        .await
        .unwrap();

    let program_data = simulator.get_program_data().await;
    assert_eq!(
        program_data.upgrade_authority_address,
        Some(
            simulator
                .get_escrow_authority(&authority_keypair_1.pubkey(), &authority_keypair_2.pubkey())
        )
    );
    simulator
        .accept(&authority_keypair_1.pubkey(), &authority_keypair_2)
        .await
        .unwrap();

    let program_data = simulator.get_program_data().await;
    assert_eq!(
        program_data.upgrade_authority_address,
        Some(authority_keypair_2.pubkey())
    );

    simulator
        .propose(&authority_keypair_2, &authority_keypair_1.pubkey())
        .await
        .unwrap();
    let program_data = simulator.get_program_data().await;
    assert_eq!(
        program_data.upgrade_authority_address,
        Some(
            simulator
                .get_escrow_authority(&authority_keypair_2.pubkey(), &authority_keypair_1.pubkey())
        )
    );

    simulator
        .revert(&authority_keypair_2, &authority_keypair_1.pubkey())
        .await
        .unwrap();

    let program_data = simulator.get_program_data().await;
    assert_eq!(
        program_data.upgrade_authority_address,
        Some(authority_keypair_2.pubkey())
    );

    simulator
        .propose(&authority_keypair_2, &authority_keypair_1.pubkey())
        .await
        .unwrap();
    let program_data = simulator.get_program_data().await;
    assert_eq!(
        program_data.upgrade_authority_address,
        Some(
            simulator
                .get_escrow_authority(&authority_keypair_2.pubkey(), &authority_keypair_1.pubkey())
        )
    );


    simulator
        .accept(&authority_keypair_2.pubkey(), &authority_keypair_1)
        .await
        .unwrap();

    let program_data = simulator.get_program_data().await;
    assert_eq!(
        program_data.upgrade_authority_address,
        Some(authority_keypair_1.pubkey())
    );
}
