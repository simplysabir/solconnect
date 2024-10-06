# Solana Address Connector CLI

The Solana Address Connector CLI is a powerful tool designed to analyze and visualize connections between Solana wallet addresses. It helps users understand the relationship between different addresses by tracing transactions and building a graph of connections.

## Features

- Fetch transaction history for Solana addresses
- Build a graph of connections between addresses
- Find paths between two given addresses
- Support for large transaction histories (up to 10,000 transactions per address)
- Configurable RPC endpoint via environment variable

## Prerequisites

- Rust programming language (latest stable version)
- Cargo package manager
- Access to a Solana RPC endpoint (e.g., Helius, QuickNode, or your own node)

## Installation

1. Clone the repository:
   ```
   git clone https://github.com/simplysabir/solconnect.git
   cd solconnect
   ```

2. Build the project:
   ```
   cargo build --release
   ```

3. The executable will be available in the `target/release` directory.

## Usage

1. Set up your Solana RPC endpoint:
   ```
   export SOLANA_RPC_ENDPOINT="https://your-rpc-endpoint.com/?api-key=your-api-key"
   ```

2. Run the CLI tool:
   ```
   ./target/release/solconnect <address1> <address2>
   ```
   Replace `<address1>` and `<address2>` with the Solana addresses you want to analyze.

## Example

```
./target/release/solconnect address1 address2
```

This command will analyze the connections between the two provided addresses and output the results.

## Output

The tool will provide the following information:

1. Number of transactions fetched for each address
2. Total number of unique transactions analyzed
3. Number of nodes (unique addresses) in the constructed graph
4. Paths found between the two input addresses
5. Detailed path information, showing the sequence of addresses connecting the input addresses

## Configuration

- `SOLANA_RPC_ENDPOINT`: Set this environment variable to your preferred Solana RPC endpoint. If not set, the tool will use the default public endpoint, which may have rate limiting.

## Limitations

- The tool currently fetches up to 10,000 recent transactions per address. For addresses with more transactions, older connections might not be discovered.
- The analysis is based on direct interactions in transactions and may not capture all types of relationships between addresses.
- Performance may vary depending on the number of transactions and the complexity of connections between addresses.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Disclaimer

This tool is for informational purposes only. Always verify important information through official Solana explorers and documentation. The accuracy of the results depends on the data available through the RPC endpoint and the limitations of the analysis performed.