# API Documentation

This document provides an overview of the available API endpoints for the Communex API client. It is intended for front-end developers to integrate with the backend services.

## Base URL

All API requests should be made to the following base URL:

```
http://127.0.0.1:8080
```

## Endpoints

### 1. Get Balance
- **URL**: `/balance/{address}`
- **Method**: GET
- **Description**: Retrieves the balance for the specified address.
- **Response**:
  - **200 OK**: Returns the balance as a string.
  - **500 Internal Server Error**: If an error occurs.

### 2. Transfer
- **URL**: `/transfer`
- **Method**: POST
- **Description**: Performs a transfer operation.
- **Request Body**:
  ```json
  {
    "from": "<sender_address>",
    "to": "<recipient_address>",
    "amount": <amount>,
    "denom": "<currency_denomination>"
  }
  ```
- **Response**:
  - **200 OK**: Returns the transfer result.
  - **500 Internal Server Error**: If an error occurs.

### 3. Sign Transaction
- **URL**: `/sign_transaction`
- **Method**: POST
- **Description**: Signs a transaction.
- **Request Body**:
  ```json
  {
    "transaction": "<transaction_data>"
  }
  ```
- **Response**:
  - **200 OK**: Returns confirmation of the signed transaction.
  - **500 Internal Server Error**: If an error occurs.

## Authentication

Currently, no authentication is required to access these endpoints.

## Error Handling

- **500 Internal Server Error**: Indicates a server-side error. Check logs for more details.

## Example Usage

Here are some example requests using `curl`:

```bash
# Get Balance
curl -X GET http://127.0.0.1:8080/balance/{address}

# Transfer
curl -X POST http://127.0.0.1:8080/transfer \
  -H "Content-Type: application/json" \
  -d '{"from": "sender_address", "to": "recipient_address", "amount": 100, "denom": "COMAI"}'

# Sign Transaction
curl -X POST http://127.0.0.1:8080/sign_transaction \
  -H "Content-Type: application/json" \
  -d '{"transaction": "transaction_data"}'
```

Please ensure that the server is running before making requests.
