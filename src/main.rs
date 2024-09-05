use clap::{Parser, Subcommand};
use chrono::{DateTime,Utc};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{account::from_account, clock::Clock, commitment_config::CommitmentConfig, sysvar,native_token::lamports_to_sol,
    signature::{keypair_from_seed, write_keypair_file},
    signer::Signer,
};
use bip39::{Language, Mnemonic, MnemonicType, Seed};
use tokio;


#[derive(Parser)]

#[clap(author, version, about, long_about=None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>
}

#[derive(Subcommand)]
enum Commands {
    ClusterInfo,
    Supply,
    KeyGen{
     #[arg(short, long, help = "Output file path for keypair file.")]
        output: String,
        #[arg(
            short,
            long,
            default_value_t = 12,
            help = "How many words to generate for the mnemonic. Valid values are: 12, 15, 18, 21, and 24."
        )]
        mnemonic_word_count: u32,
        #[arg(short, long, help = "Passphrase to use for extra security.")]
        passphrase: Option<String>,
    },
}

const SERVER_URL: &str = "https://api.devnet.solana.com";
async fn get_cluster_info(client:&RpcClient){
    let version = client.get_version().await.unwrap();
    //print!("the version of solana is {}",version);
    let result=client.get_account_with_commitment(&sysvar::clock::id(),CommitmentConfig::finalized()).await.unwrap();
    let (slot, timestamp) = match result.value {
        Some(clock_account) => {
            let clock: Clock = from_account(&clock_account).unwrap();
            (result.context.slot, clock.unix_timestamp)
        }
        None => {
            panic!("Unexpected None");
        }
    };

    let datetime_utc: DateTime<Utc> = DateTime::from_timestamp(timestamp, 0).unwrap();
    println!(
        "Block: {}, Time: {}",
        slot,
        datetime_utc.format("%Y-%m-%d %H:%M:%S")
    );
}
async fn get_supply(client:&RpcClient){
    let supply_response=client.supply().await.unwrap();
    let supply=supply_response.value;
    println!(
        "Total supply: {} SOL\nCirculating: {} SOL\nNon-Circulating: {} SOL",
        lamports_to_sol(supply.total),
        lamports_to_sol(supply.circulating),
        lamports_to_sol(supply.non_circulating)
    );
}
fn generate_keypair(output_path: &str, mnemonic_word_count: usize, passphrase: &Option<String>) {
    let mnemonic_type = MnemonicType::for_word_count(mnemonic_word_count).unwrap();
    let mnemonic = Mnemonic::new(mnemonic_type, Language::English);
    let seed = match passphrase {
        Some(phrase) => Seed::new(&mnemonic, phrase),
        None => Seed::new(&mnemonic, ""),
    };
    let keypair = keypair_from_seed(seed.as_bytes()).unwrap();
    write_keypair_file(&keypair, output_path).unwrap();
    println!("Mnemonic: {:?}", mnemonic);
    println!("Public key: {}", &keypair.pubkey());
}
#[tokio::main] 
 async fn main() {
    let cli = Cli::parse();
    let client = RpcClient::new(SERVER_URL.to_string());
    match &cli.command {
        Some(Commands::ClusterInfo) => {
            get_cluster_info(&client).await;
        },
        Some(Commands::Supply)=>{
            get_supply(&client).await; }
        Some(Commands::KeyGen { output, mnemonic_word_count, passphrase })=>{
            println!("Generate keys, output to: {}", output);
            generate_keypair(output, *mnemonic_word_count as usize, passphrase);
        }
        None => {}
    }
}
