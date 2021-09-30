import logging
import pathlib
import socket
from typing import BinaryIO
from typing import BinaryIO


def split_list(l, separator):
    parts = []
    current_part = []
    for element in l:
        if element == separator:
            parts.append(current_part)
            current_part = []
            continue
        current_part.append(element)
    parts.append(current_part)
    return parts


class RequestBuilder:
    def __init__(self):
        self._request_line = []
        self._headers = []
        self._body = None

    def method(self, str):
        self._request_line.append(str)
        return self

    def path(self, path):
        self._request_line.append(path)
        return self

    def version(self, version="1.1"):
        self._request_line.append(f"HTTP/{version}")
        return self

    def header(self, name, value):
        self._headers.append(f"{name}: {value}")
        return self

    def hostname(self, hostname):
        self._headers.append(f"Host: {hostname}")
        return self

    def body(self, body):
        if isinstance(body, pathlib.Path):
            self._body = body.read_bytes()
        elif isinstance(body, str):
            self._body = body.encode()
        elif isinstance(body, bytes):
            self._body = body
        self.header("Content-Length", len(self._body))
        self.header("Content-Type", "application/octet-stream")
        return self

    def render(self) -> bytes:
        # import pdb; pdb.set_trace()
        request_line = " ".join(self._request_line).encode()
        headers = '\r\n'.join(self._headers).encode()
        body = self._body or b""
        separator = b"\r\n\r\n\r\n" if body else b"\r\n"
        result = request_line + b"\r\n" + headers + b"\r\n\r\n" + body + separator
        return result


class ResponseParser:
    def __init__(self, response_lines):
        self._lines = response_lines
        splitted = split_list(self._lines, b"\r\n")
        self._header = splitted[0]
        self._body = b""
        if len(splitted) > 1:
            self._body = b"".join(splitted[1])

    @property
    def status(self):
        return int(self._header[0].decode().split(" ")[1])

    @property
    def status_message(self):
        return str(" ".join(self._header[0].decode().split(" ")[2:]))

    @property
    def headers(self):
        return dict(
            map(
                lambda header: tuple(
                    map(str.strip, header.decode().strip().lower().rpartition(":")[::2])
                ),
                self._header[1:],
            )
        )

    @property
    def body(self):
        return self._body

    def __bool__(self):
        return bool(self._lines)


class Connection:
    def __init__(self, host, port, hostname):
        self.host = host
        self.port = port
        self.hostname = hostname
        self.sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.sock.connect((self.host, self.port))
        self._wfile: BinaryIO = self.sock.makefile("wb", newline="\r\n\r\n")
        self._rfile: BinaryIO = self.sock.makefile("rb", newline="\r\n\r\n")

    def request(self, request) -> ResponseParser:
        self._wfile.write(request)
        self._wfile.flush()

        self.sock.shutdown(socket.SHUT_WR)

        response = self._rfile.readlines()
        logging.info(response)
        try:
            self.sock.shutdown(socket.SHUT_RD)
        except OSError as e:
            # Server has already closed the connection, continue execution
            pass

        self.sock.close()
        return ResponseParser(response)

    def __str__(self):
        return f"Connection({repr(self.host)}, {repr(self.port)}, {repr(self.hostname)})"

    __repr__ = __str__
