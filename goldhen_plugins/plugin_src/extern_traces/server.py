import socket
import struct
import logging

# Define the struct format for Span (4 uint64_t = 4 * 8 bytes = 32 bytes)
SPAN_FORMAT = 'QQQQ'  # Q stands for uint64_t (8 bytes)


def handle_client_connection(client_socket):
    try:
        while True:
            # Read 32 bytes from the client
            data = client_socket.recv(32)
            if not data:
                break

            # Unpack the data according to the Span struct
            thread_id, start_time, end_time, label_id = struct.unpack(SPAN_FORMAT, data)

            # Log the received span across multiple lines
            logging.info(f"span:\n"
                         f"  thread id: {thread_id}\n"
                         f"  start: {start_time}\n"
                         f"  end: {end_time}\n"
                         f"  label: {label_id}\n"
                         f"{'-' * 40}")  # Separator line for readability
    except Exception as e:
        logging.error(f"Error handling client: {e}")
    finally:
        client_socket.close()


def start_server(host='0.0.0.0', port=9999):
    logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
    server_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    server_socket.bind((host, port))
    server_socket.listen(1)

    logging.info(f"server started on {host}:{port}")

    try:
        while True:
            logging.info("waiting for a connection...")
            client_sock, address = server_socket.accept()
            logging.info(f"accepted connection from {address}")

            # Handle the client connection
            handle_client_connection(client_sock)

            logging.info("closed")
    except KeyboardInterrupt:
        logging.info("server shutting down.")
    finally:
        server_socket.close()


if __name__ == "__main__":
    start_server()
