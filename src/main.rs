mod epoch_blocks;
mod secondary_authors;
mod traverse_chain;
pub mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // secondary_authors::find_secondary_authors(22957376).await?;
    traverse_chain::traverse(22959775, 22959675).await?;
    // epoch_blocks::fetch_blocks_in_epochs(50).await?;
    Ok(())
}
