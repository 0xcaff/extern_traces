import socket
import struct
import logging

SPAN_START_FORMAT = "QQQ"  # 4 * uint64_t (thread_id, time, label_id)
SPAN_END_FORMAT = "QQ"  # 2 * uint64_t (thread_id, time)
INITIAL_MESSAGE_FORMAT = "QqqQ"  # InitialMessage format
COUNTERS_UPDATE_FORMAT = "QQQQ"  # CountersUpdate format
SPAN_START_ADDITIONAL_DATA_FORMAT = SPAN_START_FORMAT + "Q"  # SpanStartAdditionalData format


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


def recv_string(client_socket):
    """Receive a length-prefixed string."""
    length_data = recv_exactly(client_socket, 4)
    if not length_data:
        return None
    length = struct.unpack("I", length_data)[0]
    return recv_exactly(client_socket, length).decode('utf-8')


def handle_client_connection(client_socket):
    try:
        initial_message_data = recv_exactly(client_socket, 32)
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

        # Read module metadata
        module_count_data = recv_exactly(client_socket, 4)
        if not module_count_data:
            logging.error("Failed to read module count")
            return
        module_count = struct.unpack("I", module_count_data)[0]

        logging.info(f"Module count: {module_count}")
        for i in range(module_count):
            module_data = recv_exactly(client_socket,
                                       4)  # module_id (2 bytes) + version_major (1 byte) + version_minor (1 byte)
            if not module_data:
                logging.error(f"Failed to read module data for module {i}")
                return
            module_id, version_major, version_minor = struct.unpack("HBB", module_data)
            module_name = recv_string(client_socket)
            if module_name is None:
                logging.error(f"Failed to read module name for module {i}")
                return
            logging.info(
                f"Module {i}:\n  ID: {module_id}\n  Version: {version_major}.{version_minor}\n  Name: {module_name}")

        # Read library metadata
        library_count_data = recv_exactly(client_socket, 4)
        if not library_count_data:
            logging.error("Failed to read library count")
            return
        library_count = struct.unpack("I", library_count_data)[0]

        logging.info(f"Library count: {library_count}")
        for i in range(library_count):
            library_data = recv_exactly(client_socket, 4)  # library_id (2 bytes) + version (2 bytes)
            if not library_data:
                logging.error(f"Failed to read library data for library {i}")
                return
            library_id, version = struct.unpack("HH", library_data)
            library_name = recv_string(client_socket)
            if library_name is None:
                logging.error(f"Failed to read library name for library {i}")
                return
            logging.info(f"Library {i}:\n  ID: {library_id}\n  Version: {version}\n  Name: {library_name}")

        # Read symbol information
        symbol_count_data = recv_exactly(client_socket, 4)
        if not symbol_count_data:
            logging.error("Failed to read symbol count")
            return
        symbol_count = struct.unpack("I", symbol_count_data)[0]

        logging.info(f"Symbol count: {symbol_count}")
        for i in range(symbol_count):
            symbol_name = recv_string(client_socket)
            if symbol_name is None:
                logging.error(f"Failed to read symbol name for symbol {i}")
                return
            symbol_data = recv_exactly(client_socket, 2)  # library_id (1 byte) + module_id (1 byte)
            if not symbol_data:
                logging.error(f"Failed to read symbol data for symbol {i}")
                return
            library_id, module_id = struct.unpack("BB", symbol_data)
            logging.info(f"Symbol {i}:\n  Name: {symbol_name}\n  Library ID: {library_id}\n  Module ID: {module_id}")

        logging.info(f"{'-' * 40}")

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
                # logging.info(
                #     f"SpanStart:\n"
                #     f"  thread id: {thread_id}\n"
                #     f"  time: {time}\n"
                #     f"  label: {label_id}\n"
                #     f"{'-' * 40}"
                # )

            elif message_tag == 1:  # SpanEnd
                span_end_data = recv_exactly(client_socket, 16)
                if not span_end_data:
                    break

                # Unpack the SpanEnd data
                thread_id, time = struct.unpack(
                    SPAN_END_FORMAT, span_end_data
                )

                # Log the SpanEnd message
                # logging.info(
                #     f"SpanEnd:\n"
                #     f"  thread id: {thread_id}\n"
                #     f"  time: {time}\n"
                #     f"{'-' * 40}"
                # )

            elif message_tag == 2:  # CountersUpdate
                counters_update_data = recv_exactly(client_socket, 32)
                if not counters_update_data:
                    break

                # Unpack the CountersUpdate data
                thread_id, dropped_packets_delta, last_time, time = struct.unpack(
                    COUNTERS_UPDATE_FORMAT, counters_update_data
                )

                # Log the CountersUpdate message
                logging.info(
                    f"CountersUpdate:\n"
                    f"  thread id: {thread_id}\n"
                    f"  dropped packets delta: {dropped_packets_delta}\n"
                    f"  last time: {last_time}\n"
                    f"  time: {time}\n"
                    f"{'-' * 40}"
                )

            elif message_tag == 3:  # SpanStartAdditionalData
                span_start_additional_data = recv_exactly(client_socket, 32)
                if not span_start_additional_data:
                    break

                # Unpack the SpanStartAdditionalData header
                thread_id, time, label_id, extra_data_length = struct.unpack(
                    SPAN_START_ADDITIONAL_DATA_FORMAT, span_start_additional_data
                )

                # Read and discard the extra data
                extra_data = recv_exactly(client_socket, extra_data_length)
                if not extra_data:
                    break

                # Log the SpanStartAdditionalData message (without parsing the extra data)
                logging.info(
                    f"SpanStartAdditionalData:\n"
                    f"  thread id: {thread_id}\n"
                    f"  time: {time}\n"
                    f"  label id: {label_id}\n"
                    f"  extra data length: {extra_data_length}\n"
                    f"{'-' * 40}"
                )
            else:
                logging.error(f"unknown message tag: {message_tag}")

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
