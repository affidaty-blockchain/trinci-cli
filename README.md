TRINCI
======

TRINCI Blockchain CLI.

# Trinci-CLI help
To see all the available trinci-cli options from the project directory launch the command:
```bash
$ cargo run -- --help
```

```bash
T2 CLI 0.1.6
The Affidaty Team <trinci@affidaty.io>


USAGE:
    trinci-cli [FLAGS] [OPTIONS]

FLAGS:
        --help       Prints help information
    -V, --version    Prints version information
    -v, --verbose    Print sent and received structures

OPTIONS:
        --batch <PATH>         Submit the ordered set of transactions specified in the file
    -c, --channel <CHANNEL>    IO channel (default stdio) [possible values: stdio, file, http, bridge]
    -h, --host <HOST>          Target hostname (default localhost)
        --keyfile <PATH>       Keypair file (PKCS#8 DER format)
    -n, --network <NETWORK>    Blockchain network identifier (default skynet)
        --path <PATH>          API path (default '/api/v1')
    -p, --port <PORT>          Target port (default 8000)
        --stress <threads>     Run stress test using the specified number of threads (stop with CTRL^C)


```


# Main menu
If we type `help` int the trinci-cli prompt we can see all the available options:

```bash
> help
Available commands:
 * 'help': pr-nt this help
 * 'put-tx': submit a transaction and get ticket
 * 'get-tx <tkt>': get transaction by ticket
 * 'get-rx <tkt>': get receipt by transaction ticket
 * 'get-acc <id>': get account by id
 * 'get-blk <height>': get block at a given height
 * 'cheats': shortcuts for common tasks
 * 'quit': exit the application
> 
```

## Cheats Menu

In the `cheats` submenu we can find some facilitators for common operations:
```bash
> cheats
 * 'help': print this help
 * 'transfer': transfer an asset to an account
 * 'asset-init': initialize asset account
 * 'register': register a contract to the service account
 * 'subscribe': subscribe to blockchain events (stop with ctrl+c)
 * 'quit': back to main menu
>>> 
```

# Create Binary batch transactions
```bash
$ cargo run -- --network <network> --channel file
```

Example:
```bash
$ cargo run -- --network skynet --channel file
```

```bash
> cheats
```

```bash
>>> register
```

```bash
  Service account: QmfZy5bvk7a3DQAjCbGNtmrPXWkyVvPrdnZMyBZ5q5ieKG
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

# Stress test
This modality create N threads (where N is the parameter `<threads>` on the command) that perform a
series of transfer on the targeted blockchain.
It is used to valutate the node performance.