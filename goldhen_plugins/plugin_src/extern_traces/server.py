import socket
import struct
import logging

# Define the struct format for SpanStart and SpanEnd
SPAN_START_FORMAT = 'QQQQ'  # 4 * uint64_t (message_tag, thread_id, time, label_id)
SPAN_END_FORMAT = 'QQ'      # 2 * uint64_t (message_tag, time)

def handle_client_connection(client_socket):
    try:
        while True:
            # Read 8 bytes to determine the message type
            message_tag_data = client_socket.recv(8)
            if not message_tag_data:
                break

            # Unpack the message tag
            (message_tag,) = struct.unpack('Q', message_tag_data)

            # Process based on the message tag
            if message_tag == 0:  # SpanStart
                # Read the rest of SpanStart (24 more bytes)
                span_start_data = client_socket.recv(24)
                if not span_start_data:
                    break

                # Unpack the SpanStart data
                thread_id, time, label_id = struct.unpack('QQQ', span_start_data)

                # Log the SpanStart message
                logging.info(f"SpanStart:\n"
                             f"  message_tag: {message_tag}\n"
                             f"  thread id: {thread_id}\n"
                             f"  time: {time}\n"
                             f"  label: {label_id}\n"
                             f"{'-' * 40}")

            elif message_tag == 1:  # SpanEnd
                # Read the rest of SpanEnd (8 more bytes)
                span_end_data = client_socket.recv(8)
                if not span_end_data:
                    break

                # Unpack the SpanEnd data
                (time,) = struct.unpack('Q', span_end_data)

                # Log the SpanEnd message
                logging.info(f"SpanEnd:\n"
                             f"  message_tag: {message_tag}\n"
                             f"  time: {time}\n"
                             f"{'-' * 40}")

            else:
                logging.error(f"unknown message tag: {message_tag}")

    except Exception as e:
        logging.error(f"Error handling client: {e}")
    finally:
        client_socket.close()


def start_server(host='0.0.0.0', port=9090):
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
