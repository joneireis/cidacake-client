use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    signature::{Keypair, Signer, read_keypair_file},
    transaction::Transaction,
    program_pack::Pack,
    system_instruction,
    system_program,
};
use spl_token::ID as TOKEN_PROGRAM_ID;
use std::str::FromStr;
use std::fs::{File};
use std::io::{Read, Write};
use std::path::Path;
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

    // Carregar as carteiras
    let payer = read_keypair_file("/Users/joneirocha/cidacake-wallet.json")?;
    let owner = read_keypair_file("/Users/joneirocha/owner-wallet.json")?;

    // Definir o Program Id
    let program_id = Pubkey::from_str("nY3F2GFxvit5n6g1Ar6drGgSNcFYzwgixpcUxC9p722")?;

    // Conectar ao cluster
    let client = RpcClient::new("https://api.devnet.solana.com".to_string());

    // Carregar ou criar a cake_account
    let cake_account_file = "cake_account.txt";
    let cake_account_pubkey;

    // Tentar carregar a chave pública da cake_account do arquivo
    let loaded_pubkey = if Path::new(cake_account_file).exists() {
        let mut file = File::open(cake_account_file)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        match Pubkey::from_str(&contents.trim()) {
            Ok(pubkey) => {
                println!("Carregando cake_account existente: {}", pubkey);
                Some(pubkey)
            }
            Err(_) => {
                println!("Erro ao ler a chave pública da cake_account do arquivo. Criando uma nova...");
                None
            }
        }
    } else {
        println!("Arquivo cake_account.txt não encontrado. Criando uma nova cake_account...");
        None
    };

    // Verificar se a conta carregada é válida
    if let Some(pubkey) = loaded_pubkey {
        match client.get_account(&pubkey) {
            Ok(account) => {
                if account.owner == program_id {
                    cake_account_pubkey = pubkey;
                    println!("cake_account existente será reutilizada.");
                } else {
                    println!("cake_account existente não pertence ao programa atual. Criando uma nova...");
                    cake_account_pubkey = create_and_initialize_cake_account(&client, &payer, &program_id, cake_account_file)?;
                }
            }
            Err(_) => {
                println!("cake_account não encontrada na blockchain. Criando uma nova...");
                cake_account_pubkey = create_and_initialize_cake_account(&client, &payer, &program_id, cake_account_file)?;
            }
        }
    } else {
        // Criar uma nova cake_account
        cake_account_pubkey = create_and_initialize_cake_account(&client, &payer, &program_id, cake_account_file)?;
    }

    // Verificar as contas de token (necessárias apenas para a instrução de venda)
    let buyer_token_account;
    let owner_token_account;
    if action.as_str() == "sell" {
        buyer_token_account = Pubkey::from_str("7hJhA7P3QmPH37cth5ugpsMcsWk7iQBJqupSpE3W2AKu")?;
        owner_token_account = Pubkey::from_str("5ufohBPKyzfn8ZSFSGpuYJxgduwgkkgg4YrBwdY7JLKW")?;

        // Verificar o saldo da conta do comprador (tokens e SOL)
        let buyer_account = client.get_account(&buyer_token_account)?;
        let buyer_token_data = spl_token::state::Account::unpack(&buyer_account.data)?;
        println!("Saldo da buyer_token_account: {} tokens", 
                 buyer_token_data.amount as f64 / 1_000_000_000.0);

        let buyer_sol_balance = client.get_balance(&payer.pubkey())?;
        println!("Saldo de SOL do comprador: {} lamports", buyer_sol_balance);

        let owner_sol_balance = client.get_balance(&owner.pubkey())?;
        println!("Saldo de SOL do owner: {} lamports", owner_sol_balance);
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
                    AccountMeta::new(owner.pubkey(), true), // owner (gravável)
                    AccountMeta::new(cake_account_pubkey, false),   // cake_account
                    AccountMeta::new(payer.pubkey(), true), // buyer (gravável)
                    AccountMeta::new(buyer_token_account, false),    // buyer_token_account
                    AccountMeta::new(owner_token_account, false),    // owner_token_account
                    AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false), // token_program
                    AccountMeta::new_readonly(system_program::id(), false), // system_program
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
                    AccountMeta::new_readonly(owner.pubkey(), true), // owner
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
                    AccountMeta::new_readonly(owner.pubkey(), true), // owner
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
        &[&payer, &owner],
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

    // Verificar o saldo de SOL do comprador e do owner após a transação (se for uma venda)
    if action.as_str() == "sell" {
        let buyer_sol_balance = client.get_balance(&payer.pubkey())?;
        println!("Saldo de SOL do comprador após a transação: {} lamports", buyer_sol_balance);

        let owner_sol_balance = client.get_balance(&owner.pubkey())?;
        println!("Saldo de SOL do owner após a transação: {} lamports", owner_sol_balance);
    }

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

// Função auxiliar para criar e inicializar a cake_account
fn create_and_initialize_cake_account(
    client: &RpcClient,
    payer: &Keypair,
    program_id: &Pubkey,
    cake_account_file: &str,
) -> Result<Pubkey, Box<dyn std::error::Error>> {
    let cake_account = Keypair::new();
    let space = 48; // Tamanho da CakeState (8 + 8 + 32)
    let lamports = client.get_minimum_balance_for_rent_exemption(space)?;
    let create_account_instruction = system_instruction::create_account(
        &payer.pubkey(),
        &cake_account.pubkey(),
        lamports,
        space as u64,
        program_id, // Define o owner como o Program Id atual
    );

    let recent_blockhash = client.get_latest_blockhash()?;
    let create_account_tx = Transaction::new_signed_with_payer(
        &[create_account_instruction],
        Some(&payer.pubkey()),
        &[payer, &cake_account],
        recent_blockhash,
    );

    client.send_and_confirm_transaction(&create_account_tx)?;
    println!("Nova cake_account criada: {}", cake_account.pubkey());

    // Carregar a carteira do owner para assinar a inicialização
    let owner = read_keypair_file("/Users/joneirocha/owner-wallet.json")?;

    // Inicializar a cake_account com a instrução 0
    println!("Inicializando a cake_account...");
    let init_instruction = Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new_readonly(owner.pubkey(), true), // owner (signer)
            AccountMeta::new(cake_account.pubkey(), false),  // cake_account (gravável)
        ],
        data: vec![0], // Instrução 0 (inicializar)
    };

    let recent_blockhash = client.get_latest_blockhash()?;
    let init_tx = Transaction::new_signed_with_payer(
        &[init_instruction],
        Some(&payer.pubkey()),
        &[payer, &owner],
        recent_blockhash,
    );

    client.send_and_confirm_transaction(&init_tx)?;
    println!("cake_account inicializada com sucesso!");

    // Salvar a chave pública da cake_account em um arquivo
    let mut file = File::create(cake_account_file)?;
    writeln!(file, "{}", cake_account.pubkey())?;
    println!("Chave pública da cake_account salva em {}", cake_account_file);

    Ok(cake_account.pubkey())
}