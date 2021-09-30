from http_builder import RequestBuilder

LOREM_IPSUM = (
    "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor "
    "incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud "
    "exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute "
    "irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla "
    "pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia "
    "deserunt mollit anim id est laborum."
)


def test_update_directory_data(grader_http_put_test, connection, file_state):
    file_state["/test-directory"].mkdir(parents=True, exist_ok=True)

    request = (
        RequestBuilder()
            .method("PUT")
            .path("/test-directory")
            .version()
            .hostname(connection.hostname)
            .body(LOREM_IPSUM)
            .render()
    )

    response = connection.request(request)

    assert response.status == 409
    assert "content-length" in response.headers
    assert int(response.headers["content-length"]) > 0
    assert response.body.count() > 0


def test_update_data_in_file(grader_http_put_test, connection, file_state):
    file_state["/test-file"].unlink(missing_ok=True)
    file_state["/test-file"].write_text("Hello, world!")

    request = (
        RequestBuilder()
            .method("PUT")
            .path("/test-file")
            .version()
            .hostname(connection.hostname)
            .body(LOREM_IPSUM)
            .render()
    )

    response = connection.request(request)

    assert response.status == 200

    assert file_state['/test-file'].read_text() == LOREM_IPSUM
