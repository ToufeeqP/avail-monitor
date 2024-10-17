# Avail Monitor

A command-line utility for monitoring Avail, a Substrate-based blockchain, offering functionalities to monitor blocks produced in an epoch and era, traverse chain blocks, fetch epoch block data, and determine secondary slot authors.

## Table of Contents
- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
- [Commands](#commands)
- [Examples](#examples)

### Features
- Traverse the blockchain in reverse order and record storage values.
- Fetch the number of blocks produced in each epoch for the last `n` epochs.
- Determine secondary slot authors for specified epochs.
- Monitors chain to determine number of blocks produced in an epoch/era when it ends

### Installation
To use this tool, you'll need to have Rust installed on your machine. You can install Rust using [rustup](https://rustup.rs/).

1. Clone this repository:
   ```bash
   git clone https://github.com/ToufeeqP/avail-monitor.git
   cd avail-monitor
   ```

2. Build the project:

    ```bash
    cargo build --release
    ```

### Usage

You can invoke the tool by running the following command:

```bash
./target/release/avail-monitor --ws <WebSocket URL> <COMMAND> [options]
```

### Commands

The tool supports the following commands:
- `traverse`: Traverse the chain in reverse order from a start block to its parent until the end block is reached.
- `epoch-blocks`: Fetch the number of blocks produced in each epoch for the last n epochs.
- `secondary-authors`: Determine secondary slot authors for an epoch based on the block number at which the epoch started.
- `chain-monitor`: Monitors chain to determine number of blocks produced in an epoch/era when it ends.


### Example

1. Traverse the chain

```bash
./target/release/avail-monitor --ws ws://127.0.0.1:9944 traverse 1000 500
```

2. Fetch epoch blocks

```bash
./target/release/avail-monitor --ws ws://127.0.0.1:9944 epoch-blocks 50
```

3. Find secondary authors

```bash
./target/release/avail-monitor --ws ws://127.0.0.1:9944 secondary-authors 100
```

4. Monitor chain

```bash
./target/release/avail-monitor --ws ws://127.0.0.1:9944 chain-monitor
```

Optionally, you can send updates to Slack by providing a CHANNEL-ID and setting the SLACK_TOKEN env:

```bash
SLACK_TOKEN="your-slack-token" ./target/release/avail-monitor chain-monitor --channel-id <CHANNEL-ID>
```
