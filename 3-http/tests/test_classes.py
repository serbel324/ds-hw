import pytest
from server import server


def test_http_request_from_bytes(grader_request_parsing_test):
    request_bytes = b"GET /?query_parameter=123456 HTTP/1.1\r\nHeader: 123\r\nAnother-Header: True\r\n\r\nThis is some body text\r\n\r\n\r\n"

    r = server.HTTPRequest.from_bytes(request_bytes)

    assert r.method == "GET"
    assert r.path == "/"
    assert r.version == "1.1"
    assert r.parameters == {"query_parameter": "123456"}
    assert r.headers == {"Header": "123", "Another-Header": "True"}
    assert r.body == b"This is some body text"


def test_http_request_to_bytes(grader_request_parsing_test):
    request_bytes = b"GET /?query_parameter=123456 HTTP/1.1\r\nHeader: 123\r\nAnother-Header: True\r\n\r\nThis is some body text\r\n\r\n\r\n"
    assert server.HTTPRequest.from_bytes(request_bytes).to_bytes() == request_bytes
