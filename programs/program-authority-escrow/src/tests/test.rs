use {
    crate::tests::simulator::EscrowSimulator,
    solana_sdk::{
        signature::Keypair,
        signer::Signer,
    },
};

#[tokio::test]
async fn test() {
    let new_authority_keypair = Keypair::new();
    let (mut simulator, current_authority_keypair) = EscrowSimulator::new().await;

    let program_data = simulator.get_program_data().await;
    assert_eq!(
        program_data.upgrade_authority_address,
        Some(current_authority_keypair.pubkey())
    );
    simulator
        .propose(&current_authority_keypair, &new_authority_keypair.pubkey())
        .await
        .unwrap();

    let program_data = simulator.get_program_data().await;
    assert_eq!(
        program_data.upgrade_authority_address,
        Some(simulator.get_escrow_authority(
            &current_authority_keypair.pubkey(),
            &new_authority_keypair.pubkey()
        ))
    );
    simulator
        .revert(&current_authority_keypair, &new_authority_keypair.pubkey())
        .await
        .unwrap();

    let program_data = simulator.get_program_data().await;
    assert_eq!(
        program_data.upgrade_authority_address,
        Some(current_authority_keypair.pubkey())
    );
    simulator
        .propose(&current_authority_keypair, &new_authority_keypair.pubkey())
        .await
        .unwrap();

    let program_data = simulator.get_program_data().await;
    assert_eq!(
        program_data.upgrade_authority_address,
        Some(simulator.get_escrow_authority(
            &current_authority_keypair.pubkey(),
            &new_authority_keypair.pubkey()
        ))
    );
    simulator
        .accept(&current_authority_keypair.pubkey(), &new_authority_keypair)
        .await
        .unwrap();

    let program_data = simulator.get_program_data().await;
    assert_eq!(
        program_data.upgrade_authority_address,
        Some(new_authority_keypair.pubkey())
    );
}
