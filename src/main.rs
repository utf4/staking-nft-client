use clap::{
    app_from_crate, crate_authors, crate_description, crate_name, crate_version, Arg, SubCommand,
};
use solana_client::rpc_client::RpcClient;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{read_keypair_file, Signer};
#[allow(unused_imports)]
use solana_sdk::signer::signers::Signers;
use solana_sdk::transaction::Transaction;
use solana_sdk::system_program;
use borsh::{BorshDeserialize, BorshSerialize,BorshSchema};
use solana_sdk::commitment_config::CommitmentConfig;
use spl_token;
use spl_token_metadata;
use spl_associated_token_account;
#[allow(unused_imports)]
use solana_sdk::signer::keypair::Keypair;
use solana_sdk::borsh::try_from_slice_unchecked;

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
enum StakeInstruction{
    GenerateVault{
        #[allow(dead_code)]
        min_period:u64,
        #[allow(dead_code)]
        reward_period:u64,
    },
    Stake,
    Unstake,
    AddToWhitelist{
        #[allow(dead_code)]
        price:u64,
    },
    Withdraw{
        #[allow(dead_code)]
        amount:u64,
    },
}

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
struct StakeData{
    timestamp: u64,
    staker: Pubkey,
    active: bool,
}

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
struct ContractData{
    min_period: u64,
    reward_period: u64,
}


#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
struct RateData{
    price: u64,
}

fn main() {
    let matches = app_from_crate!()
        .subcommand(SubCommand::with_name("generate_vault_address")
            .arg(Arg::with_name("sign")
                .short("s")
                .long("sign")
                .required(true)
                .takes_value(true)
            )
            .arg(Arg::with_name("env")
                .short("e")
                .long("env")
                .required(false)
                .takes_value(true)
            )
            .arg(Arg::with_name("min_period")
                .short("m")
                .long("min_period")
                .required(false)
                .takes_value(true)
            )
            .arg(Arg::with_name("reward_period")
                .short("r")
                .long("reward_period")
                .required(false)
                .takes_value(true)
            )
        )
        .subcommand(SubCommand::with_name("add_to_whitelist")
            .arg(Arg::with_name("sign")
                .short("s")
                .long("sign")
                .required(true)
                .takes_value(true)
            )
            .arg(Arg::with_name("env")
                .short("e")
                .long("env")
                .required(false)
                .takes_value(true)
            )
            .arg(Arg::with_name("candy_machine")
                .short("c")
                .long("candy_machine")
                .required(false)
                .takes_value(true)
            )
            .arg(Arg::with_name("reward")
                .short("r")
                .long("reward")
                .required(false)
                .takes_value(true)
            )
        )
        .subcommand(SubCommand::with_name("stake")
            .arg(Arg::with_name("sign")
                .short("s")
                .long("sign")
                .required(true)
                .takes_value(true)
            )
            .arg(Arg::with_name("env")
                .short("e")
                .long("env")
                .required(false)
                .takes_value(true)
            )
            .arg(Arg::with_name("nft")
                .short("n")
                .long("nft")
                .required(true)
                .takes_value(true)
            )
        )
        .subcommand(SubCommand::with_name("unstake")
            .arg(Arg::with_name("sign")
                .short("s")
                .long("sign")
                .required(true)
                .takes_value(true)
            )
            .arg(Arg::with_name("env")
                .short("e")
                .long("env")
                .required(false)
                .takes_value(true)
            )
            .arg(Arg::with_name("nft")
                .short("n")
                .long("nft")
                .required(true)
                .takes_value(true)
            )
        )
        .subcommand(SubCommand::with_name("withdraw")
            .arg(Arg::with_name("sign")
                .short("s")
                .long("sign")
                .required(true)
                .takes_value(true)
            )
            .arg(Arg::with_name("env")
                .short("e")
                .long("env")
                .required(false)
                .takes_value(true)
            )
            .arg(Arg::with_name("amount")
                .short("a")
                .long("amount")
                .required(true)
                .takes_value(true)
            )
        )
        .get_matches();

    let program_id = "AxiFxRWafjidUFpnkfGmAC2iYMdteVZw8WdCrNQtkzL6".parse::<Pubkey>().unwrap();
    let reward_mint = "Aoz9EBZPZ8oQHnuV8UY5bCV87xJ5DpwFcy84TrRWBCzp".parse::<Pubkey>().unwrap();

    if let Some(matches) = matches.subcommand_matches("withdraw") {
        let url = match matches.value_of("env"){
            Some("dev")=>"https://api.devnet.solana.com",
            _=>"https://api.mainnet-beta.solana.com",
        };
        let client = RpcClient::new_with_commitment(url.to_string(),CommitmentConfig::confirmed());
        
        let wallet_path = matches.value_of("sign").unwrap();
        let wallet_keypair = read_keypair_file(wallet_path).expect("Can't open file-wallet");
        let wallet_pubkey = wallet_keypair.pubkey();

        let amount = matches.value_of("amount").unwrap().parse::<u64>().unwrap();
        let ( vault, _vault_bump ) = Pubkey::find_program_address(&[&"vault".as_bytes()], &program_id);
        let reward_destanation = spl_associated_token_account::get_associated_token_address(&wallet_pubkey, &reward_mint);
        let reward_source = spl_associated_token_account::get_associated_token_address(&vault, &reward_mint);

        let instarctions = vec![Instruction::new_with_borsh(
            program_id,
            &StakeInstruction::Withdraw{amount},
            vec![
                AccountMeta::new(wallet_pubkey, true),
                AccountMeta::new(reward_destanation, false),
                AccountMeta::new(reward_source, false),
                AccountMeta::new_readonly(vault, false),
                AccountMeta::new_readonly(reward_mint, false),
                AccountMeta::new_readonly(system_program::id(), false),
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly("SysvarRent111111111111111111111111111111111".parse::<Pubkey>().unwrap(), false),
                AccountMeta::new_readonly("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL".parse::<Pubkey>().unwrap(), false),
            ],
        )];
        let mut tx = Transaction::new_with_payer(&instarctions, Some(&wallet_pubkey));
        let (recent_blockhash, _) = client.get_recent_blockhash().expect("Can't get blockhash");
        tx.sign(&vec![&wallet_keypair], recent_blockhash);
        let id = client.send_transaction(&tx).expect("Transaction failed.");
        println!("tx id: {:?}", id);
    }

    if let Some(matches) = matches.subcommand_matches("unstake") {
        let url = match matches.value_of("env"){
            Some("dev")=>"https://api.devnet.solana.com",
            _=>"https://api.mainnet-beta.solana.com",
        };
        let client = RpcClient::new_with_commitment(url.to_string(),CommitmentConfig::confirmed());
        
        let wallet_path = matches.value_of("sign").unwrap();
        let wallet_keypair = read_keypair_file(wallet_path).expect("Can't open file-wallet");
        let wallet_pubkey = wallet_keypair.pubkey();

        let nft = matches.value_of("nft").unwrap().parse::<Pubkey>().unwrap();
        let (metadata,_) =Pubkey::find_program_address(&["metadata".as_bytes(), &spl_token_metadata::ID.to_bytes(), &nft.to_bytes()], &spl_token_metadata::ID);
        let ( vault, _vault_bump ) = Pubkey::find_program_address(&[&"vault".as_bytes()], &program_id);
        let destanation = spl_associated_token_account::get_associated_token_address(&wallet_pubkey, &nft);
        let source = spl_associated_token_account::get_associated_token_address(&vault, &nft);
        let reward_destanation = spl_associated_token_account::get_associated_token_address(&wallet_pubkey, &reward_mint);
        let reward_source = spl_associated_token_account::get_associated_token_address(&vault, &reward_mint);
        let ( stake_data, _ ) = Pubkey::find_program_address(&[&nft.to_bytes()], &program_id);

        let metadata_data = client.get_account_data(&metadata).unwrap();
        let metadata_data_struct: spl_token_metadata::state::Metadata = try_from_slice_unchecked(&metadata_data[..]).unwrap();
        let candy_machine = metadata_data_struct.data.creators.unwrap().first().unwrap().address;

        let (wl_data_address,_wl_data_address_bump) = Pubkey::find_program_address(&["whitelist".as_bytes(), &candy_machine.to_bytes()], &program_id);
        
        let instarctions = vec![Instruction::new_with_borsh(
            program_id,
            &StakeInstruction::Unstake,
            vec![
                AccountMeta::new(wallet_pubkey, true),
                AccountMeta::new_readonly(system_program::id(), false),
                AccountMeta::new_readonly(nft, false),
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly("SysvarRent111111111111111111111111111111111".parse::<Pubkey>().unwrap(), false),
                AccountMeta::new_readonly("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL".parse::<Pubkey>().unwrap(), false),
                AccountMeta::new(stake_data, false),
                AccountMeta::new_readonly(vault, false),
                AccountMeta::new(reward_destanation, false),
                AccountMeta::new(reward_source, false),
                AccountMeta::new(destanation, false),
                AccountMeta::new(source, false),
                AccountMeta::new_readonly(metadata, false),
                AccountMeta::new(wl_data_address, false),
                AccountMeta::new_readonly(reward_mint, false),
            ],
        )];
        let mut tx = Transaction::new_with_payer(&instarctions, Some(&wallet_pubkey));
        let (recent_blockhash, _) = client.get_recent_blockhash().expect("Can't get blockhash");
        tx.sign(&vec![&wallet_keypair], recent_blockhash);
        let id = client.send_transaction(&tx).expect("Transaction failed.");
        println!("tx id: {:?}", id);
    }

    if let Some(matches) = matches.subcommand_matches("stake") {
        let url = match matches.value_of("env"){
            Some("dev")=>"https://api.devnet.solana.com",
            _=>"https://api.mainnet-beta.solana.com",
        };
        let client = RpcClient::new_with_commitment(url.to_string(),CommitmentConfig::confirmed());
        
        let wallet_path = matches.value_of("sign").unwrap();
        let wallet_keypair = read_keypair_file(wallet_path).expect("Can't open file-wallet");
        let wallet_pubkey = wallet_keypair.pubkey();

        let nft = matches.value_of("nft").unwrap().parse::<Pubkey>().unwrap();
        let (metadata,_) =Pubkey::find_program_address(&["metadata".as_bytes(), &spl_token_metadata::ID.to_bytes(), &nft.to_bytes()], &spl_token_metadata::ID);
        let ( vault, _vault_bump ) = Pubkey::find_program_address(&[&"vault".as_bytes()], &program_id);
        let source = spl_associated_token_account::get_associated_token_address(&wallet_pubkey, &nft);
        let destanation = spl_associated_token_account::get_associated_token_address(&vault, &nft);
        let ( stake_data, _ ) = Pubkey::find_program_address(&[&nft.to_bytes()], &program_id);

        let metadata_data = client.get_account_data(&metadata).unwrap();
        let metadata_data_struct: spl_token_metadata::state::Metadata = try_from_slice_unchecked(&metadata_data[..]).unwrap();
        let candy_machine = metadata_data_struct.data.creators.unwrap().first().unwrap().address;

        let (wl_data_address,_wl_data_address_bump) = Pubkey::find_program_address(&["whitelist".as_bytes(), &candy_machine.to_bytes()], &program_id);
        
        let instarctions = vec![Instruction::new_with_borsh(
            program_id,
            &StakeInstruction::Stake,
            vec![
                AccountMeta::new(wallet_pubkey, true),
                AccountMeta::new_readonly(nft, false),
                AccountMeta::new_readonly(metadata, false),
              
                AccountMeta::new_readonly(vault, false),
                AccountMeta::new(source, false),
                AccountMeta::new(destanation, false),

                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly(system_program::id(), false),
                AccountMeta::new_readonly("SysvarRent111111111111111111111111111111111".parse::<Pubkey>().unwrap(), false),
                AccountMeta::new_readonly("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL".parse::<Pubkey>().unwrap(), false),

                AccountMeta::new(stake_data, false),
                AccountMeta::new(wl_data_address, false),
            ],
        )];
        let mut tx = Transaction::new_with_payer(&instarctions, Some(&wallet_pubkey));
        let (recent_blockhash, _) = client.get_recent_blockhash().expect("Can't get blockhash");
        tx.sign(&vec![&wallet_keypair], recent_blockhash);
        let id = client.send_transaction(&tx).expect("Transaction failed.");
        println!("tx id: {:?}", id);
    }


    if let Some(matches) = matches.subcommand_matches("add_to_whitelist") {
        let url = match matches.value_of("env"){
            Some("dev")=>"https://api.devnet.solana.com",
            _=>"https://api.mainnet-beta.solana.com",
        };
        let client = RpcClient::new_with_commitment(url.to_string(),CommitmentConfig::confirmed());
        
        let wallet_path = matches.value_of("sign").unwrap();
        let wallet_keypair = read_keypair_file(wallet_path).expect("Can't open file-wallet");
        let wallet_pubkey = wallet_keypair.pubkey();

        let candy_machine = matches.value_of("candy_machine").unwrap().parse::<Pubkey>().unwrap();
        let reward = matches.value_of("reward").unwrap().parse::<u64>().unwrap();

        let (wl_address, _) = Pubkey::find_program_address(&["whitelist".as_bytes(), &candy_machine.to_bytes()], &program_id);

        let instarctions = vec![Instruction::new_with_borsh(
            program_id,
            &StakeInstruction::AddToWhitelist{price:reward},
            vec![
                AccountMeta::new(wallet_pubkey, true),
                AccountMeta::new(candy_machine, false),
                AccountMeta::new(wl_address, false),
                AccountMeta::new(system_program::id(), false),
                AccountMeta::new_readonly("SysvarRent111111111111111111111111111111111".parse::<Pubkey>().unwrap(), false),
            ],
        )];
        let mut tx = Transaction::new_with_payer(&instarctions, Some(&wallet_pubkey));
        let (recent_blockhash, _) = client.get_recent_blockhash().expect("Can't get blockhash");
        tx.sign(&vec![&wallet_keypair], recent_blockhash);
        let id = client.send_transaction(&tx).expect("Transaction failed.");
        println!("tx id: {:?}", id);

    }

    if let Some(matches) = matches.subcommand_matches("generate_vault_address") {
        let url = match matches.value_of("env"){
            Some("dev")=>"https://api.devnet.solana.com",
            _=>"https://api.mainnet-beta.solana.com",
        };
        let client = RpcClient::new_with_commitment(url.to_string(),CommitmentConfig::confirmed());
        
        let wallet_path = matches.value_of("sign").unwrap();
        let wallet_keypair = read_keypair_file(wallet_path).expect("Can't open file-wallet");
        let wallet_pubkey = wallet_keypair.pubkey();

        let min_period = matches.value_of("min_period").unwrap().parse::<u64>().unwrap();
        let reward_period = matches.value_of("reward_period").unwrap().parse::<u64>().unwrap();


        let (vault_pda, _) = Pubkey::find_program_address(&["vault".as_bytes()], &program_id);

        let instarctions = vec![Instruction::new_with_borsh(
            program_id,
            &StakeInstruction::GenerateVault{min_period,reward_period},
            vec![
                AccountMeta::new(wallet_pubkey, true),
                AccountMeta::new(system_program::id(), false),
                AccountMeta::new(vault_pda, false),
                AccountMeta::new_readonly("SysvarRent111111111111111111111111111111111".parse::<Pubkey>().unwrap(), false),
            ],
        )];
        let mut tx = Transaction::new_with_payer(&instarctions, Some(&wallet_pubkey));
        let (recent_blockhash, _) = client.get_recent_blockhash().expect("Can't get blockhash");
        tx.sign(&vec![&wallet_keypair], recent_blockhash);
        let id = client.send_transaction(&tx).expect("Transaction failed.");
        println!("vault account generated: {:?}", vault_pda);
        println!("tx id: {:?}", id);
    }
}
