use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    signature::{Signer, read_keypair_file},
    transaction::Transaction,
    program_pack::Pack,
};
use spl_token::ID as TOKEN_PROGRAM_ID;
use std::str::FromStr;
use std::fs::File;
use std::io::Read;
use clap::{Arg, Command};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configurar argumentos de linha de comando
    let matches = Command::new("Cidacake Client")
        .version("1.0")
        .about("Cliente para interagir com o programa Cidacake")
        .arg(Arg::new("action")
            .long("action")
            .required(true)
            .value_parser(["sell", "add_stock", "update_price"])
            .help("Ação a ser executada: sell, add_stock, update_price"))
        .arg(Arg::new("amount")
            .long("amount")
            .value_parser(clap::value_parser!(u64))
            .default_value("10")
            .help("Quantidade para a ação (número de bolos para venda ou estoque, ou novo preço em lamports)"))
        .get_matches();

    let action = matches.get_one::<String>("action").unwrap();
    let amount: u64 = *matches.get_one::<u64>("amount").unwrap();

    println!("Ação: {}", action);
    println!("Quantidade: {}", amount);

    // Carregar a carteira
    let payer = read_keypair_file("/Users/joneirocha/cidacake-wallet.json")?;

    // Definir o Program Id
    let program_id = Pubkey::from_str("GxQjJi33pdZDDCp2Kg5jssNANkJGPJ7jdFUyuiamMHLf")?;

    // Conectar ao cluster
    let client = RpcClient::new("https://api.devnet.solana.com".to_string());

    // Carregar a chave pública da cake_account do arquivo
    let cake_account_file = "cake_account.txt";
    let mut file = File::open(cake_account_file)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let cake_account_pubkey = Pubkey::from_str(&contents.trim())?;

    // Verificar as contas de token (necessárias apenas para a instrução de venda)
    let buyer_token_account;
    let owner_token_account;
    if action.as_str() == "sell" {
        buyer_token_account = Pubkey::from_str("7hJhA7P3QmPH37cth5ugpsMcsWk7iQBJqupSpE3W2AKu")?;
        owner_token_account = Pubkey::from_str("5ufohBPKyzfn8ZSFSGpuYJxgduwgkkgg4YrBwdY7JLKW")?;

        // Verificar o saldo da conta do comprador
        let buyer_account = client.get_account(&buyer_token_account)?;
        let buyer_token_data = spl_token::state::Account::unpack(&buyer_account.data)?;
        println!("Saldo da buyer_token_account: {} tokens", 
                 buyer_token_data.amount as f64 / 1_000_000_000.0);
    } else {
        buyer_token_account = Pubkey::default();
        owner_token_account = Pubkey::default();
    }

    // Verificar o estado atual antes da ação
    let account_data = client.get_account_data(&cake_account_pubkey)?;
    let cake_state = CakeState::unpack_from_slice(&account_data)?;
    println!("Stock antes: {}", cake_state.stock);
    println!("Price antes: {}", cake_state.price);
    println!("Owner antes: {}", cake_state.owner);

    // Construir a instrução com base na ação
    let instruction = match action.as_str() {
        "sell" => {
            let mut sell_data = vec![3];
            sell_data.extend_from_slice(&amount.to_le_bytes());
            Instruction {
                program_id,
                accounts: vec![
                    AccountMeta::new_readonly(payer.pubkey(), true), // owner
                    AccountMeta::new(cake_account_pubkey, false),   // cake_account
                    AccountMeta::new_readonly(payer.pubkey(), true), // buyer
                    AccountMeta::new(buyer_token_account, false),    // buyer_token_account
                    AccountMeta::new(owner_token_account, false),    // owner_token_account
                    AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false), // token_program
                ],
                data: sell_data,
            }
        }
        "add_stock" => {
            let mut add_stock_data = vec![1];
            add_stock_data.extend_from_slice(&amount.to_le_bytes());
            Instruction {
                program_id,
                accounts: vec![
                    AccountMeta::new_readonly(payer.pubkey(), true), // owner
                    AccountMeta::new(cake_account_pubkey, false),   // cake_account
                ],
                data: add_stock_data,
            }
        }
        "update_price" => {
            let mut update_price_data = vec![2];
            update_price_data.extend_from_slice(&amount.to_le_bytes());
            Instruction {
                program_id,
                accounts: vec![
                    AccountMeta::new_readonly(payer.pubkey(), true), // owner
                    AccountMeta::new(cake_account_pubkey, false),   // cake_account
                ],
                data: update_price_data,
            }
        }
        _ => unreachable!(), // clap já valida os valores possíveis
    };

    // Construir e enviar a transação
    let latest_blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&payer.pubkey()),
        &[&payer],
        latest_blockhash,
    );
    let signature = client.send_and_confirm_transaction(&tx)?;
    println!("Transaction signature: {}", signature);

    // Verificar o estado atualizado
    let account_data = client.get_account_data(&cake_account_pubkey)?;
    let cake_state = CakeState::unpack_from_slice(&account_data)?;
    println!("Stock atual: {}", cake_state.stock);
    println!("Price atual: {}", cake_state.price);
    println!("Owner atual: {}", cake_state.owner);

    Ok(())
}

#[derive(borsh::BorshDeserialize, Debug)]
struct CakeState {
    pub stock: u64,
    pub price: u64,
    pub owner: Pubkey,
}

impl CakeState {
    fn unpack_from_slice(src: &[u8]) -> Result<Self, solana_program::program_error::ProgramError> {
        borsh::BorshDeserialize::try_from_slice(src)
            .map_err(|_| solana_program::program_error::ProgramError::InvalidAccountData)
    }
}