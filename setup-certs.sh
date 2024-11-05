#!/bin/bash

# Directory to store generated certificates
CERT_DIR="${1:-./tls}"
mkdir -p "$CERT_DIR"

# Set default certificate details
DAYS=365
COUNTRY="US"
STATE="California"
CITY="San Francisco"
ORG="MyOrg"
OU="DevOps"

# CA certificate and key
CA_CERT="$CERT_DIR/ca.pem"
CA_KEY="$CERT_DIR/ca-key.pem"

# Server certificate and key
SERVER_CERT="$CERT_DIR/server-cert.pem"
SERVER_KEY="$CERT_DIR/server-key.pem"
SERVER_CSR="$CERT_DIR/server.csr"

# Client certificate and key
CLIENT_CERT="$CERT_DIR/client-cert.pem"
CLIENT_KEY="$CERT_DIR/client-key.pem"
CLIENT_CSR="$CERT_DIR/client.csr"

echo "Generating CA certificate..."
openssl req -x509 -newkey rsa:4096 -days $DAYS -nodes -keyout "$CA_KEY" -out "$CA_CERT" -subj "/C=$COUNTRY/ST=$STATE/L=$CITY/O=$ORG/OU=$OU/CN=MyDockerCA"

echo "Generating server key and certificate signing request (CSR)..."
openssl req -newkey rsa:4096 -nodes -keyout "$SERVER_KEY" -out "$SERVER_CSR" -subj "/C=$COUNTRY/ST=$STATE/L=$CITY/O=$ORG/OU=$OU/CN=localhost"

echo "Signing server certificate with CA..."
openssl x509 -req -in "$SERVER_CSR" -CA "$CA_CERT" -CAkey "$CA_KEY" -CAcreateserial -out "$SERVER_CERT" -days $DAYS -sha256

echo "Generating client key and certificate signing request (CSR)..."
openssl req -newkey rsa:4096 -nodes -keyout "$CLIENT_KEY" -out "$CLIENT_CSR" -subj "/C=$COUNTRY/ST=$STATE/L=$CITY/O=$ORG/OU=$OU/CN=MyDockerClient"

echo "Signing client certificate with CA..."
openssl x509 -req -in "$CLIENT_CSR" -CA "$CA_CERT" -CAkey "$CA_KEY" -CAcreateserial -out "$CLIENT_CERT" -days $DAYS -sha256

# Clean up CSR files as they are no longer needed
rm -f "$SERVER_CSR" "$CLIENT_CSR"

echo "Certificate generation complete. Files created:"
echo "CA Certificate:        $CA_CERT"
echo "CA Key:                $CA_KEY"
echo "Server Certificate:    $SERVER_CERT"
echo "Server Key:            $SERVER_KEY"
echo "Client Certificate:    $CLIENT_CERT"
echo "Client Key:            $CLIENT_KEY"
