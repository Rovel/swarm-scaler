# Docker Swarm Autoscaler Tool

This Rust-based tool monitors CPU usage across Docker Swarm nodes and scales services up or down when usage thresholds are crossed. It securely connects to the Docker API using TLS, dynamically discovers other manager nodes, and coordinates scaling actions to prevent conflicts.

## Features

- **Autoscaling**: Monitors CPU thresholds and scales services up or down as needed.
- **Distributed Consensus**: Ensures only one manager initiates scaling by requiring confirmations from peer managers.
- **Secure Communication**: Connects to the Docker API over TLS with client and server certificates.
- **Automatic Configuration**: Reads TLS certificates from a shared Docker Swarm volume for easy configuration across nodes.

## Requirements

- **Docker and Docker Swarm** installed on each node.
- **Rust and Cargo** (if building from source).
- **OpenSSL** for generating certificates.

## Setup

### 1. Clone the Repository

```bash
git clone https://github.com/yourusername/docker-swarm-autoscaler.git
cd docker-swarm-autoscaler
```

### 2. Generate TLS Certificates
To enable secure communication with the Docker API, generate CA, server, and client certificates. Use the setup_certs.sh script provided to automate this process.

Run the Script
```bash
chmod +x setup_certs.sh
./setup_certs.sh /path/to/your/tls/directory
```
This will create the following files in /path/to/your/tls/directory:

ca.pem - CA certificate
ca-key.pem - CA private key
server-cert.pem and server-key.pem - Server certificate and key (used by Docker)
client-cert.pem and client-key.pem - Client certificate and key (used by the Rust app)

### 3. Configure Docker to Use TLS
Copy the server certificates to a secure location on each Docker manager node (e.g., /etc/docker/certs.d) and update Docker's configuration:

Edit /etc/docker/daemon.json:

```json
{
  "hosts": ["unix:///var/run/docker.sock", "tcp://0.0.0.0:2376"],
  "tls": true,
  "tlscacert": "/etc/docker/certs.d/ca.pem",
  "tlscert": "/etc/docker/certs.d/server-cert.pem",
  "tlskey": "/etc/docker/certs.d/server-key.pem",
  "tlsverify": true
}
```
Restart Docker:

```bash
sudo systemctl restart docker
4. Build the Rust Application
If you prefer to build the image yourself:
```

```bash
docker build -t rust_scaler:latest .
```
### 5. Docker Compose Setup
docker-compose.yml
Here’s an example docker-compose.yml file that sets up the rust_scaler service in Docker Swarm. Adjust the paths to your certificate directory and service ID as necessary.

```yaml
version: '3.8'

services:
  rust_scaler:
    image: rust_scaler:latest
    deploy:
      mode: replicated
      replicas: 3
      update_config:
        parallelism: 1
        delay: 10s
      restart_policy:
        condition: on-failure
      placement:
        constraints:
          - node.role == manager  # Only deploy on manager nodes
    environment:
      SERVICE_ID: "your_service_id_here"
      CPU_THRESHOLD: "800000000"  # CPU usage threshold for scaling
      LOCAL_ADDR: "rust_scaler:4000"  # UDP port for peer communication
    networks:
      - swarm-network
    volumes:
      - tls_certs:/tls  # Mount the TLS certificates
    ports:
      - "4000:4000/udp"  # Expose the UDP port for communication

volumes:
  tls_certs:
    driver: local
    driver_opts:
      type: "none"
      o: "bind"
      device: "/path/to/your/tls/directory"  # Adjust path to actual TLS directory

networks:
  swarm-network:
    driver: overlay
```

### 6. Deploy to Docker Swarm
Initialize Docker Swarm if it’s not already initialized:

```bash
docker swarm init
```

Deploy the stack:

```bash
docker stack deploy -c docker-compose.yml scaler_stack
```

This command deploys the rust_scaler service to the Docker Swarm, where it will monitor CPU usage across nodes and automatically scale services based on the defined thresholds.

### 7. Verify Setup
To check if the rust_scaler service is running correctly, you can use the following command:

```bash
docker service ls
```

This will show the status of all services, including rust_scaler. You can also check the logs for more detailed output:

```bash
docker service logs scaler_stack_rust_scaler
```

Usage
The autoscaler tool will monitor CPU usage on Docker Swarm nodes and automatically scale up or down the specified service (SERVICE_ID) when thresholds are exceeded. Adjust CPU_THRESHOLD and other environment variables in docker-compose.yml to fine-tune the autoscaling behavior.

Environment Variables
SERVICE_ID: ID of the service to scale.
CPU_THRESHOLD: CPU usage threshold for scaling up.
LOCAL_ADDR: UDP address for inter-manager communication.

# License

This project is licensed under the MIT License.