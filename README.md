TRINCI
======

TRINCI Blockchain CLI.

# Trinci-CLI help
To see all the available trinci-cli options from the project directory launch the command:
```bash
$ cargo run -- --help
```

```bash
T2 CLI 0.2.7
The Affidaty Team <trinci@affidaty.io>

USAGE:
    trinci-cli [OPTIONS]

OPTIONS:
        --batch <PATH>         Submit the ordered set of transactions specified in the file
    -c, --channel <CHANNEL>    IO channel (default stdio) [possible values: stdio, file, http,
                               bridge]
    -h, --host <HOST>          Target hostname (default localhost)
        --help                 Print help information
        --keyfile <PATH>       Keypair file (PKCS#8 DER format)
    -n, --network <NETWORK>    Blockchain network identifier (default skynet)
    -p, --port <PORT>          Target port (default 8000)
        --path <PATH>          API path (default '/api/v1')
        --stress <threads>     Run stress test using the specified number of threads (stop with
                               CTRL^C)
    -v, --verbose              Print sent and received structures
    -V, --version              Print version information
```


# Main menu
If we type `help` int the trinci-cli prompt we can see all the available options:

```bash
> help
Available commands:
 * 'help': print this help
 * 'put-tx': submit a transaction and get ticket
 * 'put-bulk-tx': submit a bulk transaction and get ticket
 * 'get-tx <tkt>': get transaction by ticket
 * 'get-rx <tkt>': get receipt by transaction ticket
 * 'get-acc <id>': get account by id
 * 'get-blk <height>': get block at a given height
 * 'export-txs <file>': export all transactions into a file
 * 'import-txs <file>': import transactions from a file
 * 'cheats': shortcuts for common tasks
 * 'quit': exit the application
> 
```

## Cheats Menu

In the `cheats` submenu we can find some facilitators for common operations:
```bash
> cheats
 * 'help': print this help
 * 'register': register a contract to the service account
 * 'subscribe': subscribe to blockchain events (stop with ctrl+c)
 * 'bootstrap': create a new bootstrap.bin
 * 'adv-asset': advanced asset cheats
 * 'quit': back to main menu
>>> 
```

### Cheats: Advanced Asset
In the `cheats -> adv-asset` submenu we can find facilitators for operation on the advanced asset:
```bash
>>> adv-asset
 @ 'help': print this help
 @ 'init': initialize the advanced asset in an account
 @ 'mint': initialize asset account
 @ 'balance': transfer an asset to an account
 @ 'transfer': transfer an asset to an account
 @ 'quit': back to cheats menu
 ```
# Create Binary batch transactions
```bash
$ cargo run -- --network <network> --channel file
```

Example:
```bash
$ cargo run -- --network Qm...yh --channel file
```

```bash
> cheats
```

```bash
>>> register
```

```bash
  Service account: TRINCI
  Service contract (optional multihash hex string):
  New contract name: vote
  New contract version: 0.1.2
  New contract description: Contract for voting
  New contract url: www.trinci.io
  New contract filename: <wasm_file_path>
```

Ignore the error that gives:
```bash
Error: other
``` 

Quitting the cli by entering 
```bash
>>> quit<enter>
```

```bash
> quit<enter>
```

The binary transaction can be found in the current directory and has the format:
```bash
out-1634113488.bin
```

# Execute binary transactions
In the same directory as the transaction binaries create a file with extension `.toml` with the following content:

```bash
# Pre-built transactions batches.

# Interactive batch execution.
interactive = true

[[batch.0]]
desc = 'Register vote contract'
file = 'register-vote-0.1.2.bin'
```

 - `interactive` = true
   ask confirmation for every transaction
 - [[batch.N]]
   if we want the transaction registered on a different block we must add them with different number

To execute the batch transactions:
```bash
$ cargo run -- --batch <file.toml> --host <host> --path <path> --port <port> --network <network>
```

# Create a new bootstrap.bin
 - Clone the [trinci-smartcontracts](https://github.com/affidaty-blockchain/trinci-smartcontracts) repository and search for a `service.wasm` file. 
   You could also modify and rebuild the Service contract that you can find in `trinci-smartcontracts/app-as/service` directory
 - Launch the `trinci-cli` with a custom keypair, the correspondent account-id will be used as blockchain admin in the Service account:
   ```bash
   $ cargo run -- --keyfile <you_binary_keyfile>
   ```
   - Go to the `cheats` submenu
     - Choose the `bootstrap` option
       - Insert the `service.wasm` complete path
         ```bash
         Service wasm contract path: <path_to_service.wasm>
         ```
       - Choose if you want to automatically create the service account init transaction (this will create a "TRINCI" service account)
         ```bash
         Create service init?   >> Please answer with Y or N: Y
         ```
         - Insert the initial network id, the default value is: `bootstrap`
         ```bash
         Network ID: bootstrap
         ```
         - Insert the account-id to use as Service Account:
         ```bash
         Service account: TRINCI
         ```
       - If you have available some binary transaction you can put the into a directory and add the to the `bootstrap.bin`
         ```bash
         Load txs from directory?   >> Please answer with Y or N: Y
         txs path: <path_to_txs_directory>
         Added tx: register-asset.bin
         Added tx: init-asset.bin
         ```
       - You can decide to create a unique bootstrap (and also a new network starting from the same service.wasm and txs) using a random string as nonce:
         ```bash
         Create unique bootstrap?   >> Please answer with Y or N: 
         nonce: sB0yIl94Ou
         ```
  - Now you can find a new `bootstrap.bin` file in the trinci-cli directory.


# Stress test
This modality create N threads (where N is the parameter `<threads>` on the command) that perform a
series of transfer on the targeted blockchain.
It is used to evaluate the node performance.

This procedure is interactive:
- we can register a new contract for the advanced asset
- we can initialize the contract on an account
- we can mint some units on the test accounts