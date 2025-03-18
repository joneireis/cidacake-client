use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use solana_sdk::{
    signature::{Signer, read_keypair_file},
    transaction::Transaction,
    program_pack::Pack,
};
use spl_token::ID as TOKEN_PROGRAM_ID;
use std::str::FromStr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Início da execução do cliente...");

    // Carregar a carteira
    println!("Carregando a chave do payer...");
    let payer = match read_keypair_file("/Users/joneirocha/cidacake-wallet.json") {
        Ok(keypair) => {
            println!("Chave do payer carregada com sucesso: {}", keypair.pubkey());
            keypair
        }
        Err(e) => {
            println!("Erro ao carregar a chave do payer: {:?}", e);
            return Err(e.into());
        }
    };

    // Definir o Program Id
    println!("Definindo o Program Id...");
    let program_id = match Pubkey::from_str("8Xi7zLZ3RMidVxQYbPomLasdywK2qcu7DfjXqDT5y9MP") {
        Ok(id) => {
            println!("Program Id definido: {}", id);
            id
        }
        Err(e) => {
            println!("Erro ao definir o Program Id: {:?}", e);
            return Err(e.into());
        }
    };

    // Definir a cake_account
    println!("Definindo a cake_account...");
    let cake_account_pubkey = match Pubkey::from_str("FELN5xApTHNKoA7cs4acxZtsFp3zbWUiMyXwn1DqSptR") {
        Ok(pubkey) => {
            println!("Cake Account Pubkey definida: {}", pubkey);
            pubkey
        }
        Err(e) => {
            println!("Erro ao definir a cake_account: {:?}", e);
            return Err(e.into());
        }
    };

    // Conectar ao cluster
    println!("Conectando ao cluster Devnet...");
    let client = RpcClient::new("https://api.devnet.solana.com".to_string());
    println!("Conexão ao cluster estabelecida.");

    println!("Iniciando validação das contas...");

    // Verificar as contas de token
    println!("Definindo a buyer_token_account...");
    let buyer_token_account = match Pubkey::from_str("7hJhA7P3QmPH37cth5ugpsMcsWk7iQBJqupSpE3W2AKu") {
        Ok(pubkey) => {
            println!("Buyer Token Account definida: {}", pubkey);
            pubkey
        }
        Err(e) => {
            println!("Erro ao definir a buyer_token_account: {:?}", e);
            return Err(e.into());
        }
    };

    println!("Definindo a owner_token_account...");
    let owner_token_account = match Pubkey::from_str("5ufohBPKyzfn8ZSFSGpuYJxgduwgkkgg4YrBwdY7JLKW") {
        Ok(pubkey) => {
            println!("Owner Token Account definida: {}", pubkey);
            pubkey
        }
        Err(e) => {
            println!("Erro ao definir a owner_token_account: {:?}", e);
            return Err(e.into());
        }
    };

    println!("Definindo o token_program...");
    let token_program = TOKEN_PROGRAM_ID;
    println!("Token Program definido: {}", token_program);

    // Verificar o saldo e o mint da conta do comprador
    println!("Obtendo informações da buyer_token_account...");
    let buyer_account = client.get_account(&buyer_token_account)?;
    let buyer_token_data = spl_token::state::Account::unpack(&buyer_account.data)?;
    println!("Mint da buyer_token_account: {}", buyer_token_data.mint);
    println!("Saldo da buyer_token_account: {} tokens ({} lamports)", 
             buyer_token_data.amount as f64 / 1_000_000_000.0, 
             buyer_token_data.amount);

    println!("Obtendo dados da buyer_token_account...");
    match client.get_account_data(&buyer_token_account) {
        Ok(data) => println!("Tamanho dos dados da buyer_token_account: {}", data.len()),
        Err(e) => println!("Erro ao obter dados da buyer_token_account: {:?}", e),
    };

    println!("Obtendo dados da owner_token_account...");
    match client.get_account_data(&owner_token_account) {
        Ok(data) => println!("Tamanho dos dados da owner_token_account: {}", data.len()),
        Err(e) => println!("Erro ao obter dados da owner_token_account: {:?}", e),
    };

    // Verificar o estado atual antes da venda
    println!("Obtendo dados da cake_account...");
    match client.get_account_data(&cake_account_pubkey) {
        Ok(account_data) => {
            println!("Tamanho dos dados da cake_account: {}", account_data.len());
            let cake_state = CakeState::unpack_from_slice(&account_data)?;
            println!("Stock antes: {}", cake_state.stock);
            println!("Price antes: {}", cake_state.price);
            println!("Owner antes: {}", cake_state.owner);
        }
        Err(e) => println!("Erro ao obter dados da cake_account: {:?}", e),
    };

    // Instrução 3: Vender 10 bolos
    println!("Construindo a instrução de venda...");
    let amount = 10u64;
    let mut sell_data = vec![3];
    sell_data.extend_from_slice(&amount.to_le_bytes());
    let sell_ix = solana_program::instruction::Instruction {
        program_id,
        accounts: vec![
            solana_program::instruction::AccountMeta::new_readonly(payer.pubkey(), true), // owner
            solana_program::instruction::AccountMeta::new(cake_account_pubkey, false),   // cake_account
            solana_program::instruction::AccountMeta::new_readonly(payer.pubkey(), true), // buyer
            solana_program::instruction::AccountMeta::new(buyer_token_account, false),    // buyer_token_account
            solana_program::instruction::AccountMeta::new(owner_token_account, false),    // owner_token_account
            solana_program::instruction::AccountMeta::new_readonly(token_program, false), // token_program
        ],
        data: sell_data.clone(),
    };
    println!("Instrução de venda construída: {:?}", sell_ix);

    println!("Obtendo o último blockhash...");
    let latest_blockhash = client.get_latest_blockhash()?;
    println!("Último blockhash obtido: {}", latest_blockhash);

    println!("Construindo a transação...");
    let tx = Transaction::new_signed_with_payer(
        &[sell_ix],
        Some(&payer.pubkey()),
        &[&payer],
        latest_blockhash,
    );
    println!("Transação construída.");

    // Simular a transação
    println!("Simulando a transação...");
    match client.simulate_transaction(&tx) {
        Ok(result) => println!("Resultado da simulação: {:?}", result),
        Err(e) => println!("Erro na simulação: {:?}", e),
    }

    println!("Enviando e confirmando a transação...");
    match client.send_and_confirm_transaction(&tx) {
        Ok(signature) => println!("Transaction signature (sell): {}", signature),
        Err(e) => println!("Erro na transação: {:?}", e),
    }

    // Verificar o estado atualizado
    println!("Obtendo estado atualizado da cake_account...");
    let account_data = client.get_account_data(&cake_account_pubkey)?;
    let cake_state = CakeState::unpack_from_slice(&account_data)?;
    println!("Stock atual: {}", cake_state.stock);
    println!("Price atual: {}", cake_state.price);
    println!("Owner: {}", cake_state.owner);

    println!("Fim da execução do cliente.");
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