from server import server

request_query_parameters = b"GET /?query_parameter=123456 HTTP/1.1\r\n" \
                           b"Header: 123\r\n" \
                           b"Another-Header: True\r\n" \
                           b"\r\n" \
                           b"This is some body text" \
                           b"\r\n\r\n\r\n"

request = b"GET / HTTP/1.1\r\n" \
          b"Header: 123\r\n" \
          b"Another-Header: True\r\n" \
          b"\r\n" \
          b"This is some body text" \
          b"\r\n\r\n\r\n"


def test_http_request_from_bytes(grader_request_parsing_test):
    r = server.HTTPRequest.from_bytes(request_query_parameters)

    assert r.method == "GET"
    assert r.path == "/"
    assert r.version == "1.1"
    assert r.parameters == {"query_parameter": "123456"}
    assert r.headers == {"Header": "123", "Another-Header": "True"}
    assert r.body == b"This is some body text"


def test_http_request_without_query_parameters(grader_request_parsing_test):
    r = server.HTTPRequest.from_bytes(request)
    assert r.method == "GET"
    assert r.path == "/"
    assert r.version == "1.1"
    assert r.parameters == {}
    assert r.headers == {"Header": "123", "Another-Header": "True"}
    assert r.body == b"This is some body text"


def test_http_request_to_bytes(grader_request_parsing_test):
    assert server.HTTPRequest.from_bytes(request_query_parameters).to_bytes().strip() == request_query_parameters.strip()


response_data = b"HTTP/1.1 200 OK\r\n" \
                b"Content-Type: application/octet-stream\r\n" \
                b"Content-Length: 10\r\n" \
                b"\r\n" \
                b"1234567890"


def test_http_response_from_bytes(grader_response_parsing_test):
    r = server.HTTPResponse.from_bytes(response_data)

    assert r.status == '200'
    assert r.version == '1.1'
    assert r.body == b'1234567890'
    assert r.headers['Content-Type'] == 'application/octet-stream'
    assert r.headers['Content-Length'] == '10'


def test_http_response_to_bytes(grader_response_parsing_test):
    assert server.HTTPResponse.from_bytes(response_data).to_bytes() == response_data
