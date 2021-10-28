use crate::{client::Client};

use trinci_core::crypto::Hash;
use trinci_core::base::serialize;
use std::fs::File;
use std::path::Path;
use std::io::Write;

use indicatif::{ProgressBar,ProgressStyle};
fn export_txs_from_block(txs:Vec<Hash>,client: &mut Client,file: &mut File) {
    txs.iter().for_each(|htx| {
        let tx = client.get_transaction(*htx).unwrap();
        let buf = serialize::rmp_serialize(&tx).expect("Error during serialization's transaction");
        file.write_all(&buf).expect("Unable to write file...");
        file.write("\n".as_bytes()).expect("Unable to write file...");
        
        //println!("TX: {:?}",buf);
    });
}
pub fn export_txs(file_name:&str,client: &mut Client) {
    let last_block = client.get_block(u64::MAX).expect("No block");
    let last_block = last_block.0.height;
    let path = Path::new(file_name);
    let display = path.display();

    // Open a file in write-only mode, returns `io::Result<File>`
    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}", display, why),
        Ok(file) => file,
    };
    let bar = ProgressBar::new(last_block.clone());
    bar.set_style(ProgressStyle::default_bar()
    .template("[{elapsed_precise}] {bar:40.cyan/blue}  {msg}")
    .progress_chars("#>-"));
    for i in 0..last_block {
        let current_block = client.get_block(i).unwrap();
        //print!("Block: {:?}",current_block.1);
        
        export_txs_from_block(current_block.1, client, &mut file);
        bar.set_message(format!("blocks: {}/{}", i + 1,last_block));
        bar.inc(1);
        //thread::sleep(Duration::from_millis(10));
    }
    bar.finish();
    println!("All transaction are exported into -> {}",file_name);

}
pub fn import_txs(file_name:&str,_client: &mut Client) {
    println!("Reading.... {}",file_name);
}