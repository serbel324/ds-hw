import pytest

from http_builder import RequestBuilder


@pytest.fixture(scope="function")
def remove_test_file(file_state):
    file_state["/test_file"].unlink(missing_ok=True)
    yield


@pytest.fixture(scope="function")
def create_test_file(file_state):
    file_state["/test_file"].write_text("Hello, world!")


@pytest.fixture(scope="function")
def request_create_test_file(connection):
    request = (
        RequestBuilder()
            .method("POST")
            .path("/test_file")
            .version()
            .hostname(connection.hostname)
            .body("Hello, world!")
            .render()
    )

    response = connection.request(request)
    yield response


def test_normal_create_file(
    grader_http_post_test, remove_test_file, request_create_test_file, file_state
):
    assert request_create_test_file.status == 200
    assert file_state["/test_file"].read_text() == "Hello, world!"


def test_file_already_exists(
    grader_http_post_test, create_test_file, request_create_test_file
):
    assert request_create_test_file.status == 409
    assert "content-length" in request_create_test_file.headers
    assert int(request_create_test_file.headers["content-length"]) > 0
    assert request_create_test_file.body.count() > 0


def test_create_directory(grader_http_post_test, connection, file_state):
    request = (
        RequestBuilder()
            .method("POST")
            .path("/test-directory")
            .version()
            .hostname(connection.hostname)
            .header("Create-diRecTORy", "True")
            .body("Trololo")
            .render()
    )
    response = connection.request(request)
    assert response.status == 200

    assert file_state["/test-directory"].is_dir()


def test_file_create_invalid_path(grader_http_post_test, connection):
    request = (
        RequestBuilder()
            .method("POST")
            .path("some-path/../../../../we-cannot-do-there")
            .version()
            .hostname(connection.hostname)
            .body("Trololo")
            .render()
    )
    response = connection.request(request)

    assert response.status == 400
    assert "content-length" in request_create_test_file.headers
    assert int(request_create_test_file.headers["content-length"]) > 0
    assert request_create_test_file.body.count() > 0
