import pathlib

import pytest
import os

from file_state import FileState
from grader import Grader
from http import Connection


@pytest.hookimpl(tryfirst=True, hookwrapper=True)
def pytest_runtest_makereport(item, call):
    # execute all other hooks to obtain the report object
    outcome = yield
    rep = outcome.get_result()

    # set a report attribute for each phase of a call, which can
    # be "setup", "call", "teardown"

    setattr(item, "rep_" + rep.when, rep)


@pytest.fixture(scope="session")
def grader():
    g = Grader()

    import atexit

    def report():
        print(g)

    atexit.register(report)

    yield g


@pytest.fixture(scope="function")
def grader_http_get_test(request, grader):
    grader.http_method_get_total_tests += 1
    yield
    if request.node.rep_setup.passed and request.node.rep_call.passed:
        grader.http_method_get_passed_tests += 1


@pytest.fixture(scope="function")
def grader_http_post_test(request, grader):
    grader.http_method_post_total_tests += 1
    yield
    if request.node.rep_setup.passed and request.node.rep_call.passed:
        grader.http_method_post_passed_tests += 1


@pytest.fixture(scope="function")
def grader_http_put_test(request, grader):
    grader.http_method_put_total_tests += 1
    yield
    if request.node.rep_setup.passed and request.node.rep_call.passed:
        grader.http_method_put_passed_tests += 1


@pytest.fixture(scope="function")
def grader_http_delete_test(request, grader):
    grader.http_method_delete_total_tests += 1
    yield
    if request.node.rep_setup.passed and request.node.rep_call.passed:
        grader.http_method_delete_passed_tests += 1


@pytest.fixture(scope="function")
def grader_request_parsing_test(request, grader):
    grader.http_request_parsing_total_tests += 1
    yield
    if request.node.rep_setup.passed and request.node.rep_call.passed:
        grader.http_request_parsing_passed_tests += 1


@pytest.fixture(scope="function")
def grader_response_parsing_test(request, grader):
    grader.http_response_parsing_total_tests += 1
    yield
    if request.node.rep_setup.passed and request.node.rep_call.passed:
        grader.http_response_parsing_total_tests += 1


@pytest.fixture(scope="module")
def connection():
    host = os.environ.get("SERVER_HOST", "localhost")
    port = int(os.environ.get("SERVER_PORT", 8000))
    hostname = os.environ.get("SERVER_HOSTNAME", "example.domain")

    yield Connection(host, port, hostname)


@pytest.fixture(scope="session")
def files_path():
    yield pathlib.Path(os.environ.get("SERVER_WORKING_DIRECTORY", './base_data'))


@pytest.fixture(scope="session")
def file_state(files_path):
    yield FileState(files_path)
