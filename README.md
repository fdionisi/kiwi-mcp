# Kiwi MCP (Model Context Protocol)

A lightweight service that connects Large Language Models to the Kiwi flight search API, enabling AI assistants to search for and recommend flights.

## Features

- Implements the Context Server RPC protocol
- Provides a `plan_trip` tool that searches the Kiwi flight database
- Returns formatted flight information including prices, times, and booking links

## Requirements

- Rust toolchain
- `KIWI_API_KEY` environment variable with your Tequila API key

## Tool Parameters

The `plan_trip` tool accepts these parameters:

- `fly_from`: IATA code of departure location (required)
- `fly_to`: IATA code of arrival location (required)
- `date_from`: Departure date in dd/mm/yyyy format (required)
- `date_to`: Latest departure date in dd/mm/yyyy format (required)
- `return_from`: Return departure date (optional)
- `return_to`: Latest return date (optional)
- `adults`: Number of adult passengers (default: 1)
- `children`: Number of child passengers (default: 0)
- `infants`: Number of infant passengers (default: 0)
- `selected_cabins`: Cabin class (M, W, C, F) (default: M)
- `curr`: Currency for prices (default: EUR)
- `max_stopovers`: Maximum stopovers (default: 2)
- `sort`: Sort by price, duration, date, or quality (default: price)
- `limit`: Maximum results to return (default: 5)

## License

MIT
