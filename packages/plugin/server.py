import socket
import struct
import logging

SPAN_START_FORMAT = "QQQ"  # 4 * uint64_t (thread_id, time, label_id)
SPAN_END_FORMAT = "QQ"  # 2 * uint64_t (thread_id, time)
INITIAL_MESSAGE_FORMAT = "QqqQ"  # InitialMessage format


def recv_exactly(client_socket, num_bytes):
    """Receive the exact number of bytes from the socket."""
    chunks = []
    bytes_received = 0
    while bytes_received < num_bytes:
        chunk = client_socket.recv(num_bytes - bytes_received)
        if not chunk:
            return None  # Connection closed or incomplete data
        chunks.append(chunk)
        bytes_received += len(chunk)
    return b"".join(chunks)


def handle_client_connection(client_socket):
    try:
        initial_message_data = recv_exactly(client_socket, 40)
        if not initial_message_data:
            logging.error("Failed to read InitialMessage")
            return

        tsc_frequency, anchor_seconds, anchor_nanoseconds, anchor_timestamp = struct.unpack(
            INITIAL_MESSAGE_FORMAT, initial_message_data
        )

        # Log the InitialMessage details
        logging.info(
            f"InitialMessage:\n"
            f"  tsc_frequency: {tsc_frequency}\n"
            f"  anchor_seconds: {anchor_seconds}\n"
            f"  anchor_nanoseconds: {anchor_nanoseconds}\n"
            f"  anchor_timestamp: {anchor_timestamp}\n"
            f"{'-' * 40}"
        )

        while True:
            message_tag_data = recv_exactly(client_socket, 8)
            if not message_tag_data:
                break

            (message_tag,) = struct.unpack("Q", message_tag_data)

            # Process based on the message tag
            if message_tag == 0:  # SpanStart
                # Read the rest of SpanStart (24 more bytes)
                span_start_data = recv_exactly(client_socket, 24)
                if not span_start_data:
                    break

                # Unpack the SpanStart data
                thread_id, time, label_id = struct.unpack(
                    SPAN_START_FORMAT, span_start_data
                )

                # Log the SpanStart message
                logging.info(
                    f"SpanStart:\n"
                    f"  thread id: {thread_id}\n"
                    f"  time: {time}\n"
                    f"  label: {label_id}\n"
                    f"{'-' * 40}"
                )

            elif message_tag == 1:  # SpanEnd
                # Read the rest of SpanEnd (8 more bytes)
                span_end_data = recv_exactly(client_socket, 16)
                if not span_end_data:
                    break

                # Unpack the SpanEnd data
                thread_id, time = struct.unpack(
                    SPAN_END_FORMAT, span_end_data
                )

                # Log the SpanEnd message
                logging.info(
                    f"SpanEnd:\n"
                    f"  thread id: {thread_id}\n"
                    f"  time: {time}\n"
                    f"{'-' * 40}"
                )

            else:
                logging.error(f"unknown message tag: {message_tag}")

    except Exception as e:
        logging.error(f"Error handling client: {e}")
    finally:
        client_socket.close()


def start_server(host="0.0.0.0", port=9090):
    logging.basicConfig(
        level=logging.INFO, format="%(asctime)s - %(levelname)s - %(message)s"
    )
    server_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    server_socket.bind((host, port))
    server_socket.listen(1)

    logging.info(f"server started on {host}:{port}")

    try:
        while True:
            logging.info("waiting for a connection...")
            client_sock, address = server_socket.accept()
            logging.info(f"accepted connection from {address}")

            handle_client_connection(client_sock)

            logging.info("closed")
    except KeyboardInterrupt:
        logging.info("server shutting down.")
    finally:
        server_socket.close()


if __name__ == "__main__":
    start_server()
