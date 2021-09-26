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
        return self

    def render(self) -> bytes:
        request_line = " ".join(self._request_line).encode()
        headers = " ".join(self._headers).encode()
        body = self._body or b""
        separator = b"\n\n\n" if body else b"\n"

        return request_line + b"\n" + headers + b"\n\n" + body + separator


class ResponseParser:
    def __init__(self, response_lines):
        self._lines = response_lines
        splitted = split_list(self._lines, b'\r\n')
        self._header = splitted[0]
        self._body = b''
        if len(splitted) > 1:
            self._body = b''.join(splitted[1])

    @property
    def status(self):
        return int(self._header[0].decode().split(' ')[1])

    @property
    def headers(self):
        return dict(map(lambda header: tuple(map(str.strip, header.decode().strip().lower().rpartition(':')[::2])), self._header[1:]))

    @property
    def body(self):
        return self._body


class Connection:
    def __init__(self, host, port, hostname):
        self.host = host
        self.port = port
        self.hostname = hostname
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.connect((self.host, self.port))
        self._wfile: BinaryIO = sock.makefile("wb", newline="\r\n\r\n")
        self._rfile: BinaryIO = sock.makefile("rb", newline="\r\n\r\n")

    def request(self, request) -> ResponseParser:
        self._wfile.write(request)
        self._wfile.flush()
        response = self._rfile.readlines()
        return ResponseParser(response)
